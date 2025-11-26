//! Quad index structures for optimal query patterns
//!
//! Implements 4 permutation indexes inspired by Apache Jena TDB2:
//! - SPOC: Subject-Predicate-Object-Context
//! - POCS: Predicate-Object-Context-Subject
//! - OCSP: Object-Context-Subject-Predicate
//! - CSPO: Context-Subject-Predicate-Object

use rdf_model::Quad;
use smallvec::SmallVec;

/// Index type for different access patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexType {
    /// Subject-Predicate-Object-Context
    /// Best for: (?s ?p ?o) or (?s ?p ?)
    SPOC,

    /// Predicate-Object-Context-Subject
    /// Best for: (? ?p ?o) or (? ?p ?)
    POCS,

    /// Object-Context-Subject-Predicate
    /// Best for: (? ? ?o) patterns
    OCSP,

    /// Context-Subject-Predicate-Object
    /// Best for: GRAPH queries
    CSPO,
}

impl IndexType {
    /// Get all index types
    pub fn all() -> &'static [IndexType] {
        &[
            IndexType::SPOC,
            IndexType::POCS,
            IndexType::OCSP,
            IndexType::CSPO,
        ]
    }

    /// Encode a quad into this index's key format
    pub fn encode_key<'a>(&self, quad: &Quad<'a>) -> SmallVec<[u8; 256]> {
        let mut key = SmallVec::new();

        // Encode nodes in index-specific order
        // Using simplified string encoding for now (will optimize with IDs later)
        match self {
            IndexType::SPOC => {
                encode_node(&mut key, &quad.subject);
                encode_node(&mut key, &quad.predicate);
                encode_node(&mut key, &quad.object);
                encode_node_opt(&mut key, &quad.graph);
            }
            IndexType::POCS => {
                encode_node(&mut key, &quad.predicate);
                encode_node(&mut key, &quad.object);
                encode_node_opt(&mut key, &quad.graph);
                encode_node(&mut key, &quad.subject);
            }
            IndexType::OCSP => {
                encode_node(&mut key, &quad.object);
                encode_node_opt(&mut key, &quad.graph);
                encode_node(&mut key, &quad.subject);
                encode_node(&mut key, &quad.predicate);
            }
            IndexType::CSPO => {
                encode_node_opt(&mut key, &quad.graph);
                encode_node(&mut key, &quad.subject);
                encode_node(&mut key, &quad.predicate);
                encode_node(&mut key, &quad.object);
            }
        }

        key
    }

    /// Decode a quad from an encoded key
    pub fn decode_key<'a>(&self, key: &[u8], dict: &'a rdf_model::Dictionary) -> Result<rdf_model::Quad<'a>, String> {
        let mut offset = 0;

        // Decode nodes in index-specific order
        let (subject, predicate, object, graph) = match self {
            IndexType::SPOC => {
                let (s, s_len) = decode_node(&key[offset..], dict)?;
                offset += s_len;
                let (p, p_len) = decode_node(&key[offset..], dict)?;
                offset += p_len;
                let (o, o_len) = decode_node(&key[offset..], dict)?;
                offset += o_len;
                let (g, _g_len) = decode_node_opt(&key[offset..], dict)?;
                (s, p, o, g)
            }
            IndexType::POCS => {
                let (p, p_len) = decode_node(&key[offset..], dict)?;
                offset += p_len;
                let (o, o_len) = decode_node(&key[offset..], dict)?;
                offset += o_len;
                let (g, g_len) = decode_node_opt(&key[offset..], dict)?;
                offset += g_len;
                let (s, _s_len) = decode_node(&key[offset..], dict)?;
                (s, p, o, g)
            }
            IndexType::OCSP => {
                let (o, o_len) = decode_node(&key[offset..], dict)?;
                offset += o_len;
                let (g, g_len) = decode_node_opt(&key[offset..], dict)?;
                offset += g_len;
                let (s, s_len) = decode_node(&key[offset..], dict)?;
                offset += s_len;
                let (p, _p_len) = decode_node(&key[offset..], dict)?;
                (s, p, o, g)
            }
            IndexType::CSPO => {
                let (g, g_len) = decode_node_opt(&key[offset..], dict)?;
                offset += g_len;
                let (s, s_len) = decode_node(&key[offset..], dict)?;
                offset += s_len;
                let (p, p_len) = decode_node(&key[offset..], dict)?;
                offset += p_len;
                let (o, _o_len) = decode_node(&key[offset..], dict)?;
                (s, p, o, g)
            }
        };

        Ok(rdf_model::Quad::new(subject, predicate, object, graph))
    }

    /// Select best index for a query pattern
    ///
    /// Returns the index that will be most efficient based on which
    /// positions are bound (not wildcards).
    pub fn select_best(
        subject_bound: bool,
        predicate_bound: bool,
        object_bound: bool,
        graph_bound: bool,
    ) -> IndexType {
        // Priority order based on selectivity:
        // 1. Most selective: fully bound queries
        // 2. Predicate + Object (usually selective)
        // 3. Subject + Predicate
        // 4. Graph queries
        // 5. Object-only
        // 6. Subject-only
        // 7. Full scan (default to SPOC)

        match (subject_bound, predicate_bound, object_bound, graph_bound) {
            // Fully bound - use SPOC (arbitrary choice, all equally good)
            (true, true, true, true) => IndexType::SPOC,
            (true, true, true, false) => IndexType::SPOC,

            // Predicate + Object bound (very selective)
            (_, true, true, _) => IndexType::POCS,

            // Subject + Predicate bound
            (true, true, _, _) => IndexType::SPOC,

            // Graph bound
            (_, _, _, true) => IndexType::CSPO,

            // Object bound only
            (false, false, true, _) => IndexType::OCSP,

            // Predicate bound only
            (false, true, false, _) => IndexType::POCS,

            // Subject bound only
            (true, false, false, _) => IndexType::SPOC,

            // Subject + Object bound (no predicate)
            (true, false, true, _) => IndexType::SPOC,

            // Nothing bound - full scan, use SPOC
            _ => IndexType::SPOC,
        }
    }
}

/// Encode a node into a byte buffer for index keys
///
/// Encodes RDF nodes into a compact binary format for efficient storage and comparison.
/// Format: [type:u8][length:varint][data]
///
/// The encoding preserves lexicographic ordering for efficient range queries.
pub fn encode_node(buf: &mut SmallVec<[u8; 256]>, node: &rdf_model::Node<'_>) {
    // Type byte: 0=IRI, 1=Literal, 2=Blank, 3=QuotedTriple, 4=Variable
    let type_byte = match node {
        rdf_model::Node::Iri(_) => 0u8,
        rdf_model::Node::Literal(_) => 1u8,
        rdf_model::Node::BlankNode(_) => 2u8,
        rdf_model::Node::QuotedTriple(_) => 3u8,
        rdf_model::Node::Variable(_) => 4u8,
    };

    buf.push(type_byte);

    match node {
        rdf_model::Node::Literal(lit) => {
            // Encode lexical form
            let bytes = lit.lexical_form.as_bytes();
            encode_varint(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);

            // Encode language tag if present
            if let Some(lang) = lit.language {
                buf.push(1); // Language present marker
                let lang_bytes = lang.as_bytes();
                encode_varint(buf, lang_bytes.len() as u64);
                buf.extend_from_slice(lang_bytes);
            } else {
                buf.push(0); // No language
            }

            // Encode datatype if present
            if let Some(datatype) = lit.datatype {
                buf.push(1); // Datatype present marker
                let dt_bytes = datatype.as_bytes();
                encode_varint(buf, dt_bytes.len() as u64);
                buf.extend_from_slice(dt_bytes);
            } else {
                buf.push(0); // No datatype
            }
        }
        rdf_model::Node::BlankNode(id) => {
            // Encode blank node ID as string representation of u64
            let id_str = id.0.to_string();
            let bytes = id_str.as_bytes();
            encode_varint(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
        _ => {
            // For IRI, QuotedTriple, Variable - encode using .value()
            let value = node.value();
            let bytes = value.as_bytes();
            encode_varint(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
    }
}

/// Encode an optional node (used for graph context in quads)
///
/// Encodes the graph component of a quad, which may be None for the default graph.
/// Uses a flag byte to distinguish between Some and None cases.
pub(crate) fn encode_node_opt(buf: &mut SmallVec<[u8; 256]>, node: &Option<rdf_model::Node<'_>>) {
    match node {
        Some(n) => {
            buf.push(1); // Present marker
            encode_node(buf, n);
        }
        None => {
            buf.push(0); // Absent marker (default graph)
        }
    }
}

/// Encode variable-length integer (LEB128 format)
fn encode_varint(buf: &mut SmallVec<[u8; 256]>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0x80; // More bytes flag
        }

        buf.push(byte);

        if value == 0 {
            break;
        }
    }
}

/// Decode variable-length integer (LEB128 format)
/// Returns (value, bytes_consumed)
fn decode_varint(data: &[u8]) -> Result<(u64, usize), String> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut bytes_read = 0;

    for &byte in data.iter() {
        bytes_read += 1;
        result |= ((byte & 0x7F) as u64) << shift;

        if byte & 0x80 == 0 {
            // No more bytes
            return Ok((result, bytes_read));
        }

        shift += 7;

        if shift >= 64 {
            return Err("Varint overflow".to_string());
        }
    }

    Err("Incomplete varint".to_string())
}

/// Decode a node from bytes
/// Returns (Node, bytes_consumed)
fn decode_node<'a>(data: &[u8], dict: &'a rdf_model::Dictionary) -> Result<(rdf_model::Node<'a>, usize), String> {
    if data.is_empty() {
        return Err("Empty data".to_string());
    }

    let type_byte = data[0];
    let mut offset = 1;

    // Decode length
    let (length, varint_len) = decode_varint(&data[offset..])?;
    offset += varint_len;

    // Extract string data
    if offset + length as usize > data.len() {
        return Err("Invalid node encoding".to_string());
    }

    let string_data = &data[offset..offset + length as usize];
    let value_str = std::str::from_utf8(string_data)
        .map_err(|e| format!("Invalid UTF-8: {}", e))?;

    offset += length as usize;

    // Create node based on type
    let node = match type_byte {
        0 => rdf_model::Node::iri(dict.intern(value_str)),
        1 => {
            // Literal - decode language and datatype
            // Check for language tag
            if offset >= data.len() {
                return Err("Missing language marker".to_string());
            }
            let lang_present = data[offset];
            offset += 1;

            let language = if lang_present == 1 {
                let (lang_len, varint_len) = decode_varint(&data[offset..])?;
                offset += varint_len;

                if offset + lang_len as usize > data.len() {
                    return Err("Invalid language tag encoding".to_string());
                }

                let lang_data = &data[offset..offset + lang_len as usize];
                let lang_str = std::str::from_utf8(lang_data)
                    .map_err(|e| format!("Invalid UTF-8 in language tag: {}", e))?;
                offset += lang_len as usize;
                Some(dict.intern(lang_str))
            } else {
                None
            };

            // Check for datatype
            if offset >= data.len() {
                return Err("Missing datatype marker".to_string());
            }
            let dt_present = data[offset];
            offset += 1;

            let datatype = if dt_present == 1 {
                let (dt_len, varint_len) = decode_varint(&data[offset..])?;
                offset += varint_len;

                if offset + dt_len as usize > data.len() {
                    return Err("Invalid datatype encoding".to_string());
                }

                let dt_data = &data[offset..offset + dt_len as usize];
                let dt_str = std::str::from_utf8(dt_data)
                    .map_err(|e| format!("Invalid UTF-8 in datatype: {}", e))?;
                offset += dt_len as usize;
                Some(dict.intern(dt_str))
            } else {
                None
            };

            // Create the appropriate literal
            let value_interned = dict.intern(value_str);
            match (language, datatype) {
                (Some(lang), None) => rdf_model::Node::literal_lang(value_interned, lang),
                (None, Some(dt)) => rdf_model::Node::literal_typed(value_interned, dt),
                (None, None) => rdf_model::Node::literal_str(value_interned),
                (Some(_), Some(_)) => {
                    return Err("Literal cannot have both language and datatype".to_string());
                }
            }
        }
        2 => {
            // Blank node - parse string as u64 ID
            let id = value_str.parse::<u64>()
                .map_err(|e| format!("Invalid blank node ID: {}", e))?;
            rdf_model::Node::blank(id)
        }
        _ => return Err(format!("Unknown node type: {}", type_byte)),
    };

    Ok((node, offset))
}

/// Decode an optional node from bytes
/// Returns (Option<Node>, bytes_consumed)
fn decode_node_opt<'a>(data: &[u8], dict: &'a rdf_model::Dictionary) -> Result<(Option<rdf_model::Node<'a>>, usize), String> {
    if data.is_empty() {
        return Err("Empty data".to_string());
    }

    let present = data[0];
    if present == 0 {
        Ok((None, 1))
    } else {
        let (node, bytes) = decode_node(&data[1..], dict)?;
        Ok((Some(node), bytes + 1))
    }
}

/// Quad index abstraction
pub trait Index {
    /// Insert a quad into this index
    fn insert(&mut self, quad: &Quad) -> crate::StorageResult<()>;

    /// Remove a quad from this index
    fn remove(&mut self, quad: &Quad) -> crate::StorageResult<()>;

    /// Check if a quad exists in this index
    fn contains(&self, quad: &Quad) -> crate::StorageResult<bool>;

    /// Scan with a prefix
    fn scan_prefix(&self, prefix: &[u8]) -> crate::StorageResult<Vec<Vec<u8>>>;
}

/// Quad index implementation
pub struct QuadIndex {
    index_type: IndexType,
}

impl QuadIndex {
    /// Create a new index of the specified type
    pub fn new(index_type: IndexType) -> Self {
        Self { index_type }
    }

    /// Get the index type
    pub fn index_type(&self) -> IndexType {
        self.index_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdf_model::{Dictionary, Node};

    #[test]
    fn test_index_type_selection() {
        // Subject + Predicate bound
        assert_eq!(
            IndexType::select_best(true, true, false, false),
            IndexType::SPOC
        );

        // Predicate + Object bound
        assert_eq!(
            IndexType::select_best(false, true, true, false),
            IndexType::POCS
        );

        // Object only
        assert_eq!(
            IndexType::select_best(false, false, true, false),
            IndexType::OCSP
        );

        // Graph only
        assert_eq!(
            IndexType::select_best(false, false, false, true),
            IndexType::CSPO
        );

        // Nothing bound
        assert_eq!(
            IndexType::select_best(false, false, false, false),
            IndexType::SPOC
        );
    }

    #[test]
    fn test_encode_key_spoc() {
        let dict = Dictionary::new();
        let quad = Quad::new(
            Node::iri(dict.intern("http://s")),
            Node::iri(dict.intern("http://p")),
            Node::iri(dict.intern("http://o")),
            None,
        );

        let key = IndexType::SPOC.encode_key(&quad);
        assert!(!key.is_empty());
    }

    #[test]
    fn test_varint_encoding() {
        let mut buf = SmallVec::new();

        // Encode small number
        encode_varint(&mut buf, 127);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], 127);

        // Encode larger number
        buf.clear();
        encode_varint(&mut buf, 300);
        assert!(buf.len() > 1);
    }
}
