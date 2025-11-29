# Python SDK (Coming Soon)

The Python SDK for rust-kgdb is currently in development and will provide native Python bindings for easy integration with Python applications.

## Planned Features

- Direct Python interface to rust-kgdb
- SPARQL query execution from Python
- RDF data loading and manipulation
- Semantic reasoning capabilities
- Jupyter notebook support
- Integration with popular Python ML libraries (scikit-learn, pandas, etc.)

## Status

**Current**: Pre-release development
**Target Release**: Q1 2025
**Development Branch**: `feat/python-bindings`

## Expected Installation

```bash
pip install rust-kgdb
```

## Planned Quick Example

```python
from rust_kgdb import RDFStore, Dictionary

# Create dictionary and store
dict = Dictionary()
store = RDFStore.new_inmemory()

# Add RDF data
subject = dict.intern("http://example.org/Alice")
predicate = dict.intern("http://example.org/knows")
object = dict.intern("http://example.org/Bob")

store.put(Triple(subject, predicate, object))

# Execute SPARQL query
results = store.sparql_query("""
    SELECT ?person WHERE {
        ?person <http://example.org/knows> ?friend
    }
""")
```

## Stay Updated

- Check the [main repository](https://github.com/zenya-graphdb/rust-kgdb)
- Star the project to get notifications
- Subscribe to releases on GitHub

## Questions?

See [Coming Soon](./coming-soon.md) for more details on the Python SDK timeline.

## Related Documentation

- [Rust SDK](../rust/index.md) - Currently available
- [Kotlin SDK](../kotlin/index.md) - Mobile development
- [TypeScript SDK](../typescript/index.md) - Web development
