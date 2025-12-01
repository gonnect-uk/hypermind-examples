# Python SDK Coming Soon

The Python SDK is one of our highest priorities for expanding rust-kgdb's accessibility.

## Development Timeline

| Phase | Timeline | Status |
|-------|----------|--------|
| Initial implementation | Q1 2025 | Planning |
| Beta release | Q1-Q2 2025 | Pending |
| Production release | Q2 2025 | Pending |

## What We're Building

### Direct Python Bindings
Native Python bindings using PyO3 for seamless integration with Python applications.

### Features
- **SPARQL Execution**: Run SPARQL queries directly from Python
- **RDF Loading**: Parse TTL, N-Triples, and other RDF formats
- **Data Manipulation**: Add, update, and delete RDF data
- **Semantic Reasoning**: RDFS and OWL 2 RL reasoning
- **Jupyter Support**: Interactive notebook experimentation
- **Pandas Integration**: Convert results to pandas DataFrames

### Performance
Expected to maintain rust-kgdb's performance characteristics:
- Sub-millisecond queries
- Memory-efficient storage
- Batch processing capabilities

## Use Cases

### Data Science
```python
import pandas as pd
from rust_kgdb import RDFStore

store = RDFStore.new_inmemory()
# Load knowledge graph
store.load("data.ttl")

# Execute SPARQL query
results = store.sparql_query("SELECT ?x ?y WHERE { ... }")

# Convert to DataFrame
df = pd.DataFrame(results)
df.to_csv("results.csv")
```

### Knowledge Graph Integration
```python
from rust_kgdb import RDFStore, RDFS

store = RDFStore.new_rocksdb("./mykg")
store.load_file("ontology.ttl")

# Apply semantic reasoning
reasoner = RDFS(store)
reasoner.apply_rules()

# Query inferred data
results = store.sparql_query("SELECT * WHERE { ... }")
```

### Machine Learning
```python
from rust_kgdb import RDFStore, Embeddings

store = RDFStore.new_inmemory()
embeddings = Embeddings(store)

# Generate embeddings
vector = embeddings.embed("http://example.org/Alice")

# Use with scikit-learn
from sklearn.cluster import KMeans
kmeans = KMeans(n_clusters=5)
kmeans.fit(embeddings.all_vectors())
```

## Current Workarounds

Until the Python SDK is released, you can:

1. **Use the Rust SDK**: Call Rust code from Python via FFI
2. **HTTP API**: Deploy rust-kgdb server and query via REST API
3. **Command-line tools**: Use CLI utilities for batch operations

## Contributing

We welcome contributions to the Python SDK development. If you're interested in helping:

1. Check the [GitHub repository](https://github.com/gonnect-uk/rust-kgdb)
2. Look for `python-bindings` issues
3. Submit PRs with implementations

## FAQ

**Q: When will Python SDK be available?**
A: Target Q2 2025. We'll announce on GitHub releases and the main website.

**Q: Will it be free/open source?**
A: Yes, all rust-kgdb SDKs are open source under the same license as the main project.

**Q: Can I use rust-kgdb with Python now?**
A: Yes, via the Rust SDK with FFI bindings, or by deploying an HTTP server.

**Q: What Python versions will be supported?**
A: Python 3.8+

**Q: Will it support async/await?**
A: Yes, async operations are planned.

## Resources

- [Rust SDK Documentation](../rust/index.md)
- [Main Repository](https://github.com/gonnect-uk/rust-kgdb)
- [Issue Tracker](https://github.com/gonnect-uk/rust-kgdb/issues)

## Get Notified

To be notified when the Python SDK is ready:

1. Star the [GitHub repository](https://github.com/gonnect-uk/rust-kgdb)
2. Watch for releases in the "Releases" tab
3. Subscribe to project announcements

Thank you for your interest in rust-kgdb!
