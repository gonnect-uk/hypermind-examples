//! SIMD Benchmark - Measure Performance Improvements
//!
//! This benchmark compares SIMD-optimized implementations against
//! scalar baselines using Criterion for statistical analysis.
//!
//! Run with: cargo +nightly bench --features simd --bench simd_benchmark

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rdf_model::{Dictionary, Node};
use smallvec::SmallVec;
use std::sync::Arc;

#[cfg(feature = "simd")]
use storage::simd_encode::{encode_nodes_batch_simd, prefix_compare_simd};

// Scalar baseline for encoding
fn encode_nodes_scalar(nodes: &[Node]) -> Vec<u8> {
    let mut output = Vec::new();
    for node in nodes {
        let mut buf = SmallVec::<[u8; 256]>::new();
        // Note: encode_node is now public in indexes module
        unsafe {
            // Use internal function directly for benchmarking
            // This is safe because we're just calling a pure function
            use smallvec::SmallVec as SV;
            let encode_ptr: unsafe fn(&mut SV<[u8; 256]>, &rdf_model::Node) = std::mem::transmute(storage::Index::insert as *const ());
            // Actually, let's use a simpler approach - call via QuadStore
            // For now, duplicate the encoding logic for benchmark baseline
        }
        // Simplified scalar encoding for benchmark baseline
        match node {
            Node::Iri(uri) => {
                buf.push(0u8); // IRI type
                let bytes = uri.as_bytes();
                encode_varint_bench(&mut buf, bytes.len() as u64);
                buf.extend_from_slice(bytes);
            }
            Node::Literal(lit) => {
                buf.push(1u8); // Literal type
                let bytes = lit.lexical_form.as_bytes();
                encode_varint_bench(&mut buf, bytes.len() as u64);
                buf.extend_from_slice(bytes);
                // Simplified: skip language/datatype for baseline
                buf.push(0); buf.push(0);
            }
            Node::BlankNode(id) => {
                buf.push(2u8); // Blank type
                let id_str = id.0.to_string();
                let bytes = id_str.as_bytes();
                encode_varint_bench(&mut buf, bytes.len() as u64);
                buf.extend_from_slice(bytes);
            }
            _ => {
                let val = node.value();
                buf.push(4u8);
                let bytes = val.as_bytes();
                encode_varint_bench(&mut buf, bytes.len() as u64);
                buf.extend_from_slice(bytes);
            }
        }
        output.extend_from_slice(&buf);
    }
    output
}

fn encode_varint_bench(buf: &mut SmallVec<[u8; 256]>, mut value: u64) {
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

// Scalar baseline for prefix comparison
fn prefix_compare_scalar(data: &[u8], prefix: &[u8]) -> bool {
    data.starts_with(prefix)
}

fn benchmark_node_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_encoding");

    // Test different batch sizes
    for size in [4, 10, 50, 100, 500, 1000].iter() {
        let dict = Arc::new(Dictionary::new());

        // Create test data
        let nodes: Vec<_> = (0..*size)
            .map(|i| Node::iri(dict.intern(&format!("http://example.org/resource{}", i))))
            .collect();

        // Benchmark scalar
        group.bench_with_input(BenchmarkId::new("scalar", size), &nodes, |b, nodes| {
            b.iter(|| {
                let result = encode_nodes_scalar(nodes);
                black_box(result)
            });
        });

        // Benchmark SIMD (if enabled)
        #[cfg(feature = "simd")]
        group.bench_with_input(BenchmarkId::new("simd", size), &nodes, |b, nodes| {
            b.iter(|| {
                let result = encode_nodes_batch_simd(nodes);
                black_box(result)
            });
        });
    }

    group.finish();
}

fn benchmark_node_encoding_mixed_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_encoding_mixed");

    let dict = Arc::new(Dictionary::new());

    // Create mixed-type nodes
    let nodes: Vec<_> = (0..100)
        .flat_map(|i| {
            vec![
                Node::iri(dict.intern(&format!("http://example.org/s{}", i))),
                Node::literal_str(dict.intern(&format!("value{}", i))),
                Node::blank(i),
                Node::literal_typed(
                    dict.intern(&format!("{}", i)),
                    dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
                ),
            ]
        })
        .collect();

    // Scalar baseline
    group.bench_function("scalar", |b| {
        b.iter(|| {
            let result = encode_nodes_scalar(&nodes);
            black_box(result)
        });
    });

    // SIMD optimized
    #[cfg(feature = "simd")]
    group.bench_function("simd", |b| {
        b.iter(|| {
            let result = encode_nodes_batch_simd(&nodes);
            black_box(result)
        });
    });

    group.finish();
}

fn benchmark_prefix_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("prefix_compare");

    // Test different data sizes
    for size in [16, 32, 64, 128, 256].iter() {
        let data: Vec<u8> = (0..*size).map(|i| (i % 256) as u8).collect();

        // Short prefix (< 16 bytes)
        let short_prefix = &data[..8];
        group.bench_with_input(
            BenchmarkId::new("scalar_short", size),
            &(&data, short_prefix),
            |b, (d, p)| {
                b.iter(|| {
                    let result = prefix_compare_scalar(d, p);
                    black_box(result)
                });
            },
        );

        #[cfg(feature = "simd")]
        group.bench_with_input(
            BenchmarkId::new("simd_short", size),
            &(&data, short_prefix),
            |b, (d, p)| {
                b.iter(|| {
                    let result = prefix_compare_simd(d, p);
                    black_box(result)
                });
            },
        );

        // Long prefix (> 16 bytes, exercises SIMD chunking)
        if *size >= 32 {
            let long_prefix = &data[..24];
            group.bench_with_input(
                BenchmarkId::new("scalar_long", size),
                &(&data, long_prefix),
                |b, (d, p)| {
                    b.iter(|| {
                        let result = prefix_compare_scalar(d, p);
                        black_box(result)
                    });
                },
            );

            #[cfg(feature = "simd")]
            group.bench_with_input(
                BenchmarkId::new("simd_long", size),
                &(&data, long_prefix),
                |b, (d, p)| {
                    b.iter(|| {
                        let result = prefix_compare_simd(d, p);
                        black_box(result)
                    });
                },
            );
        }
    }

    group.finish();
}

fn benchmark_realistic_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workload");

    let dict = Arc::new(Dictionary::new());

    // Simulate LUBM-style data (university ontology)
    let nodes: Vec<_> = (0..1000)
        .flat_map(|i| {
            vec![
                // Subject: University, Department, Professor, Student, Course
                Node::iri(dict.intern(&format!(
                    "http://www.University{}.edu/Department{}",
                    i / 100,
                    i % 15
                ))),
                // Predicate: type, name, worksFor, takesCourse
                Node::iri(dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")),
                // Object: literals and URIs
                Node::literal_str(dict.intern(&format!("Entity{}", i))),
            ]
        })
        .collect();

    println!("\n=== Realistic Workload ===");
    println!("Nodes: {}", nodes.len());
    println!("Simulating LUBM university ontology data");

    // Scalar baseline
    group.bench_function("scalar", |b| {
        b.iter(|| {
            let result = encode_nodes_scalar(&nodes);
            black_box(result)
        });
    });

    // SIMD optimized
    #[cfg(feature = "simd")]
    group.bench_function("simd", |b| {
        b.iter(|| {
            let result = encode_nodes_batch_simd(&nodes);
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_node_encoding,
    benchmark_node_encoding_mixed_types,
    benchmark_prefix_comparison,
    benchmark_realistic_workload
);

criterion_main!(benches);
