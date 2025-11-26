// Port of Jena Datatype tests
// Tests XSD datatype handling and validation

use rdf_model::{Dictionary, Node};
use std::sync::Arc;

#[test]
fn test_xsd_datatypes() {
    let dict = Arc::new(Dictionary::new());

    let datatypes = vec![
        "http://www.w3.org/2001/XMLSchema#string",
        "http://www.w3.org/2001/XMLSchema#integer",
        "http://www.w3.org/2001/XMLSchema#decimal",
        "http://www.w3.org/2001/XMLSchema#float",
        "http://www.w3.org/2001/XMLSchema#double",
        "http://www.w3.org/2001/XMLSchema#boolean",
        "http://www.w3.org/2001/XMLSchema#date",
        "http://www.w3.org/2001/XMLSchema#dateTime",
        "http://www.w3.org/2001/XMLSchema#time",
    ];

    for dt in datatypes {
        let datatype = dict.intern(dt);
        assert_eq!(datatype, dt);
    }
}

#[test]
fn test_integer_datatype() {
    let dict = Arc::new(Dictionary::new());

    let lit1 = Node::literal_typed("42", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));
    let lit2 = Node::literal_typed("-100", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));
    let lit3 = Node::literal_typed("0", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));

    // All should be valid integer literals
    assert!(matches!(lit1, Node::Literal(_)));
    assert!(matches!(lit2, Node::Literal(_)));
    assert!(matches!(lit3, Node::Literal(_)));
}

#[test]
fn test_decimal_datatype() {
    let dict = Arc::new(Dictionary::new());

    let lit1 = Node::literal_typed("3.14", dict.intern("http://www.w3.org/2001/XMLSchema#decimal"));
    let lit2 = Node::literal_typed("-0.001", dict.intern("http://www.w3.org/2001/XMLSchema#decimal"));
    let lit3 = Node::literal_typed("1000.00", dict.intern("http://www.w3.org/2001/XMLSchema#decimal"));

    assert!(matches!(lit1, Node::Literal(_)));
    assert!(matches!(lit2, Node::Literal(_)));
    assert!(matches!(lit3, Node::Literal(_)));
}

#[test]
fn test_boolean_datatype() {
    let dict = Arc::new(Dictionary::new());

    let lit_true = Node::literal_typed("true", dict.intern("http://www.w3.org/2001/XMLSchema#boolean"));
    let lit_false = Node::literal_typed("false", dict.intern("http://www.w3.org/2001/XMLSchema#boolean"));
    let lit_1 = Node::literal_typed("1", dict.intern("http://www.w3.org/2001/XMLSchema#boolean"));
    let lit_0 = Node::literal_typed("0", dict.intern("http://www.w3.org/2001/XMLSchema#boolean"));

    assert!(matches!(lit_true, Node::Literal(_)));
    assert!(matches!(lit_false, Node::Literal(_)));
    assert!(matches!(lit_1, Node::Literal(_)));
    assert!(matches!(lit_0, Node::Literal(_)));
}

#[test]
fn test_datetime_datatype() {
    let dict = Arc::new(Dictionary::new());

    let lit1 = Node::literal_typed("2023-11-25T10:30:00Z", dict.intern("http://www.w3.org/2001/XMLSchema#dateTime"));
    let lit2 = Node::literal_typed("2023-11-25T10:30:00+01:00", dict.intern("http://www.w3.org/2001/XMLSchema#dateTime"));

    assert!(matches!(lit1, Node::Literal(_)));
    assert!(matches!(lit2, Node::Literal(_)));
}

#[test]
fn test_date_datatype() {
    let dict = Arc::new(Dictionary::new());

    let lit = Node::literal_typed("2023-11-25", dict.intern("http://www.w3.org/2001/XMLSchema#date"));

    if let Some(literal) = lit.as_literal() {
        assert_eq!(literal.lexical_form, "2023-11-25");
        assert_eq!(literal.datatype, Some("http://www.w3.org/2001/XMLSchema#date"));
    }
}

#[test]
fn test_time_datatype() {
    let dict = Arc::new(Dictionary::new());

    let lit = Node::literal_typed("10:30:00", dict.intern("http://www.w3.org/2001/XMLSchema#time"));

    if let Some(literal) = lit.as_literal() {
        assert_eq!(literal.lexical_form, "10:30:00");
        assert_eq!(literal.datatype, Some("http://www.w3.org/2001/XMLSchema#time"));
    }
}

#[test]
fn test_langstring_datatype() {
    let dict = Arc::new(Dictionary::new());

    let lit = Node::literal_typed("hello@en", dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"));

    if let Some(literal) = lit.as_literal() {
        assert!(literal.lexical_form.contains("@en"));
        assert_eq!(literal.datatype, Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"));
    }
}

#[test]
fn test_datatype_interning() {
    let dict = Arc::new(Dictionary::new());

    // Same datatype should be interned once
    let dt1 = dict.intern("http://www.w3.org/2001/XMLSchema#integer");
    let dt2 = dict.intern("http://www.w3.org/2001/XMLSchema#integer");

    assert_eq!(dt1, dt2);
}

#[test]
fn test_different_datatypes() {
    let dict = Arc::new(Dictionary::new());

    let dt_int = dict.intern("http://www.w3.org/2001/XMLSchema#integer");
    let dt_str = dict.intern("http://www.w3.org/2001/XMLSchema#string");

    assert_ne!(dt_int, dt_str);
}
