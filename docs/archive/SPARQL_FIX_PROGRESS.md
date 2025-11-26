# SPARQL Executor API Fixes - Progress Report

**Date**: 2025-11-17
**Status**: 🟡 In Progress (48 → 23 errors)

## Summary

Successfully fixed **25 out of 48 compilation errors** (52% reduction) by systematically updating the SPARQL executor to match the actual rdf-model API.

## Fixes Applied ✅

### 1. bindings.rs - Removed Ord/PartialOrd
**Issue**: Node doesn't implement Ord, so Binding<'a> can't derive it.
**Fix**: Removed `PartialOrd, Ord` from derive macro
**Line**: 16

### 2. Node Variant Names (executor.rs)
**Issue**: Executor used old variant names from previous API
**Fixes Applied**:
- `Node::IRI(` → `Node::Iri(` (~10 occurrences)
- `Node::Blank(` → `Node::BlankNode(` (~10 occurrences)

### 3. Quad Field Access (executor.rs)
**Issue**: Executor called methods but Quad fields are public
**Fixes Applied**:
- `quad.subject()` → `quad.subject`
- `quad.predicate()` → `quad.predicate`
- `quad.object()` → `quad.object`

### 4. Literal Destructuring (executor.rs)
**Issue**: Literal is now a tuple struct `Node::Literal(lit)` with fields:
- `lit.lexical_form: &'a str`
- `lit.language: Option<&'a str>`
- `lit.datatype: Option<&'a str>`

**Fixes Applied** (7 locations):

#### Line ~326 - Lang Function
```rust
// OLD:
Node::Literal { lang: Some(lang), .. } => {
    Some(Node::literal_str(self.dictionary.intern(lang)))
}

// NEW:
Node::Literal(lit) if lit.language.is_some() => {
    Some(Node::literal_str(lit.language.unwrap()))
}
```

#### Line ~336 - Datatype Function
```rust
// OLD:
Node::Literal { datatype: Some(dt), .. } => Some(Node::iri(dt))

// NEW:
Node::Literal(lit) if lit.datatype.is_some() => {
    Some(Node::iri(lit.datatype.unwrap()))
}
```

#### Line ~810 - to_string_node
```rust
// OLD:
Node::Literal { value, .. } => Node::literal_str(*value)

// NEW:
Node::Literal(lit) => Node::literal_str(lit.lexical_form)
```

#### Line ~822 - effective_boolean_value
```rust
// OLD:
Some(Node::Literal { value, datatype, .. }) => {
    if let Some(dt) = datatype {
        let dt_str = self.dictionary.resolve(*dt);
        ...
    }
    !self.dictionary.resolve(*value).unwrap_or("").is_empty()
}

// NEW:
Some(Node::Literal(lit)) => {
    if let Some(dt) = lit.datatype {
        if dt == "http://www.w3.org/2001/XMLSchema#boolean" {
            return lit.lexical_form == "true" || lit.lexical_form == "1";
        }
    }
    !lit.lexical_form.is_empty()
}
```

#### Line ~1020 - get_numeric_value
```rust
// OLD:
Node::Literal { value, .. } => {
    self.dictionary.resolve(*value)?.parse::<f64>().ok()
}

// NEW:
Node::Literal(lit) => lit.lexical_form.parse::<f64>().ok()
```

#### Line ~1029 - get_string_value
```rust
// OLD:
Node::Literal { value, .. } => self.dictionary.resolve(*value)
Node::IRI(iri) => self.dictionary.resolve(*iri)

// NEW:
Node::Literal(lit) => Some(lit.lexical_form)
Node::Iri(iri) => Some(iri.as_str())
```

### 5. Added compare_nodes() Helper (executor.rs)
**Issue**: Node doesn't implement Ord, breaking all sorting operations
**Fix**: Created custom comparison function (line ~1037)

```rust
fn compare_nodes(&self, a: &Node<'a>, b: &Node<'a>) -> std::cmp::Ordering {
    match (a, b) {
        (Node::Iri(a), Node::Iri(b)) => a.as_str().cmp(b.as_str()),
        (Node::Literal(a), Node::Literal(b)) => a.lexical_form.cmp(b.lexical_form),
        (Node::BlankNode(a), Node::BlankNode(b)) => a.cmp(b),
        (Node::Iri(_), _) => Ordering::Less,
        (_, Node::Iri(_)) => Ordering::Greater,
        (Node::Literal(_), Node::BlankNode(_)) => Ordering::Less,
        (Node::BlankNode(_), Node::Literal(_)) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}
```

### 6. Fixed All Sorting Operations (executor.rs)
**Issue**: `.sort()` and `.sort_by()` failed because Node doesn't implement Ord
**Fixes Applied** (5 locations):

#### Line ~1116 - COUNT DISTINCT
```rust
// OLD: values.sort();
// NEW:
values.sort_by(|a, b| self.compare_nodes(a, b));
```

#### Line ~1136, ~1146 - SUM/AVG DISTINCT (tuples)
```rust
// OLD: values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
// NEW:
values.sort_by(|a, b| self.compare_nodes(&a.1, &b.1));
```

#### Line ~1233 - GROUP_CONCAT DISTINCT
```rust
// OLD: values.sort_by(|a, b| a.1.cmp(&b.1));
// NEW:
values.sort_by(|a, b| self.compare_nodes(&a.1, &b.1));
```

#### Line ~1389 - ORDER BY
```rust
// OLD: _ => av.cmp(&bv)
// NEW:
_ => self.compare_nodes(&av, &bv)
```

### 7. Added Missing Pattern Matches (executor.rs)
**Issue**: Non-exhaustive patterns for Node enum
**Fix**: Added QuotedTriple and Variable variants to to_string_node

```rust
Node::QuotedTriple(_) => Node::literal_str("<<quoted-triple>>"),
Node::Variable(_) => Node::literal_str("?var"),
```

## Remaining Errors (23 total)

### Error Categories

1. **Binding Ord Trait** (1 error)
   - `error[E0277]: the trait bound bindings::Binding<'a>: Ord is not satisfied`
   - Somewhere in bindings.rs or executor.rs is still trying to sort/order Bindings
   - Need to find where and apply custom comparison

2. **Lifetime Errors** (E0621) (~7 errors)
   - `explicit lifetime required in the type of expr/input`
   - Visitor pattern lifetime mismatches
   - May need to adjust visitor trait signatures

3. **Borrow Checker Errors** (E0596) (~3 errors)
   - `cannot borrow *self as mutable, as it is behind a & reference`
   - Visitor methods take `&mut self` but are called from `&self` context
   - May need to restructure visitor pattern

4. **Lifetime Extension Errors** (E0597) (~1 error)
   - `quad_pattern does not live long enough`
   - Temporary values don't live long enough for the lifetime constraints

5. **Additional Lifetime Errors** (~11 errors)
   - Various "lifetime may not live long enough" errors
   - Related to visitor pattern and reference borrowing

## Architecture Issues

The remaining errors suggest **fundamental architectural challenges** with the current visitor pattern implementation:

1. **Immutable Visitor with Mutable State**: The AlgebraVisitor trait may need `&self` instead of `&mut self` if execution doesn't modify internal state, OR the Executor needs to use interior mutability (RefCell)

2. **Lifetime Constraints**: The visitor pattern's lifetime requirements (`'a`) are incompatible with how executor methods borrow from algebra structures

3. **Possible Solutions**:
   - Change Executor to not use visitor pattern (directly match on Algebra enum)
   - Use interior mutability (Cell/RefCell) for parts needing mutation
   - Restructure to pass owned values instead of references
   - Split execution into smaller lifetime scopes

## Next Steps

### Option A: Fix Remaining Errors (Complex)
1. Investigate Binding Ord usage and add custom comparison
2. Refactor visitor pattern to resolve lifetime issues
3. Consider interior mutability or ownership changes
4. ~4-6 hours of architectural work

### Option B: Alternative Approach (Simpler)
1. Remove visitor pattern from executor
2. Use direct pattern matching on Algebra enum
3. Simpler lifetime management
4. ~2-3 hours of refactoring

### Option C: Partial Fix (Pragmatic)
1. Fix non-lifetime errors first (Binding Ord)
2. Document lifetime issues as "Known Limitations"
3. Implement workarounds for critical paths
4. ~1-2 hours for partial functionality

## Test Status

**Cannot run tests** until compilation errors are resolved.

Once compiling:
- Run: `cargo test --package sparql`
- Expected: 27 tests from original implementation
- Need to verify SELECT and ASK queries work correctly

## Files Modified

1. `crates/sparql/src/bindings.rs` (1 change)
2. `crates/sparql/src/executor.rs` (25+ changes)
   - Node variant updates
   - Quad field access
   - Literal destructuring (7 locations)
   - Sorting operations (5 locations)
   - Helper function addition
   - Pattern match completion

## Progress Metrics

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Compilation Errors | 48 | 23 | ✅ -25 (-52%) |
| API Mismatches Fixed | 0 | 25 | ✅ +25 |
| Tests Passing | N/A | N/A | ⏳ Blocked |

## Conclusion

**Major progress** made on API compatibility fixes. All straightforward mismatches resolved. Remaining errors are **architectural** and require deeper refactoring of the visitor pattern or alternative approach to execution.

**Recommendation**: Option B (Remove visitor pattern) for fastest path to working code, or Option A if preserving visitor pattern design is critical.

---

**Last Updated**: 2025-11-17
**Next Reviewer**: Should decide on Option A vs Option B approach
