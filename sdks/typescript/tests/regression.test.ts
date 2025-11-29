/**
 * Comprehensive Regression Test Suite for TypeScript SDK
 *
 * This test suite covers:
 * 1. Basic RDF triple operations (CRUD)
 * 2. SPARQL query functionality
 * 3. Node types (IRI, literal, typed literals, language tags)
 * 4. Hypergraph operations (binary, ternary, n-ary edges)
 * 5. Error handling and edge cases
 * 6. Performance with large datasets
 *
 * Matches Rust SDK test coverage for cross-language consistency.
 */

import { describe, test, expect, beforeEach } from '@jest/globals';
import { GraphDB, Node } from '../src';

describe('Basic CRUD Operations', () => {
    let db: GraphDB;

    beforeEach(() => {
        db = GraphDB.inMemory();
    });

    test('creates empty database', () => {
        expect(db.isEmpty()).toBe(true);
        expect(db.count()).toBe(0);
    });

    test('inserts single triple', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
                Node.iri('http://xmlns.com/foaf/0.1/Person')
            )
            .execute();

        expect(db.count()).toBe(1);
        expect(db.isEmpty()).toBe(false);
    });

    test('inserts multiple triples in single operation', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/name'),
                Node.literal('Alice')
            )
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/age'),
                Node.integer(30)
            )
            .execute();

        expect(db.count()).toBe(2);
    });

    test('queries all triples', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/test'),
                Node.iri('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
                Node.iri('http://example.org/TestClass')
            )
            .execute();

        const results = await db.query()
            .sparql('SELECT ?s ?p ?o WHERE { ?s ?p ?o }')
            .execute();

        expect(results.length).toBe(1);
        expect(results.isEmpty()).toBe(false);
    });

    test('clears all triples', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/test'),
                Node.iri('http://example.org/p'),
                Node.literal('value')
            )
            .execute();

        expect(db.count()).toBe(1);

        db.clear();

        expect(db.isEmpty()).toBe(true);
        expect(db.count()).toBe(0);
    });
});

describe('Node Types', () => {
    let db: GraphDB;

    beforeEach(() => {
        db = GraphDB.inMemory();
    });

    test('creates IRI node', async () => {
        const node = Node.iri('http://example.org/resource');

        await db.insert()
            .triple(
                node,
                Node.iri('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
                Node.iri('http://example.org/Class')
            )
            .execute();

        expect(db.count()).toBe(1);
    });

    test('creates plain literal', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/doc'),
                Node.iri('http://example.org/title'),
                Node.literal('Hello World')
            )
            .execute();

        expect(db.count()).toBe(1);
    });

    test('creates integer literal', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/person'),
                Node.iri('http://example.org/age'),
                Node.integer(42)
            )
            .execute();

        const results = await db.query()
            .sparql('SELECT ?age WHERE { <http://example.org/person> <http://example.org/age> ?age }')
            .execute();

        expect(results.length).toBe(1);
    });

    test('creates boolean literal', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/setting'),
                Node.iri('http://example.org/enabled'),
                Node.boolean(true)
            )
            .execute();

        expect(db.count()).toBe(1);
    });

    test('creates double literal', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/measurement'),
                Node.iri('http://example.org/value'),
                Node.double(3.14159)
            )
            .execute();

        expect(db.count()).toBe(1);
    });

    test('creates language-tagged literal', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/doc'),
                Node.iri('http://example.org/title'),
                Node.langLiteral('Hello', 'en')
            )
            .triple(
                Node.iri('http://example.org/doc'),
                Node.iri('http://example.org/title'),
                Node.langLiteral('Bonjour', 'fr')
            )
            .execute();

        expect(db.count()).toBe(2);
    });

    test('creates blank node', async () => {
        await db.insert()
            .triple(
                Node.blank('b1'),
                Node.iri('http://xmlns.com/foaf/0.1/name'),
                Node.literal('Anonymous')
            )
            .execute();

        expect(db.count()).toBe(1);
    });

    test('handles unicode literals', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/test'),
                Node.iri('http://example.org/label'),
                Node.literal('Hello 世界 🌍 مرحبا')
            )
            .execute();

        const results = await db.query()
            .sparql('SELECT ?label WHERE { <http://example.org/test> <http://example.org/label> ?label }')
            .execute();

        expect(results.length).toBe(1);
    });
});

describe('SPARQL Queries', () => {
    let db: GraphDB;

    // Helper function to populate test data
    async function populateDB(db: GraphDB): Promise<void> {
        // Alice
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
                Node.iri('http://xmlns.com/foaf/0.1/Person')
            )
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/name'),
                Node.literal('Alice')
            )
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/age'),
                Node.integer(30)
            )
            .execute();

        // Bob
        await db.insert()
            .triple(
                Node.iri('http://example.org/bob'),
                Node.iri('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
                Node.iri('http://xmlns.com/foaf/0.1/Person')
            )
            .triple(
                Node.iri('http://example.org/bob'),
                Node.iri('http://xmlns.com/foaf/0.1/name'),
                Node.literal('Bob')
            )
            .triple(
                Node.iri('http://example.org/bob'),
                Node.iri('http://xmlns.com/foaf/0.1/age'),
                Node.integer(25)
            )
            .execute();

        // Relationship
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/knows'),
                Node.iri('http://example.org/bob')
            )
            .execute();
    }

    beforeEach(async () => {
        db = GraphDB.inMemory();
        await populateDB(db);
    });

    test('selects all triples', async () => {
        const results = await db.query()
            .sparql('SELECT ?s ?p ?o WHERE { ?s ?p ?o }')
            .execute();

        expect(results.length).toBe(7); // 7 triples total
    });

    test('selects with pattern', async () => {
        const results = await db.query()
            .sparql(`
                SELECT ?person ?name WHERE {
                    ?person <http://xmlns.com/foaf/0.1/name> ?name
                }
            `)
            .execute();

        expect(results.length).toBe(2); // Alice and Bob
    });

    test('filters by type', async () => {
        const results = await db.query()
            .sparql(`
                SELECT ?person WHERE {
                    ?person a <http://xmlns.com/foaf/0.1/Person>
                }
            `)
            .execute();

        expect(results.length).toBe(2);
    });

    test('counts with aggregation', async () => {
        const results = await db.query()
            .sparql(`
                SELECT (COUNT(?s) as ?count) WHERE {
                    ?s a <http://xmlns.com/foaf/0.1/Person>
                }
            `)
            .execute();

        expect(results.length).toBe(1);
    });

    test('returns empty results on empty database', async () => {
        const emptyDB = GraphDB.inMemory();

        const results = await emptyDB.query()
            .sparql('SELECT ?s WHERE { ?s ?p ?o }')
            .execute();

        expect(results.isEmpty()).toBe(true);
        expect(results.length).toBe(0);
    });
});

describe('Hypergraph Operations', () => {
    let db: GraphDB;

    beforeEach(() => {
        db = GraphDB.inMemory();
    });

    test('creates binary hyperedge (2 nodes)', async () => {
        // Simple relationship between two entities
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://example.org/likes'),
                Node.iri('http://example.org/pizza')
            )
            .execute();

        const results = await db.query()
            .sparql('SELECT ?who ?what WHERE { ?who <http://example.org/likes> ?what }')
            .execute();

        expect(results.length).toBe(1);
    });

    test('creates ternary hyperedge (standard RDF triple)', async () => {
        // Subject-Predicate-Object pattern
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),           // Node 1: Subject
                Node.iri('http://xmlns.com/foaf/0.1/name'),   // Node 2: Predicate
                Node.literal('Alice')                           // Node 3: Object
            )
            .execute();

        expect(db.count()).toBe(1);
    });

    test('creates quaternary hyperedge (named graph)', async () => {
        // Subject-Predicate-Object-Graph pattern (RDF quad)
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/name'),
                Node.literal('Alice')
            )
            .graph(Node.iri('http://example.org/graph1'))
            .execute();

        expect(db.count()).toBe(1);
    });

    test('handles hyperedge with multiple objects', async () => {
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/name'),
                Node.literal('Alice')
            )
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/age'),
                Node.integer(30)
            )
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/email'),
                Node.literal('alice@example.org')
            )
            .execute();

        const results = await db.query()
            .sparql(`
                SELECT ?p ?o WHERE {
                    <http://example.org/alice> ?p ?o
                }
            `)
            .execute();

        expect(results.length).toBe(3);
    });

    test('traverses connected hyperedges', async () => {
        // Create path: Alice -> knows -> Bob -> knows -> Charlie
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/knows'),
                Node.iri('http://example.org/bob')
            )
            .triple(
                Node.iri('http://example.org/bob'),
                Node.iri('http://xmlns.com/foaf/0.1/knows'),
                Node.iri('http://example.org/charlie')
            )
            .execute();

        // Find friends of friends
        const results = await db.query()
            .sparql(`
                SELECT ?friend_of_friend WHERE {
                    <http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> ?friend .
                    ?friend <http://xmlns.com/foaf/0.1/knows> ?friend_of_friend
                }
            `)
            .execute();

        expect(results.length).toBe(1); // Charlie
    });

    test('matches complex hypergraph patterns', async () => {
        // Create small social network
        await db.insert()
            .triple(Node.iri('http://example.org/alice'), Node.iri('http://xmlns.com/foaf/0.1/name'), Node.literal('Alice'))
            .triple(Node.iri('http://example.org/alice'), Node.iri('http://xmlns.com/foaf/0.1/age'), Node.integer(30))
            .triple(Node.iri('http://example.org/bob'), Node.iri('http://xmlns.com/foaf/0.1/name'), Node.literal('Bob'))
            .triple(Node.iri('http://example.org/bob'), Node.iri('http://xmlns.com/foaf/0.1/age'), Node.integer(25))
            .triple(Node.iri('http://example.org/alice'), Node.iri('http://xmlns.com/foaf/0.1/knows'), Node.iri('http://example.org/bob'))
            .execute();

        // Find people who know someone
        const results = await db.query()
            .sparql(`
                SELECT ?person ?name ?knows WHERE {
                    ?person <http://xmlns.com/foaf/0.1/name> ?name .
                    ?person <http://xmlns.com/foaf/0.1/knows> ?knows
                }
            `)
            .execute();

        expect(results.length).toBe(1); // Alice knows Bob
    });
});

describe('Error Handling', () => {
    let db: GraphDB;

    beforeEach(() => {
        db = GraphDB.inMemory();
    });

    test('throws error on empty query', async () => {
        await expect(async () => {
            await db.query().execute();
        }).rejects.toThrow();
    });

    test('throws error on invalid SPARQL syntax', async () => {
        await expect(async () => {
            await db.query().sparql('INVALID SPARQL SYNTAX').execute();
        }).rejects.toThrow();
    });

    test('succeeds with valid query on empty database', async () => {
        const results = await db.query()
            .sparql('SELECT ?s WHERE { ?s ?p ?o }')
            .execute();

        expect(results.isEmpty()).toBe(true);
    });
});

describe('Performance Tests', () => {
    let db: GraphDB;

    beforeEach(() => {
        db = GraphDB.inMemory();
    });

    test('inserts 100 triples', async () => {
        for (let i = 0; i < 100; i++) {
            await db.insert()
                .triple(
                    Node.iri(`http://example.org/entity${i}`),
                    Node.iri('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
                    Node.iri('http://example.org/Entity')
                )
                .execute();
        }

        expect(db.count()).toBe(100);
    });

    test('queries 100 triples', async () => {
        // Insert data
        for (let i = 0; i < 100; i++) {
            await db.insert()
                .triple(
                    Node.iri(`http://example.org/entity${i}`),
                    Node.iri('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
                    Node.iri('http://example.org/Entity')
                )
                .execute();
        }

        // Query all
        const results = await db.query()
            .sparql('SELECT ?s WHERE { ?s a <http://example.org/Entity> }')
            .execute();

        expect(results.length).toBe(100);
    });

    test('bulk inserts 1000 triples in batches', async () => {
        // Insert in batches of 10
        for (let batch = 0; batch < 100; batch++) {
            const builder = db.insert();
            for (let i = 0; i < 10; i++) {
                const entityNum = batch * 10 + i;
                builder.triple(
                    Node.iri(`http://example.org/entity${entityNum}`),
                    Node.iri('http://example.org/index'),
                    Node.integer(entityNum)
                );
            }
            await builder.execute();
        }

        expect(db.count()).toBe(1000);
    });
});

describe('Binding Results', () => {
    let db: GraphDB;

    beforeEach(async () => {
        db = GraphDB.inMemory();
        await db.insert()
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/name'),
                Node.literal('Alice')
            )
            .triple(
                Node.iri('http://example.org/alice'),
                Node.iri('http://xmlns.com/foaf/0.1/age'),
                Node.integer(30)
            )
            .execute();
    });

    test('gets variable value from binding', async () => {
        const results = await db.query()
            .sparql('SELECT ?name WHERE { <http://example.org/alice> <http://xmlns.com/foaf/0.1/name> ?name }')
            .execute();

        expect(results.length).toBe(1);
        const binding = results[0];
        const name = binding.get('name');
        expect(name).toBeDefined();
        expect(name).toContain('Alice');
    });

    test('iterates over query results', async () => {
        const results = await db.query()
            .sparql('SELECT ?p ?o WHERE { <http://example.org/alice> ?p ?o }')
            .execute();

        let count = 0;
        for (const binding of results) {
            expect(binding.has('p')).toBe(true);
            expect(binding.has('o')).toBe(true);
            count++;
        }

        expect(count).toBe(2); // name and age
    });

    test('gets variable names from binding', async () => {
        const results = await db.query()
            .sparql('SELECT ?s ?p WHERE { ?s ?p ?o }')
            .execute();

        if (results.length > 0) {
            const binding = results[0];
            const variables = binding.variables;
            expect(variables).toContain('s');
            expect(variables).toContain('p');
        }
    });
});
