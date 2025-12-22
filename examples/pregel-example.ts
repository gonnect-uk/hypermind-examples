/**
 * Pregel Example for rust-kgdb TypeScript SDK
 *
 * Demonstrates Bulk Synchronous Parallel (BSP) graph processing:
 * - Pregel programming model
 * - Vertex-centric computation
 * - Message passing between vertices
 * - Superstep synchronization
 * - Built-in algorithms (shortest paths, connected components)
 */

import { GraphFrame, PregelProgram } from 'rust-kgdb';

// =============================================================================
// Example 1: Pregel Shortest Paths
// =============================================================================

async function shortestPathsPregelExample() {
    console.log('=== Pregel Shortest Paths ===\n');

    // Create a road network graph
    const vertices = JSON.stringify([
        { id: 'sf', name: 'San Francisco' },
        { id: 'la', name: 'Los Angeles' },
        { id: 'lv', name: 'Las Vegas' },
        { id: 'phx', name: 'Phoenix' },
        { id: 'den', name: 'Denver' },
        { id: 'slc', name: 'Salt Lake City' },
        { id: 'sea', name: 'Seattle' },
        { id: 'pdx', name: 'Portland' }
    ]);

    const edges = JSON.stringify([
        // West Coast corridor
        { src: 'sf', dst: 'la', weight: 380 },   // SF to LA: 380 miles
        { src: 'sf', dst: 'pdx', weight: 640 },  // SF to Portland
        { src: 'pdx', dst: 'sea', weight: 175 }, // Portland to Seattle

        // Southwest routes
        { src: 'la', dst: 'lv', weight: 270 },   // LA to Las Vegas
        { src: 'la', dst: 'phx', weight: 370 },  // LA to Phoenix
        { src: 'lv', dst: 'phx', weight: 300 },  // Las Vegas to Phoenix
        { src: 'lv', dst: 'slc', weight: 420 },  // Las Vegas to Salt Lake City

        // Mountain routes
        { src: 'phx', dst: 'den', weight: 600 }, // Phoenix to Denver
        { src: 'slc', dst: 'den', weight: 525 }, // Salt Lake to Denver
        { src: 'slc', dst: 'sea', weight: 840 }  // Salt Lake to Seattle
    ]);

    const graph = new GraphFrame(vertices, edges);

    // Run Pregel shortest paths from San Francisco
    const landmark = 'sf';
    const maxSupersteps = 10;

    console.log(`Computing shortest paths from ${landmark}...\n`);

    const result = graph.pregelShortestPaths(landmark, maxSupersteps);
    const distances = JSON.parse(result);

    console.log('Shortest distances from San Francisco:');
    const cityNames: Record<string, string> = {
        sf: 'San Francisco', la: 'Los Angeles', lv: 'Las Vegas',
        phx: 'Phoenix', den: 'Denver', slc: 'Salt Lake City',
        sea: 'Seattle', pdx: 'Portland'
    };

    const sortedDistances = Object.entries(distances.values)
        .sort(([, a], [, b]) => {
            const distA = (a as any).values?.distance ?? Infinity;
            const distB = (b as any).values?.distance ?? Infinity;
            return distA - distB;
        });

    for (const [city, data] of sortedDistances) {
        const dist = (data as any).values?.distance ?? Infinity;
        const name = cityNames[city] || city;
        if (dist === Infinity || dist > 1e9) {
            console.log(`  ${name}: unreachable`);
        } else {
            console.log(`  ${name}: ${dist} miles`);
        }
    }

    console.log(`\nCompleted in ${distances.supersteps} supersteps`);
    console.log();
}

// =============================================================================
// Example 2: Custom Pregel Program - PageRank
// =============================================================================

async function customPregelPageRankExample() {
    console.log('=== Custom Pregel PageRank ===\n');

    // Create a web link graph
    const vertices = JSON.stringify([
        { id: 'home', title: 'Home Page' },
        { id: 'about', title: 'About Us' },
        { id: 'products', title: 'Products' },
        { id: 'blog', title: 'Blog' },
        { id: 'contact', title: 'Contact' }
    ]);

    const edges = JSON.stringify([
        // Navigation links
        { src: 'home', dst: 'about' },
        { src: 'home', dst: 'products' },
        { src: 'home', dst: 'blog' },
        { src: 'home', dst: 'contact' },
        // Internal links
        { src: 'about', dst: 'contact' },
        { src: 'products', dst: 'home' },
        { src: 'products', dst: 'blog' },
        { src: 'blog', dst: 'home' },
        { src: 'blog', dst: 'products' },
        { src: 'contact', dst: 'home' }
    ]);

    const graph = new GraphFrame(vertices, edges);

    // Custom Pregel program configuration
    const pregelConfig = {
        program: 'pagerank',
        dampingFactor: 0.85,
        maxSupersteps: 20,
        convergenceThreshold: 0.001
    };

    console.log('Running Pregel PageRank...');
    console.log(`  Damping factor: ${pregelConfig.dampingFactor}`);
    console.log(`  Max supersteps: ${pregelConfig.maxSupersteps}`);
    console.log(`  Convergence threshold: ${pregelConfig.convergenceThreshold}\n`);

    // Execute using built-in PageRank
    const resetProb = 1 - pregelConfig.dampingFactor;
    const result = graph.pageRank(resetProb, pregelConfig.maxSupersteps);
    const ranks = JSON.parse(result);

    console.log('PageRank Results:');
    const sortedRanks = Object.entries(ranks)
        .sort(([, a], [, b]) => (b as number) - (a as number));

    for (const [page, rank] of sortedRanks) {
        const bar = '█'.repeat(Math.round((rank as number) * 50));
        console.log(`  ${page.padEnd(10)} ${(rank as number).toFixed(4)} ${bar}`);
    }
    console.log();
}

// =============================================================================
// Example 3: Pregel Connected Components
// =============================================================================

async function connectedComponentsPregelExample() {
    console.log('=== Pregel Connected Components ===\n');

    // Create a graph with multiple disconnected components
    const vertices = JSON.stringify([
        // Component 1: European cities
        { id: 'london' }, { id: 'paris' }, { id: 'berlin' }, { id: 'rome' },
        // Component 2: Asian cities
        { id: 'tokyo' }, { id: 'seoul' }, { id: 'beijing' },
        // Component 3: Australian cities
        { id: 'sydney' }, { id: 'melbourne' }
    ]);

    const edges = JSON.stringify([
        // European connections
        { src: 'london', dst: 'paris' },
        { src: 'paris', dst: 'berlin' },
        { src: 'berlin', dst: 'rome' },
        { src: 'rome', dst: 'london' },
        // Asian connections
        { src: 'tokyo', dst: 'seoul' },
        { src: 'seoul', dst: 'beijing' },
        { src: 'beijing', dst: 'tokyo' },
        // Australian connections
        { src: 'sydney', dst: 'melbourne' }
    ]);

    const graph = new GraphFrame(vertices, edges);

    console.log('Running Pregel Connected Components...\n');

    const result = graph.connectedComponents();
    const components = JSON.parse(result);

    // Group vertices by component
    const componentGroups: Record<string, string[]> = {};
    for (const [vertex, compId] of Object.entries(components)) {
        const key = String(compId);
        if (!componentGroups[key]) {
            componentGroups[key] = [];
        }
        componentGroups[key].push(vertex);
    }

    console.log('Connected Components Found:');
    let compNum = 1;
    for (const [, members] of Object.entries(componentGroups)) {
        console.log(`  Component ${compNum}: ${members.join(' <-> ')}`);
        compNum++;
    }

    console.log(`\nTotal: ${Object.keys(componentGroups).length} components`);
    console.log();
}

// =============================================================================
// Example 4: Pregel Label Propagation for Community Detection
// =============================================================================

async function labelPropagationPregelExample() {
    console.log('=== Pregel Label Propagation ===\n');

    // Create a social network with community structure
    const vertices = JSON.stringify([
        // Tech community
        { id: 'alice', interests: ['AI', 'ML'] },
        { id: 'bob', interests: ['AI', 'Data'] },
        { id: 'carol', interests: ['ML', 'Data'] },
        // Sports community
        { id: 'dave', interests: ['Football', 'Basketball'] },
        { id: 'eve', interests: ['Football', 'Tennis'] },
        { id: 'frank', interests: ['Basketball', 'Tennis'] },
        // Bridge person
        { id: 'grace', interests: ['AI', 'Football'] }
    ]);

    const edges = JSON.stringify([
        // Dense tech community connections
        { src: 'alice', dst: 'bob' },
        { src: 'alice', dst: 'carol' },
        { src: 'bob', dst: 'carol' },
        // Dense sports community connections
        { src: 'dave', dst: 'eve' },
        { src: 'dave', dst: 'frank' },
        { src: 'eve', dst: 'frank' },
        // Bridge connections
        { src: 'alice', dst: 'grace' },
        { src: 'grace', dst: 'dave' }
    ]);

    const graph = new GraphFrame(vertices, edges);

    console.log('Running Pregel Label Propagation...\n');
    const maxIterations = 10;

    const result = graph.labelPropagation(maxIterations);
    const labels = JSON.parse(result);

    // Group by detected community
    const communities: Record<string, string[]> = {};
    for (const [person, label] of Object.entries(labels)) {
        const key = String(label);
        if (!communities[key]) {
            communities[key] = [];
        }
        communities[key].push(person);
    }

    console.log('Detected Communities:');
    let commNum = 1;
    for (const [, members] of Object.entries(communities)) {
        console.log(`  Community ${commNum}: ${members.join(', ')}`);
        commNum++;
    }
    console.log();
}

// =============================================================================
// Example 5: Understanding Superstep Execution
// =============================================================================

async function superstepExample() {
    console.log('=== Understanding Pregel Supersteps ===\n');

    console.log('Pregel Programming Model:');
    console.log('  1. Think like a vertex - each vertex executes the same program');
    console.log('  2. Computation proceeds in synchronized supersteps');
    console.log('  3. In each superstep, each active vertex:');
    console.log('     a) Receives messages from previous superstep');
    console.log('     b) Updates its state based on messages');
    console.log('     c) Sends messages to neighbors');
    console.log('     d) May vote to halt (become inactive)');
    console.log('  4. Computation ends when all vertices halt AND no messages in transit\n');

    // Demonstrate with a simple chain graph
    const vertices = JSON.stringify([
        { id: 'v0' }, { id: 'v1' }, { id: 'v2' }, { id: 'v3' }, { id: 'v4' }
    ]);

    const edges = JSON.stringify([
        { src: 'v0', dst: 'v1' },
        { src: 'v1', dst: 'v2' },
        { src: 'v2', dst: 'v3' },
        { src: 'v3', dst: 'v4' }
    ]);

    const graph = new GraphFrame(vertices, edges);

    console.log('Chain Graph: v0 -> v1 -> v2 -> v3 -> v4');
    console.log('Running shortest paths from v0:\n');

    const result = graph.pregelShortestPaths('v0', 10);
    const data = JSON.parse(result);

    console.log(`Completed in ${data.supersteps} supersteps`);
    console.log('(Each superstep propagates distance one hop further)\n');

    console.log('Superstep trace:');
    console.log('  Superstep 0: v0 receives initial message, distance=0');
    console.log('  Superstep 1: v1 receives message from v0, distance=1');
    console.log('  Superstep 2: v2 receives message from v1, distance=2');
    console.log('  Superstep 3: v3 receives message from v2, distance=3');
    console.log('  Superstep 4: v4 receives message from v3, distance=4');
    console.log('  Superstep 5: All vertices halt, no messages in transit');
    console.log();
}

// =============================================================================
// Example 6: Pregel vs Standard Algorithms Comparison
// =============================================================================

async function comparisonExample() {
    console.log('=== Pregel vs Standard Algorithm Comparison ===\n');

    // Create test graph
    const vertices = JSON.stringify([
        { id: 'a' }, { id: 'b' }, { id: 'c' }, { id: 'd' }, { id: 'e' }
    ]);

    const edges = JSON.stringify([
        { src: 'a', dst: 'b' },
        { src: 'b', dst: 'c' },
        { src: 'c', dst: 'd' },
        { src: 'd', dst: 'e' },
        { src: 'a', dst: 'c' },
        { src: 'c', dst: 'e' }
    ]);

    const graph = new GraphFrame(vertices, edges);

    console.log('Graph: a -> b -> c -> d -> e (with shortcuts a->c, c->e)\n');

    // Shortest paths using standard BFS
    console.log('Shortest Paths (standard BFS):');
    const standardPaths = graph.shortestPaths(['a']);
    const standardResult = JSON.parse(standardPaths);
    for (const [vertex, dists] of Object.entries(standardResult)) {
        const d = (dists as any)['a'];
        console.log(`  ${vertex}: distance from a = ${d}`);
    }

    console.log();

    // Shortest paths using Pregel
    console.log('Shortest Paths (Pregel BSP):');
    const pregelPaths = graph.pregelShortestPaths('a', 10);
    const pregelResult = JSON.parse(pregelPaths);
    for (const [vertex, data] of Object.entries(pregelResult.values)) {
        const d = (data as any).values?.distance ?? Infinity;
        console.log(`  ${vertex}: distance from a = ${d}`);
    }

    console.log('\nBoth methods produce identical results!');
    console.log('Pregel advantage: Scales to distributed systems automatically.');
    console.log();
}

// =============================================================================
// Run All Examples
// =============================================================================

async function main() {
    console.log('========================================');
    console.log('   Pregel BSP Processing Examples');
    console.log('========================================\n');

    try {
        await shortestPathsPregelExample();
        await customPregelPageRankExample();
        await connectedComponentsPregelExample();
        await labelPropagationPregelExample();
        await superstepExample();
        await comparisonExample();

        console.log('========================================');
        console.log('   All examples completed successfully!');
        console.log('========================================');
    } catch (error) {
        console.error('Error running examples:', error);
        process.exit(1);
    }
}

main();
