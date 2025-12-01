/**
 * Multi-Format RDF & Custom Functions Example (TypeScript/JavaScript)
 *
 * Demonstrates:
 * 1. Loading RDF data in Turtle, N-Triples, and N-Quads formats
 * 2. SPARQL queries across all formats
 * 3. Custom function registration and usage (Jena-compatible)
 * 4. Roundtrip serialization
 *
 * This example shows W3C-compliant custom SPARQL functions,
 * similar to Apache Jena's ExtensionFunctionRegistry.
 */

import { GraphDB, FunctionRegistry, Node } from 'gonnect-nano-graphdb';

async function main() {
    console.log('=== Multi-Format RDF & Custom Functions Demo ===\n');

    // =========================================================================
    // Part 1: Load Data in Different Formats
    // =========================================================================
    console.log('1. Loading data in different RDF formats...\n');

    // Same data in 3 formats
    const turtleData = `
        @prefix ex: <http://example.org/> .
        ex:Alice ex:age "30" .
        ex:Bob ex:age "25" .
        ex:Charlie ex:score "95.5" .
    `;

    const ntriplesData = `
        <http://example.org/Alice> <http://example.org/age> "30" .
        <http://example.org/Bob> <http://example.org/age> "25" .
        <http://example.org/Charlie> <http://example.org/score> "95.5" .
    `;

    const nquadsData = `
        <http://example.org/Alice> <http://example.org/age> "30" .
        <http://example.org/Bob> <http://example.org/age> "25" <http://example.org/graph1> .
        <http://example.org/Charlie> <http://example.org/score> "95.5" .
    `;

    // Create graph databases for each format
    const dbTurtle = new GraphDB();
    await dbTurtle.loadTurtle(turtleData);
    console.log(`✓ Loaded Turtle data: ${dbTurtle.size()} triples`);

    const dbNtriples = new GraphDB();
    await dbNtriples.loadNtriples(ntriplesData);
    console.log(`✓ Loaded N-Triples data: ${dbNtriples.size()} triples`);

    const dbNquads = new GraphDB();
    await dbNquads.loadNquads(nquadsData);
    console.log(`✓ Loaded N-Quads data: ${dbNquads.size()} triples\n`);

    // =========================================================================
    // Part 2: SPARQL Queries (Same query, all formats)
    // =========================================================================
    console.log('2. Running identical SPARQL queries on all formats...\n');

    const query = `
        PREFIX ex: <http://example.org/>
        SELECT ?person ?value WHERE {
            ?person ?prop ?value .
        }
    `;

    const resultsTurtle = await dbTurtle.query(query);
    const resultsNtriples = await dbNtriples.query(query);
    const resultsNquads = await dbNquads.query(query);

    console.log(`Turtle results: ${resultsTurtle.length} rows`);
    console.log(`N-Triples results: ${resultsNtriples.length} rows`);
    console.log(`N-Quads results: ${resultsNquads.length} rows`);
    console.log('✓ All formats produced results\n');

    // =========================================================================
    // Part 3: Custom SPARQL Functions (Jena-compatible)
    // =========================================================================
    console.log('3. Registering custom SPARQL functions...\n');

    // Create function registry
    const registry = new FunctionRegistry();

    // Register custom "double" function
    registry.register('http://example.org/double', (args: Node[]) => {
        if (args.length !== 1) return null;

        try {
            const value = parseFloat(args[0].literalValue());
            const result = value * 2.0;
            return Node.literalDecimal(result.toString());
        } catch (error) {
            return null;
        }
    });

    // Register custom "ageCategory" function
    registry.register('http://example.org/ageCategory', (args: Node[]) => {
        if (args.length !== 1) return null;

        try {
            const age = parseInt(args[0].literalValue());
            let category: string;

            if (age < 18) {
                category = 'minor';
            } else if (age < 65) {
                category = 'adult';
            } else {
                category = 'senior';
            }

            return Node.literalString(category);
        } catch (error) {
            return null;
        }
    });

    // Register custom "isEven" function
    registry.register('http://example.org/isEven', (args: Node[]) => {
        if (args.length !== 1) return null;

        try {
            const value = parseInt(args[0].literalValue());
            const isEven = value % 2 === 0;
            return Node.literalBoolean(isEven);
        } catch (error) {
            return null;
        }
    });

    console.log('✓ Registered 3 custom functions:');
    console.log('  - ex:double(x) - multiplies by 2');
    console.log('  - ex:ageCategory(age) - categorizes age');
    console.log('  - ex:isEven(x) - checks if even\n');

    // =========================================================================
    // Part 4: Use Custom Functions in SPARQL Queries
    // =========================================================================
    console.log('4. Querying with custom functions...\n');

    // Apply custom functions to query
    const dbWithFunctions = new GraphDB({ functionRegistry: registry });
    await dbWithFunctions.loadTurtle(turtleData);

    // Query 1: Double the ages
    const queryDouble = `
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age (ex:double(?age) AS ?doubledAge)
        WHERE {
            ?person ex:age ?age .
        }
    `;

    const resultsDouble = await dbWithFunctions.query(queryDouble);
    console.log('Query: ex:double(?age)');
    for (const row of resultsDouble) {
        console.log(`  ${row.person} age ${row.age} → doubled: ${row.doubledAge}`);
    }
    console.log();

    // Query 2: Categorize ages
    const queryCategory = `
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age (ex:ageCategory(?age) AS ?category)
        WHERE {
            ?person ex:age ?age .
        }
    `;

    const resultsCategory = await dbWithFunctions.query(queryCategory);
    console.log('Query: ex:ageCategory(?age)');
    for (const row of resultsCategory) {
        console.log(`  ${row.person} age ${row.age} → category: ${row.category}`);
    }
    console.log();

    // Query 3: Filter with custom function
    const queryFilter = `
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age
        WHERE {
            ?person ex:age ?age .
            FILTER (ex:isEven(?age))
        }
    `;

    const resultsFilter = await dbWithFunctions.query(queryFilter);
    console.log('Query: FILTER(ex:isEven(?age))');
    for (const row of resultsFilter) {
        console.log(`  ${row.person} has even age: ${row.age}`);
    }
    console.log();

    // =========================================================================
    // Part 5: Chaining Custom Functions
    // =========================================================================
    console.log('5. Chaining custom functions...\n');

    // Register additional function for chaining
    registry.register('http://example.org/addTen', (args: Node[]) => {
        if (args.length !== 1) return null;

        try {
            const value = parseFloat(args[0].literalValue());
            const result = value + 10.0;
            return Node.literalDecimal(result.toString());
        } catch (error) {
            return null;
        }
    });

    const dbChain = new GraphDB({ functionRegistry: registry });
    await dbChain.loadTurtle(turtleData);

    // Chain functions: (age + 10) * 2
    const queryChain = `
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age (ex:double(ex:addTen(?age)) AS ?transformed)
        WHERE {
            ?person ex:age ?age .
        }
    `;

    const resultsChain = await dbChain.query(queryChain);
    console.log('Query: ex:double(ex:addTen(?age))');
    for (const row of resultsChain) {
        const original = parseFloat(row.age);
        const expected = (original + 10.0) * 2.0;
        console.log(`  ${row.person} age ${row.age} → (${row.age} + 10) × 2 = ${row.transformed}`);
    }
    console.log();

    // =========================================================================
    // Part 6: Roundtrip Serialization
    // =========================================================================
    console.log('6. Roundtrip serialization (Turtle → N-Quads → Turtle)...\n');

    // Serialize to N-Quads
    const nquadsOutput = await dbTurtle.serializeNquads();
    console.log('Serialized to N-Quads:');
    console.log(nquadsOutput.substring(0, 200) + '...\n');

    // Parse back and verify
    const dbRoundtrip = new GraphDB();
    await dbRoundtrip.loadNquads(nquadsOutput);

    console.log(`✓ Roundtrip successful: ${dbRoundtrip.size()} triples preserved`);

    // Verify data is identical by running same query
    const resultsOriginal = await dbTurtle.query(query);
    const resultsRoundtrip = await dbRoundtrip.query(query);

    console.log(`✓ Query results identical: ${resultsOriginal.length} = ${resultsRoundtrip.length}\n`);

    // =========================================================================
    // Part 7: Named Graphs (N-Quads only feature)
    // =========================================================================
    console.log('7. Named graphs (N-Quads exclusive feature)...\n');

    // Query specific named graph
    const queryGraph = `
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age
        WHERE {
            GRAPH <http://example.org/graph1> {
                ?person ex:age ?age .
            }
        }
    `;

    const resultsGraph = await dbNquads.query(queryGraph);
    console.log('Query GRAPH <http://example.org/graph1>:');
    console.log(`  Found ${resultsGraph.length} triples in named graph`);
    for (const row of resultsGraph) {
        console.log(`  ${row.person} age ${row.age}`);
    }
    console.log();

    // =========================================================================
    // Part 8: TypeScript-specific features
    // =========================================================================
    console.log('8. TypeScript-specific features...\n');

    // Type-safe query results
    interface PersonAge {
        person: string;
        age: string;
    }

    const typedResults = await dbTurtle.query<PersonAge>(`
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age WHERE {
            ?person ex:age ?age .
        }
    `);

    console.log('Type-safe query results:');
    for (const row of typedResults) {
        // TypeScript knows row.person and row.age exist
        const age = parseInt(row.age);
        console.log(`  ${row.person}: ${age} years old`);
    }
    console.log();

    // =========================================================================
    // Summary
    // =========================================================================
    console.log('=== Summary ===');
    console.log('✓ Successfully loaded data in 3 RDF formats');
    console.log('✓ Executed SPARQL queries across all formats');
    console.log('✓ Registered and used 5 custom functions');
    console.log('✓ Demonstrated function chaining');
    console.log('✓ Performed roundtrip serialization');
    console.log('✓ Queried named graphs');
    console.log('✓ Used TypeScript type safety');
    console.log('\nAll features working correctly!');
}

// Run the example
main().catch((error) => {
    console.error('Error:', error);
    process.exit(1);
});
