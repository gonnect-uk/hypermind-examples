# W3C Turtle Parser Validation Analysis

**Date**: 2025-11-26
**Current Status**: 38/65 tests passing (58%)
**Target**: 65/65 tests passing (100%)

---

## Root Cause

The nom parser is **too permissive** - it accepts invalid syntax that should be rejected. All 27 failing tests are **negative tests** that should fail but are passing.

---

## Missing Validation Rules (Categorized)

### Category 1: Quoted Triple Constraints (7 tests)

**Tests**: turtle12-syntax-bad-01 through turtle12-syntax-bad-07

**Violations**:

1. **turtle12-syntax-bad-01.ttl**: `:x <<:s :p :o>> 123`
   - ❌ Quoted triple as **predicate** (only subject/object allowed)

2. **turtle12-syntax-bad-02.ttl**: `<<3 :p :o>>`
   - ❌ **Literal** as subject of quoted triple (only IRI/BlankNode allowed)

3. **turtle12-syntax-bad-03.ttl**: `<<:s [] :o>>`
   - ❌ **Blank node property list** as predicate (only IRI allowed)

4. **turtle12-syntax-bad-04.ttl**: `:s :p << :p :r >>`
   - ❌ Missing subject in quoted triple (only 2 items instead of 3)

5. **turtle12-syntax-bad-05.ttl**: `:s :p << :g :s :p :o >>`
   - ❌ Too many items in quoted triple (4 items instead of 3)

6. **turtle12-syntax-bad-06.ttl**: `<<:s :p ("abc")>>`
   - ❌ **Collection** as object of quoted triple (not allowed)

7. **turtle12-syntax-bad-07.ttl**: `<<:s :p [ :p1 :o1 ]>>`
   - ❌ **Blank node property list** as object (not allowed)

**W3C Grammar Rules**:
```ebnf
[16] reifiedTriple  ::= '<<' rtSubject verb rtObject reifier? '>>'
[17] rtSubject      ::= iri | BlankNode | reifiedTriple
[18] rtObject       ::= iri | BlankNode | literal | tripleTerm | reifiedTriple
[12] verb           ::= predicate | 'a'
[14] predicate      ::= iri
```

**Required Validations**:
- ✅ Subject: IRI | BlankNode | QuotedTriple
- ✅ Predicate: **ONLY IRI** (no blank nodes, literals, collections, etc.)
- ✅ Object: IRI | BlankNode | Literal | TripleTerm | QuotedTriple
- ✅ Must have **exactly 3 components** (not 2, not 4)
- ❌ Collections NOT allowed in quoted triples
- ❌ Blank node property lists NOT allowed in quoted triples

---

### Category 2: VERSION Directive Constraints (6 tests)

**Tests**: turtle12-version-bad-01 through turtle12-version-bad-06

**Violations**:

1. **turtle12-version-bad-01.ttl**: `VERSION 1.2`
   - ❌ Missing quotes around version string

2. **turtle12-version-bad-02.ttl**: `VERSION """1.2"""`
   - ❌ Long string literal not allowed (only short quotes)

3. **turtle12-version-bad-03.ttl**: `VERSION '''1.2'''`
   - ❌ Long single-quote literal not allowed

4. **turtle12-version-bad-04.ttl**: `@version 1.2 .`
   - ❌ Missing quotes around version string

5. **turtle12-version-bad-05.ttl**: `@version """1.2""" .`
   - ❌ Long quote not allowed

6. **turtle12-version-bad-06.ttl**: `@version '''1.2''' .`
   - ❌ Long single-quote not allowed

**W3C Grammar Rule**:
```ebnf
[4] version ::= 'VERSION' STRING_LITERAL_QUOTE
              | 'VERSION' STRING_LITERAL_SINGLE_QUOTE
              | '@version' STRING_LITERAL_QUOTE '.'
              | '@version' STRING_LITERAL_SINGLE_QUOTE '.'
```

**Required Validations**:
- ✅ VERSION must use STRING_LITERAL_QUOTE (`"..."`) or STRING_LITERAL_SINGLE_QUOTE (`'...'`)
- ❌ NOT allowed: unquoted literals
- ❌ NOT allowed: long string literals (`"""..."""` or `'''...'''`)

---

### Category 3: Annotation Syntax (2 tests)

**Tests**: turtle12-syntax-bad-ann-1, turtle12-syntax-bad-ann-2

**Violations**:

1. **turtle12-syntax-bad-ann-1.ttl**: Contains `{| |}`
   - ❌ SPARQL-star annotation syntax NOT valid in Turtle

2. **turtle12-syntax-bad-ann-2.ttl**: `:a :b :c {| :s :p :o |}`
   - ❌ SPARQL-star `{| ... |}` syntax NOT valid in Turtle

**Required Validation**:
- ❌ Reject `{|` and `|}` tokens (SPARQL-star only, not Turtle)
- ✅ Only allow `<< ... >>` for quoted triples

---

### Category 4: Language Direction Tags (2 tests)

**Tests**: nt-ttl12-langdir-bad-1, nt-ttl12-langdir-bad-2

**Violations**:

1. **nt-ttl12-langdir-bad-1.ttl**: `"Hello"@en--unk`
   - ❌ Invalid direction tag `--unk` (only `--ltr` or `--rtl` allowed)

2. **nt-ttl12-langdir-bad-2.ttl**: `"Hello"@en--LTR`
   - ❌ Uppercase `--LTR` not allowed (must be lowercase `--ltr`)

**W3C Grammar Rule**:
```ebnf
[37] LANGTAG ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)* ('--' ('ltr'|'rtl'))?
```

**Required Validations**:
- ✅ Language direction must be exactly `--ltr` or `--rtl` (lowercase)
- ❌ NOT allowed: arbitrary values like `--unk`
- ❌ NOT allowed: uppercase variants like `--LTR`, `--RTL`

---

### Category 5: Triple Terms (10 tests)

**Tests**: nt-ttl12-bad-syntax-01 through nt-ttl12-bad-syntax-10, nt-ttl12-bad-10

**Violations**:

1. **nt-ttl12-bad-syntax-01.ttl**: `<a> <<( <s> <p> <o> )>> <z>`
   - ❌ Triple term as **predicate** (only object allowed in Turtle)

2. **nt-ttl12-bad-syntax-02.ttl**: `<<( "XYZ" <p> <o> )>>`
   - ❌ **Literal** as subject of triple term (only IRI/BlankNode allowed)

3. **nt-ttl12-bad-syntax-03.ttl**: `<<( <s> "XYZ" <o> )>>`
   - ❌ **Literal** as predicate of triple term (only IRI allowed)

4. **nt-ttl12-bad-syntax-04.ttl**: `<< <s> _:label <o> >>`
   - ❌ **Blank node** as predicate of quoted triple (only IRI allowed)

5. **nt-ttl12-bad-syntax-05.ttl**: `<<( <s> <p> <o> )>> <a> <z>`
   - ❌ Triple term as **subject** (not allowed in standard Turtle)

6. **nt-ttl12-bad-syntax-06.ttl**: `<a> <<( <s> <p> <o> )>> <z>`
   - ❌ Triple term as **predicate** (not allowed)

7-10. **Similar violations**: Triple terms in invalid positions

**W3C Grammar Rules**:
```ebnf
[20] tripleTerm ::= '<<(' ttSubject verb ttObject ')>>'
[21] ttSubject  ::= iri | BlankNode
[22] ttObject   ::= iri | BlankNode | literal | tripleTerm
```

**Note**: In Turtle (vs N-Triples-star), triple terms can ONLY appear in **object position**, NOT as subject or predicate.

**Required Validations**:
- ✅ Triple term subject: **ONLY** IRI | BlankNode (NOT literal)
- ✅ Triple term predicate: **ONLY** IRI (NOT blank node, literal)
- ✅ Triple term object: IRI | BlankNode | Literal | TripleTerm
- ❌ Triple terms **ONLY** in object position (NOT as subject/predicate in standard Turtle)

---

## Implementation Strategy

### Phase 1: Quoted Triple Validation
**Time**: 1 hour
**Files**: `crates/rdf-io/src/turtle.rs`

1. Add validation to `quoted_triple()` parser:
   - Reject literals/collections/blank property lists in subject
   - **Strict**: Only IRI in predicate position
   - Reject collections/blank property lists in object
   - Verify exactly 3 components

2. Reject quoted triples in predicate position at call sites

### Phase 2: VERSION Directive Validation
**Time**: 30 minutes
**Files**: `crates/rdf-io/src/turtle.rs`

1. Update `version_upper()` parser:
   - Only accept `string_literal_quote()` or `string_literal_single_quote()`
   - Reject long string literals

2. Update `version_at()` parser:
   - Same validation

### Phase 3: Annotation Syntax Rejection
**Time**: 15 minutes
**Files**: `crates/rdf-io/src/turtle.rs`

1. Add early check for `{|` tokens
2. Return parse error if found

### Phase 4: Language Direction Tag Validation
**Time**: 30 minutes
**Files**: `crates/rdf-io/src/turtle.rs`

1. Update `langtag()` parser:
   - Validate direction tag is exactly `--ltr` or `--rtl`
   - Reject uppercase variants

### Phase 5: Triple Term Validation
**Time**: 45 minutes
**Files**: `crates/rdf-io/src/turtle.rs`

1. Add `triple_term()` parser (currently missing)
2. Validate constraints:
   - Subject: IRI | BlankNode only
   - Predicate: IRI only
   - Object: IRI | BlankNode | Literal | TripleTerm
3. Ensure triple terms only appear in object position

### Phase 6: Testing and Refinement
**Time**: 30 minutes

1. Run W3C test suite after each phase
2. Verify pass rate improves
3. Final run to confirm 100% (65/65)

---

## Expected Outcome

**Current**: 38/65 (58%)
**After Phase 1**: ~50/65 (77%) - 7 tests fixed
**After Phase 2**: ~56/65 (86%) - 6 tests fixed
**After Phase 3**: ~58/65 (89%) - 2 tests fixed
**After Phase 4**: ~60/65 (92%) - 2 tests fixed
**After Phase 5**: **65/65 (100%)** - 10 tests fixed ✅

**Total Time**: ~3.5 hours (under original 4-6 hour estimate)

---

## Test Files Reference

### Quoted Triple Tests (7)
- turtle12-syntax-bad-01.ttl through turtle12-syntax-bad-07.ttl

### VERSION Tests (6)
- turtle12-version-bad-01.ttl through turtle12-version-bad-06.ttl

### Annotation Tests (2)
- turtle12-syntax-bad-ann-1.ttl
- turtle12-syntax-bad-ann-2.ttl

### Language Direction Tests (2)
- nt-ttl12-langdir-bad-1.ttl
- nt-ttl12-langdir-bad-2.ttl

### Triple Term Tests (10)
- nt-ttl12-bad-syntax-01.ttl through nt-ttl12-bad-syntax-09.ttl
- nt-ttl12-bad-10.ttl
