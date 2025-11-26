// Port of Jena TestLiteralImpl.java
// Tests Literal creation, datatypes, and conversion

use rdf_model::{Dictionary, Node};
use std::sync::Arc;

#[test]
fn test_create_string_literal() {
    let dict = Arc::new(Dictionary::new());

    let literal = Node::literal_typed("hello world", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    if let Node::Literal(lit) = literal {
        assert_eq!(lit.lexical_form, "hello world");
        assert_eq!(lit.datatype, Some("http://www.w3.org/2001/XMLSchema#string"));
    } else {
        panic!("Expected Literal node");
    }
}

#[test]
fn test_create_integer_literal() {
    let dict = Arc::new(Dictionary::new());

    let literal = Node::literal_typed("42", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));

    if let Node::Literal(lit) = literal {
        assert_eq!(lit.lexical_form, "42");
        assert_eq!(lit.datatype, Some("http://www.w3.org/2001/XMLSchema#integer"));

        // Parse as integer
        let int_value: i32 = lit.lexical_form.parse().expect("Should parse as integer");
        assert_eq!(int_value, 42);
    }
}

#[test]
fn test_create_boolean_literal() {
    let dict = Arc::new(Dictionary::new());

    let literal_true = Node::literal_typed("true", dict.intern("http://www.w3.org/2001/XMLSchema#boolean"));
    let literal_false = Node::literal_typed("false", dict.intern("http://www.w3.org/2001/XMLSchema#boolean"));

    if let Node::Literal(lit) = literal_true {
        let bool_value: bool = lit.lexical_form.parse().expect("Should parse as boolean");
        assert_eq!(bool_value, true);
    }

    if let Node::Literal(lit) = literal_false {
        let bool_value: bool = lit.lexical_form.parse().expect("Should parse as boolean");
        assert_eq!(bool_value, false);
    }
}

#[test]
fn test_create_double_literal() {
    let dict = Arc::new(Dictionary::new());

    let literal = Node::literal_typed("3.14159", dict.intern("http://www.w3.org/2001/XMLSchema#double"));

    if let Node::Literal(lit) = literal {
        let value = lit.lexical_form;
        let datatype = lit.datatype.unwrap();
        assert_eq!(datatype, "http://www.w3.org/2001/XMLSchema#double");

        let double_value: f64 = value.parse().expect("Should parse as double");
        assert!((double_value - 3.14159).abs() < 0.00001);
    }
}

#[test]
fn test_create_date_literal() {
    let dict = Arc::new(Dictionary::new());

    let literal = Node::literal_typed("2023-11-25", dict.intern("http://www.w3.org/2001/XMLSchema#date"));

    if let Node::Literal(lit) = literal {
        let value = lit.lexical_form;
        let datatype = lit.datatype.unwrap();
        assert_eq!(value, "2023-11-25");
        assert_eq!(datatype, "http://www.w3.org/2001/XMLSchema#date");
    }
}

#[test]
fn test_literal_datatype_equality() {
    let dict = Arc::new(Dictionary::new());

    let lit1 = Node::literal_typed("100", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));
    let lit2 = Node::literal_typed("100", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));

    // Same value and datatype should be equal
    if let (Node::Literal(lit1), Node::Literal(lit2)) = (&lit1, &lit2) {
        let v1 = lit1.lexical_form;
        let d1 = lit1.datatype.unwrap();
        let v2 = lit2.lexical_form;
        let d2 = lit2.datatype.unwrap();
        assert_eq!(v1, v2);
        assert_eq!(d1, d2);
    }
}

#[test]
fn test_literal_different_datatypes() {
    let dict = Arc::new(Dictionary::new());

    let int_lit = Node::literal_typed("100", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));
    let str_lit = Node::literal_typed("100", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    // Same value but different datatypes
    if let (Node::Literal(lit1), Node::Literal(lit2)) = (&int_lit, &str_lit) {
        let v1 = lit1.lexical_form;
        let d1 = lit1.datatype.unwrap();
        let v2 = lit2.lexical_form;
        let d2 = lit2.datatype.unwrap();
        assert_eq!(v1, v2); // Same lexical value
        assert_ne!(d1, d2); // Different datatypes
    }
}

#[test]
fn test_literal_language_tag() {
    let dict = Arc::new(Dictionary::new());

    // Language-tagged strings (using rdf:langString datatype)
    let en_lit = Node::literal_typed("hello@en", dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"));
    let fr_lit = Node::literal_typed("bonjour@fr", dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"));

    if let Node::Literal(lit) = en_lit {
        let value = lit.lexical_form;
        let datatype = lit.datatype.unwrap();
        assert!(value.contains("@en"));
        assert_eq!(datatype, "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString");
    }

    if let Node::Literal(lit) = fr_lit {
        let value = lit.lexical_form;
        assert!(value.contains("@fr"));
    }
}

#[test]
fn test_numeric_literal_parsing() {
    let dict = Arc::new(Dictionary::new());

    // Test various numeric types
    let int_lit = Node::literal_typed("42", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));
    let long_lit = Node::literal_typed("9223372036854775807", dict.intern("http://www.w3.org/2001/XMLSchema#long"));
    let float_lit = Node::literal_typed("3.14", dict.intern("http://www.w3.org/2001/XMLSchema#float"));

    // Verify parsing works
    if let Node::Literal(lit) = int_lit {
        let value = lit.lexical_form;
        let parsed: i32 = value.parse().expect("Should parse as i32");
        assert_eq!(parsed, 42);
    }

    if let Node::Literal(lit) = long_lit {
        let value = lit.lexical_form;
        let parsed: i64 = value.parse().expect("Should parse as i64");
        assert_eq!(parsed, 9223372036854775807);
    }

    if let Node::Literal(lit) = float_lit {
        let value = lit.lexical_form;
        let parsed: f32 = value.parse().expect("Should parse as f32");
        assert!((parsed - 3.14).abs() < 0.01);
    }
}

#[test]
fn test_literal_empty_string() {
    let dict = Arc::new(Dictionary::new());

    let literal = Node::literal_typed("", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    if let Node::Literal(lit) = literal {
        let value = lit.lexical_form;
        assert_eq!(value, "");
        assert_eq!(value.len(), 0);
    }
}

#[test]
fn test_literal_with_special_characters() {
    let dict = Arc::new(Dictionary::new());

    let literal = Node::literal_typed("Hello\nWorld\t!", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    if let Node::Literal(lit) = literal {
        let value = lit.lexical_form;
        assert!(value.contains('\n'));
        assert!(value.contains('\t'));
        assert_eq!(value, "Hello\nWorld\t!");
    }
}

#[test]
fn test_literal_unicode() {
    let dict = Arc::new(Dictionary::new());

    let literal = Node::literal_typed("Hello 世界 🌍", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    if let Node::Literal(lit) = literal {
        let value = lit.lexical_form;
        assert!(value.contains("世界"));
        assert!(value.contains("🌍"));
        assert_eq!(value, "Hello 世界 🌍");
    }
}

#[test]
fn test_literal_long_string() {
    let dict = Arc::new(Dictionary::new());

    let long_text = "a".repeat(10000);
    let literal = Node::literal_typed(&long_text, dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    if let Node::Literal(lit) = literal {
        let value = lit.lexical_form;
        assert_eq!(value.len(), 10000);
        assert!(value.chars().all(|c| c == 'a'));
    }
}
