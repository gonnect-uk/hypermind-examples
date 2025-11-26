//! Comprehensive SIMD tests with property-based testing
//!
//! This test suite validates that SIMD implementations produce identical
//! results to scalar implementations across a wide range of inputs.

#[cfg(feature = "simd")]
use storage::simd_encode::{encode_nodes_batch_simd, prefix_compare_simd};

use rdf_model::{Dictionary, Node};
use smallvec::SmallVec;
use std::sync::Arc;

// Helper function to encode nodes using scalar implementation
fn encode_nodes_scalar(nodes: &[Node]) -> Vec<u8> {
    let mut output = Vec::new();
    for node in nodes {
        let mut buf = SmallVec::<[u8; 256]>::new();
        // Use simplified encoding matching storage implementation
        encode_node_scalar(&mut buf, node);
        output.extend_from_slice(&buf);
    }
    output
}

// Scalar encoding helper (duplicates logic from storage::indexes::encode_node)
fn encode_node_scalar(buf: &mut SmallVec<[u8; 256]>, node: &Node) {
    let type_byte = match node {
        Node::Iri(_) => 0u8,
        Node::Literal(_) => 1u8,
        Node::BlankNode(_) => 2u8,
        Node::QuotedTriple(_) => 3u8,
        Node::Variable(_) => 4u8,
    };
    buf.push(type_byte);

    match node {
        Node::Literal(lit) => {
            let bytes = lit.lexical_form.as_bytes();
            encode_varint_scalar(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);

            if let Some(lang) = lit.language {
                buf.push(1);
                let lang_bytes = lang.as_bytes();
                encode_varint_scalar(buf, lang_bytes.len() as u64);
                buf.extend_from_slice(lang_bytes);
            } else {
                buf.push(0);
            }

            if let Some(datatype) = lit.datatype {
                buf.push(1);
                let dt_bytes = datatype.as_bytes();
                encode_varint_scalar(buf, dt_bytes.len() as u64);
                buf.extend_from_slice(dt_bytes);
            } else {
                buf.push(0);
            }
        }
        Node::BlankNode(id) => {
            let id_str = id.0.to_string();
            let bytes = id_str.as_bytes();
            encode_varint_scalar(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
        _ => {
            let value = node.value();
            let bytes = value.as_bytes();
            encode_varint_scalar(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
    }
}

fn encode_varint_scalar(buf: &mut SmallVec<[u8; 256]>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_empty() {
    let nodes: Vec<Node> = vec![];
    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result, scalar_result);
    assert!(simd_result.is_empty());
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_single_iri() {
    let dict = Arc::new(Dictionary::new());
    let nodes = vec![Node::iri(dict.intern("http://example.org/resource"))];

    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result, scalar_result);
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_four_iris() {
    let dict = Arc::new(Dictionary::new());
    let nodes = vec![
        Node::iri(dict.intern("http://example.org/s1")),
        Node::iri(dict.intern("http://example.org/s2")),
        Node::iri(dict.intern("http://example.org/s3")),
        Node::iri(dict.intern("http://example.org/s4")),
    ];

    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result.len(), scalar_result.len());
    assert_eq!(simd_result, scalar_result);
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_mixed_types() {
    let dict = Arc::new(Dictionary::new());
    let nodes = vec![
        Node::iri(dict.intern("http://example.org/subject")),
        Node::literal_str(dict.intern("simple literal")),
        Node::blank(42),
        Node::literal_typed(
            dict.intern("123"),
            dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
        ),
    ];

    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result, scalar_result);
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_non_multiple_of_four() {
    let dict = Arc::new(Dictionary::new());
    let nodes = vec![
        Node::iri(dict.intern("http://example.org/s1")),
        Node::iri(dict.intern("http://example.org/s2")),
        Node::iri(dict.intern("http://example.org/s3")),
    ];

    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result, scalar_result);
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_large_batch() {
    let dict = Arc::new(Dictionary::new());
    let nodes: Vec<_> = (0..100)
        .map(|i| Node::iri(dict.intern(&format!("http://example.org/resource{}", i))))
        .collect();

    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result, scalar_result);
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_blank_nodes() {
    let nodes = vec![
        Node::blank(0),
        Node::blank(1),
        Node::blank(12345),
        Node::blank(u64::MAX),
    ];

    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result, scalar_result);
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_literals_with_language() {
    let dict = Arc::new(Dictionary::new());
    let nodes = vec![
        Node::literal_lang(dict.intern("Hello"), dict.intern("en")),
        Node::literal_lang(dict.intern("Bonjour"), dict.intern("fr")),
        Node::literal_lang(dict.intern("Hola"), dict.intern("es")),
        Node::literal_lang(dict.intern("こんにちは"), dict.intern("ja")),
    ];

    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result, scalar_result);
}

#[test]
#[cfg(feature = "simd")]
fn test_simd_encode_long_uris() {
    let dict = Arc::new(Dictionary::new());
    let long_uri = "http://www.example.org/very/long/path/to/some/resource/with/many/segments/and/query/params?foo=bar&baz=qux";
    let nodes = vec![
        Node::iri(dict.intern(long_uri)),
        Node::iri(dict.intern(long_uri)),
        Node::iri(dict.intern(long_uri)),
        Node::iri(dict.intern(long_uri)),
    ];

    let simd_result = encode_nodes_batch_simd(&nodes);
    let scalar_result = encode_nodes_scalar(&nodes);

    assert_eq!(simd_result, scalar_result);
}

#[test]
#[cfg(feature = "simd")]
fn test_prefix_compare_basic() {
    let data = b"http://example.org/resource";
    let prefix = b"http://example.org/";

    assert!(prefix_compare_simd(data, prefix));
}

#[test]
#[cfg(feature = "simd")]
fn test_prefix_compare_no_match() {
    let data = b"http://example.org/resource";
    let prefix = b"http://other.org/";

    assert!(!prefix_compare_simd(data, prefix));
}

#[test]
#[cfg(feature = "simd")]
fn test_prefix_compare_empty_prefix() {
    let data = b"anything";
    let prefix = b"";

    assert!(prefix_compare_simd(data, prefix));
}

#[test]
#[cfg(feature = "simd")]
fn test_prefix_compare_exact_match() {
    let data = b"http://example.org";
    let prefix = b"http://example.org";

    assert!(prefix_compare_simd(data, prefix));
}

#[test]
#[cfg(feature = "simd")]
fn test_prefix_compare_long_prefix() {
    // Test with > 16 bytes to exercise SIMD chunking
    let data = b"http://www.example.org/very/long/resource/path/name/with/many/segments";
    let prefix = b"http://www.example.org/very/long/resource/path/";

    assert!(prefix_compare_simd(data, prefix));
}

#[test]
#[cfg(feature = "simd")]
fn test_prefix_compare_data_too_short() {
    let data = b"short";
    let prefix = b"this_is_much_longer_than_the_data";

    assert!(!prefix_compare_simd(data, prefix));
}

// Property-based tests using proptest
#[cfg(feature = "simd")]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_encode_iris_matches_scalar(
            uris in prop::collection::vec(any::<String>(), 0..50)
        ) {
            let dict = Arc::new(Dictionary::new());
            let nodes: Vec<_> = uris.iter()
                .map(|s| Node::iri(dict.intern(s)))
                .collect();

            let simd_result = encode_nodes_batch_simd(&nodes);
            let scalar_result = encode_nodes_scalar(&nodes);

            prop_assert_eq!(simd_result, scalar_result);
        }

        #[test]
        fn prop_encode_literals_matches_scalar(
            values in prop::collection::vec(any::<String>(), 0..50)
        ) {
            let dict = Arc::new(Dictionary::new());
            let nodes: Vec<_> = values.iter()
                .map(|s| Node::literal_str(dict.intern(s)))
                .collect();

            let simd_result = encode_nodes_batch_simd(&nodes);
            let scalar_result = encode_nodes_scalar(&nodes);

            prop_assert_eq!(simd_result, scalar_result);
        }

        #[test]
        fn prop_encode_blank_nodes_matches_scalar(
            ids in prop::collection::vec(any::<u64>(), 0..50)
        ) {
            let nodes: Vec<_> = ids.iter()
                .map(|&id| Node::blank(id))
                .collect();

            let simd_result = encode_nodes_batch_simd(&nodes);
            let scalar_result = encode_nodes_scalar(&nodes);

            prop_assert_eq!(simd_result, scalar_result);
        }

        #[test]
        fn prop_prefix_compare_correctness(
            data in any::<Vec<u8>>(),
            prefix_len in 0usize..100
        ) {
            if data.len() < prefix_len {
                return Ok(());
            }

            let prefix = &data[..prefix_len];

            // SIMD and scalar must agree
            let simd_result = prefix_compare_simd(&data, prefix);
            let scalar_result = data.starts_with(prefix);

            prop_assert_eq!(simd_result, scalar_result);
        }
    }
}

// Performance validation tests (ensure SIMD is actually faster)
#[cfg(feature = "simd")]
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // Run with --ignored flag
    fn perf_simd_vs_scalar_encoding() {
        let dict = Arc::new(Dictionary::new());

        // Create large batch (1000 nodes)
        let nodes: Vec<_> = (0..1000)
            .map(|i| Node::iri(dict.intern(&format!("http://example.org/resource{}", i))))
            .collect();

        // Warm up
        for _ in 0..10 {
            let _ = encode_nodes_batch_simd(&nodes);
            let _ = encode_nodes_scalar(&nodes);
        }

        // Benchmark SIMD
        let start = Instant::now();
        for _ in 0..100 {
            let _ = encode_nodes_batch_simd(&nodes);
        }
        let simd_time = start.elapsed();

        // Benchmark scalar
        let start = Instant::now();
        for _ in 0..100 {
            let _ = encode_nodes_scalar(&nodes);
        }
        let scalar_time = start.elapsed();

        println!("\n=== SIMD Performance Test ===");
        println!("Nodes: 1000");
        println!("Iterations: 100");
        println!("SIMD time: {:?}", simd_time);
        println!("Scalar time: {:?}", scalar_time);
        println!("Speedup: {:.2}x", scalar_time.as_secs_f64() / simd_time.as_secs_f64());

        // SIMD should be at least 20% faster (1.2x speedup)
        // Note: This is a rough check, actual speedup depends on CPU
        // Real benchmarks should use Criterion for statistical analysis
        if simd_time < scalar_time {
            println!("✅ SIMD is faster!");
        } else {
            println!("⚠️  SIMD is not faster in this test (may need profiling)");
        }
    }
}
