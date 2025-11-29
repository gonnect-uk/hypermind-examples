# TypeScript SDK (Coming Soon)

The TypeScript SDK brings rust-kgdb's performance and capabilities to web, Node.js, and Electron applications.

## Planned Features

- Native WebAssembly (WASM) bindings
- Full SPARQL query execution
- RDF data manipulation
- Zero-copy data structures
- React hooks for UI integration
- Node.js support
- Browser support with Web Workers
- Electron desktop app support

## Status

**Current**: Pre-release planning
**Target Release**: Q2 2025
**Focus**: Web and Node.js ecosystems

## Planned Installation

```bash
npm install @zenya/rust-kgdb
# or
yarn add @zenya/rust-kgdb
```

## Planned Quick Example

```typescript
import { RDFStore, Dictionary } from '@zenya/rust-kgdb';

// Create store
const dict = new Dictionary();
const store = await RDFStore.newInMemory();

// Add RDF data
const subject = dict.intern('http://example.org/Alice');
const predicate = dict.intern('http://example.org/knows');
const object = dict.intern('http://example.org/Bob');

store.put({ subject, predicate, object });

// Execute SPARQL query
const results = await store.sparqlQuery(`
    SELECT ?person WHERE {
        ?person <http://example.org/knows> ?friend
    }
`);

results.forEach(binding => {
    console.log(binding);
});
```

## React Integration (Planned)

```typescript
import { useRDFStore, useSparqlQuery } from '@zenya/rust-kgdb/react';

function KnowledgeGraphViewer() {
    const store = useRDFStore();
    const [query, setQuery] = useState('');

    const { data: results, isLoading } = useSparqlQuery(store, query, {
        skip: !query,
    });

    return (
        <div>
            <input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Enter SPARQL query..."
            />
            {isLoading && <div>Loading...</div>}
            <ResultsTable results={results} />
        </div>
    );
}
```

## Node.js Server (Planned)

```typescript
import express from 'express';
import { RDFStore } from '@zenya/rust-kgdb';

const app = express();
const store = await RDFStore.newRocksDB('./data');

app.post('/query', express.text(), async (req, res) => {
    try {
        const results = await store.sparqlQuery(req.body);
        res.json(results);
    } catch (error) {
        res.status(400).json({ error: error.message });
    }
});

app.listen(3000);
```

## Electron Desktop App (Planned)

```typescript
import { app, BrowserWindow } from 'electron';
import { RDFStore } from '@zenya/rust-kgdb';

let mainWindow: BrowserWindow;
const store = await RDFStore.newRocksDB('./app-data');

app.on('ready', () => {
    mainWindow = new BrowserWindow({ webPreferences: { nodeIntegration: false } });
    mainWindow.loadFile('index.html');

    // IPC for safe store access
    ipcMain.handle('sparql-query', (event, query) => {
        return store.sparqlQuery(query);
    });
});
```

## Performance

Expected performance characteristics:
- Sub-millisecond SPARQL queries
- Efficient memory usage
- WASM optimizations for modern browsers
- Web Worker support for non-blocking operations

## Browser Compatibility

Planned support:
- **Chrome/Chromium**: 95+
- **Firefox**: 91+
- **Safari**: 15+
- **Edge**: 95+
- **Node.js**: 16+

## Stay Updated

- Star the [GitHub repository](https://github.com/zenya-graphdb/rust-kgdb)
- Watch for announcements
- Follow the [issue tracker](https://github.com/zenya-graphdb/rust-kgdb/issues)

## Questions?

See [Coming Soon Details](./coming-soon.md) for the full development roadmap.

## Related Documentation

- [Rust SDK](../rust/index.md) - Currently available
- [Python SDK](../python/index.md) - In development
- [Kotlin SDK](../kotlin/index.md) - Android development
