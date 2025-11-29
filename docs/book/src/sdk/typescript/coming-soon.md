# TypeScript SDK Coming Soon

The TypeScript SDK brings rust-kgdb to the web, Node.js, and Electron ecosystems through WebAssembly.

## Development Timeline

| Phase | Timeline | Status |
|-------|----------|--------|
| WASM compilation | Q2 2025 | In progress |
| Core TypeScript bindings | Q2 2025 | Pending |
| React hooks | Q2-Q3 2025 | Pending |
| Node.js integration | Q3 2025 | Pending |
| Production release | Q3 2025 | Pending |

## What We're Building

### WebAssembly Core
Compiled Rust code running at native speed in browsers and Node.js.

### TypeScript First
Type-safe APIs with full TypeScript support and excellent IDE integration.

### Framework Integrations
- **React**: Custom hooks and components
- **Vue**: Composables
- **Angular**: Services and pipes
- **Svelte**: Stores and actions
- **Next.js**: API routes and middleware
- **Nuxt**: Modules and plugins

## Use Cases

### Browser-Based Knowledge Graph Explorer

```typescript
import React, { useState } from 'react';
import { useRDFStore, useSparqlQuery } from '@zenya/rust-kgdb/react';

export function GraphExplorer() {
    const store = useRDFStore();
    const [selectedEntity, setSelectedEntity] = useState<string>('');

    const { data: relations, isLoading } = useSparqlQuery(
        store,
        selectedEntity
            ? `SELECT ?predicate ?object WHERE { <${selectedEntity}> ?predicate ?object }`
            : null
    );

    return (
        <div className="graph-explorer">
            <input
                type="text"
                value={selectedEntity}
                onChange={(e) => setSelectedEntity(e.target.value)}
                placeholder="Enter entity URI"
            />
            {isLoading && <p>Loading...</p>}
            <div className="relations-list">
                {relations?.map((rel) => (
                    <div key={`${rel.predicate}-${rel.object}`}>
                        <strong>{rel.predicate}</strong>: {rel.object}
                    </div>
                ))}
            </div>
        </div>
    );
}
```

### Node.js REST API

```typescript
import express from 'express';
import { RDFStore, Dictionary } from '@zenya/rust-kgdb';

const app = express();
const dict = new Dictionary();
const store = await RDFStore.newRocksDB('./production-kg');

// Load ontology
await store.loadTurtle(await fs.promises.readFile('ontology.ttl', 'utf-8'));

app.get('/api/entity/:id', async (req, res) => {
    try {
        const query = `
            PREFIX ex: <http://example.org/>
            SELECT ?property ?value WHERE {
                <${decodeURIComponent(req.params.id)}> ?property ?value
            }
        `;

        const results = await store.sparqlQuery(query);
        res.json({
            entity: decodeURIComponent(req.params.id),
            properties: results,
        });
    } catch (error) {
        res.status(400).json({ error: error.message });
    }
});

app.post('/api/query', express.json(), async (req, res) => {
    try {
        const results = await store.sparqlQuery(req.body.query);
        res.json(results);
    } catch (error) {
        res.status(400).json({ error: error.message });
    }
});

app.listen(3000, () => console.log('Server running on port 3000'));
```

### Electron Desktop Application

```typescript
import { ipcRenderer } from 'electron';
import { RDFStore } from '@zenya/rust-kgdb';

// Main process
export async function setupStore() {
    const store = await RDFStore.newRocksDB('./app-data/kg');

    ipcMain.handle('load-data', async (event, filePath: string) => {
        const content = await fs.promises.readFile(filePath, 'utf-8');
        return store.loadTurtle(content);
    });

    ipcMain.handle('sparql-query', async (event, query: string) => {
        return store.sparqlQuery(query);
    });

    return store;
}

// Renderer process
async function executeQuery(query: string) {
    const results = await ipcRenderer.invoke('sparql-query', query);
    displayResults(results);
}
```

### Data Visualization with D3.js

```typescript
import * as d3 from 'd3';
import { RDFStore } from '@zenya/rust-kgdb';

async function visualizeGraph(store: RDFStore) {
    // Query entity relationships
    const query = `
        PREFIX ex: <http://example.org/>
        SELECT ?source ?target WHERE {
            ?source ex:relatedTo ?target
        }
        LIMIT 100
    `;

    const relationships = await store.sparqlQuery(query);

    // Build graph data
    const nodes = new Set<string>();
    const links: Array<{ source: string; target: string }> = [];

    relationships.forEach((rel) => {
        nodes.add(rel.source);
        nodes.add(rel.target);
        links.push({ source: rel.source, target: rel.target });
    });

    // D3 visualization
    const svg = d3.select('svg');
    const simulation = d3.forceSimulation(Array.from(nodes).map((id) => ({ id })))
        .force('link', d3.forceLink(links).id((d) => d.id))
        .force('charge', d3.forceManyBody())
        .force('center', d3.forceCenter());

    // ... rest of D3 code
}
```

## Key Features

### Performance
- WASM execution at near-native speed
- Efficient memory management (24 bytes per triple)
- Connection pooling for scalability

### Type Safety
- Full TypeScript definitions
- No `any` types in public API
- Excellent IDE autocomplete

### Async/Await
All operations are async-first:

```typescript
// Wait for initialization
const store = await RDFStore.newInMemory();

// Async queries
const results = await store.sparqlQuery(query);

// Stream results
for await (const binding of store.streamResults(query)) {
    process.results(binding);
}
```

### Web Worker Support
Non-blocking operations in browsers:

```typescript
// Main thread
const worker = new Worker('kg-worker.ts');

worker.postMessage({
    type: 'query',
    payload: 'SELECT * WHERE { ?s ?p ?o }'
});

worker.onmessage = (event) => {
    console.log('Results:', event.data);
};

// Worker thread (kg-worker.ts)
import { RDFStore } from '@zenya/rust-kgdb';

const store = await RDFStore.newInMemory();

self.onmessage = async (event) => {
    if (event.data.type === 'query') {
        const results = await store.sparqlQuery(event.data.payload);
        self.postMessage(results);
    }
};
```

## Testing Support

Built-in testing utilities:

```typescript
import { describe, it, expect } from 'vitest';
import { RDFStore } from '@zenya/rust-kgdb';

describe('Knowledge Graph', () => {
    let store: RDFStore;

    beforeEach(async () => {
        store = await RDFStore.newInMemory();
    });

    it('should store and retrieve triples', async () => {
        const query = `
            INSERT DATA {
                <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob>
            }
        `;

        await store.sparqlUpdate(query);

        const results = await store.sparqlQuery(
            'SELECT ?x WHERE { ?x <http://example.org/knows> ?y }'
        );

        expect(results).toHaveLength(1);
    });
});
```

## Size and Performance

**Bundle Sizes** (estimated):
- WASM binary: ~8-12 MB
- Tree-shakeable: Only include what you use
- Compression (gzip): ~2-3 MB

**Query Performance**:
- Simple lookup: ~2.78 µs
- Complex query: <100 µs
- Bulk insert: ~146K triples/sec

## Planned Starters

Quick start templates:
- `create-kg-app` - React template
- `kg-next-starter` - Next.js template
- `kg-vue-starter` - Vue template
- `kg-express-starter` - Express.js template

## FAQ

**Q: When will TypeScript SDK be available?**
A: Target Q3 2025. Core WASM by Q2.

**Q: What about bundle size?**
A: WASM is ~12 MB uncompressed, ~3 MB gzipped. We provide size optimization tools.

**Q: Can I use it offline?**
A: Yes, WASM works completely offline in browsers and Node.js.

**Q: Does it work with Next.js?**
A: Yes, with special configuration for WASM. Next.js starter included.

**Q: Is it free?**
A: Yes, open source under the same license as rust-kgdb.

**Q: What about SSR with Next.js?**
A: Planned for v1.1. Initial release is client-side and Node.js.

## Current Workarounds

Until TypeScript SDK is released:

1. **REST API**: Deploy rust-kgdb server, query via fetch
2. **Rust WASM**: Manually compile Rust to WASM using wasm-pack
3. **GraphQL endpoint**: Use SPARQL-to-GraphQL bridge

## Resources

- [Rust SDK Documentation](../rust/index.md) - Available now
- [Main Repository](https://github.com/zenya-graphdb/rust-kgdb)
- [Issue Tracker](https://github.com/zenya-graphdb/rust-kgdb/issues)

## Contributing

Shape the TypeScript SDK:

1. Star the [GitHub repository](https://github.com/zenya-graphdb/rust-kgdb)
2. Comment on TypeScript SDK [issues](https://github.com/zenya-graphdb/rust-kgdb/issues)
3. Submit design proposals

## Get Notified

1. Star the GitHub repository
2. Watch releases
3. Subscribe to announcements

We're excited to bring rust-kgdb's power to the web!
