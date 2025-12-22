/**
 * GraphFrames Example for rust-kgdb TypeScript SDK
 *
 * Demonstrates graph analytics capabilities including:
 * - Creating graphs from vertices and edges
 * - Running PageRank algorithm
 * - Finding connected components
 * - Computing shortest paths
 * - Triangle counting
 * - Label propagation for community detection
 */

import { GraphFrame } from 'rust-kgdb';

// =============================================================================
// Example 1: Basic GraphFrame Creation
// =============================================================================

async function basicGraphFrameExample() {
    console.log('=== Basic GraphFrame Example ===\n');

    // Create a simple social network graph
    const vertices = JSON.stringify([
        { id: 'alice', name: 'Alice', age: 34 },
        { id: 'bob', name: 'Bob', age: 36 },
        { id: 'charlie', name: 'Charlie', age: 30 },
        { id: 'david', name: 'David', age: 29 },
        { id: 'esther', name: 'Esther', age: 32 },
        { id: 'fanny', name: 'Fanny', age: 36 }
    ]);

    const edges = JSON.stringify([
        { src: 'alice', dst: 'bob', relationship: 'friend' },
        { src: 'bob', dst: 'charlie', relationship: 'follow' },
        { src: 'charlie', dst: 'bob', relationship: 'follow' },
        { src: 'fanny', dst: 'charlie', relationship: 'follow' },
        { src: 'esther', dst: 'fanny', relationship: 'follow' },
        { src: 'esther', dst: 'david', relationship: 'friend' },
        { src: 'david', dst: 'alice', relationship: 'friend' }
    ]);

    // Create GraphFrame
    const graph = new GraphFrame(vertices, edges);

    // Get basic statistics
    const vertexCount = graph.vertexCount();
    const edgeCount = graph.edgeCount();

    console.log(`Graph created with ${vertexCount} vertices and ${edgeCount} edges`);
    console.log();
}

// =============================================================================
// Example 2: PageRank Algorithm
// =============================================================================

async function pageRankExample() {
    console.log('=== PageRank Example ===\n');

    // Create a citation network
    const vertices = JSON.stringify([
        { id: 'paper1', title: 'Machine Learning Fundamentals' },
        { id: 'paper2', title: 'Deep Learning Advances' },
        { id: 'paper3', title: 'Neural Networks Review' },
        { id: 'paper4', title: 'AI Applications' },
        { id: 'paper5', title: 'Computer Vision Survey' }
    ]);

    const edges = JSON.stringify([
        { src: 'paper2', dst: 'paper1' },  // paper2 cites paper1
        { src: 'paper3', dst: 'paper1' },  // paper3 cites paper1
        { src: 'paper3', dst: 'paper2' },  // paper3 cites paper2
        { src: 'paper4', dst: 'paper1' },  // paper4 cites paper1
        { src: 'paper4', dst: 'paper2' },  // paper4 cites paper2
        { src: 'paper4', dst: 'paper3' },  // paper4 cites paper3
        { src: 'paper5', dst: 'paper3' },  // paper5 cites paper3
        { src: 'paper5', dst: 'paper4' }   // paper5 cites paper4
    ]);

    const graph = new GraphFrame(vertices, edges);

    // Run PageRank with damping factor 0.85 and 20 iterations
    const resetProbability = 0.15;
    const maxIterations = 20;
    const ranks = graph.pageRank(resetProbability, maxIterations);

    // Parse and display results
    const rankResults = JSON.parse(ranks);
    console.log('PageRank Results (sorted by rank):');

    const sortedRanks = Object.entries(rankResults)
        .sort(([, a], [, b]) => (b as number) - (a as number));

    for (const [paperId, rank] of sortedRanks) {
        console.log(`  ${paperId}: ${(rank as number).toFixed(4)}`);
    }
    console.log();
}

// =============================================================================
// Example 3: Connected Components
// =============================================================================

async function connectedComponentsExample() {
    console.log('=== Connected Components Example ===\n');

    // Create a graph with multiple disconnected components
    const vertices = JSON.stringify([
        // Component 1: Social group A
        { id: 'a1' }, { id: 'a2' }, { id: 'a3' },
        // Component 2: Social group B
        { id: 'b1' }, { id: 'b2' }, { id: 'b3' }, { id: 'b4' },
        // Component 3: Isolated node
        { id: 'c1' }
    ]);

    const edges = JSON.stringify([
        // Component 1 edges
        { src: 'a1', dst: 'a2' },
        { src: 'a2', dst: 'a3' },
        { src: 'a3', dst: 'a1' },
        // Component 2 edges
        { src: 'b1', dst: 'b2' },
        { src: 'b2', dst: 'b3' },
        { src: 'b3', dst: 'b4' },
        { src: 'b4', dst: 'b1' }
        // c1 has no edges - isolated component
    ]);

    const graph = new GraphFrame(vertices, edges);

    // Find connected components
    const components = graph.connectedComponents();
    const componentResults = JSON.parse(components);

    console.log('Connected Components:');

    // Group vertices by component
    const componentGroups: Record<string, string[]> = {};
    for (const [vertex, component] of Object.entries(componentResults)) {
        const compId = String(component);
        if (!componentGroups[compId]) {
            componentGroups[compId] = [];
        }
        componentGroups[compId].push(vertex);
    }

    let compNum = 1;
    for (const [compId, members] of Object.entries(componentGroups)) {
        console.log(`  Component ${compNum}: [${members.join(', ')}]`);
        compNum++;
    }
    console.log(`\nTotal components found: ${Object.keys(componentGroups).length}`);
    console.log();
}

// =============================================================================
// Example 4: Shortest Paths
// =============================================================================

async function shortestPathsExample() {
    console.log('=== Shortest Paths Example ===\n');

    // Create a transportation network
    const vertices = JSON.stringify([
        { id: 'NYC', name: 'New York' },
        { id: 'BOS', name: 'Boston' },
        { id: 'CHI', name: 'Chicago' },
        { id: 'MIA', name: 'Miami' },
        { id: 'LAX', name: 'Los Angeles' },
        { id: 'SEA', name: 'Seattle' }
    ]);

    const edges = JSON.stringify([
        { src: 'NYC', dst: 'BOS' },
        { src: 'NYC', dst: 'CHI' },
        { src: 'NYC', dst: 'MIA' },
        { src: 'BOS', dst: 'CHI' },
        { src: 'CHI', dst: 'LAX' },
        { src: 'CHI', dst: 'SEA' },
        { src: 'LAX', dst: 'SEA' },
        { src: 'MIA', dst: 'LAX' }
    ]);

    const graph = new GraphFrame(vertices, edges);

    // Find shortest paths from NYC and LAX as landmarks
    const landmarks = ['NYC', 'LAX'];
    const paths = graph.shortestPaths(landmarks);
    const pathResults = JSON.parse(paths);

    console.log('Shortest Paths to Landmarks:');
    for (const [vertex, distances] of Object.entries(pathResults)) {
        const distObj = distances as Record<string, number>;
        const distStr = Object.entries(distObj)
            .map(([landmark, dist]) => `${landmark}=${dist}`)
            .join(', ');
        console.log(`  ${vertex}: {${distStr}}`);
    }
    console.log();
}

// =============================================================================
// Example 5: Triangle Counting
// =============================================================================

async function triangleCountExample() {
    console.log('=== Triangle Counting Example ===\n');

    // Create a graph with known triangles
    const vertices = JSON.stringify([
        { id: 'v0' }, { id: 'v1' }, { id: 'v2' },
        { id: 'v3' }, { id: 'v4' }
    ]);

    // This creates a K4 (complete graph on 4 vertices) plus one extra vertex
    // K4 has C(4,3) = 4 triangles
    const edges = JSON.stringify([
        // K4 edges (v0, v1, v2, v3)
        { src: 'v0', dst: 'v1' },
        { src: 'v0', dst: 'v2' },
        { src: 'v0', dst: 'v3' },
        { src: 'v1', dst: 'v2' },
        { src: 'v1', dst: 'v3' },
        { src: 'v2', dst: 'v3' },
        // Extra edge to v4 (no triangles with v4)
        { src: 'v3', dst: 'v4' }
    ]);

    const graph = new GraphFrame(vertices, edges);

    // Count triangles
    const triangleCount = graph.triangleCount();

    console.log(`Number of triangles in graph: ${triangleCount}`);
    console.log('(Expected: 4 triangles from the K4 subgraph)');
    console.log();
}

// =============================================================================
// Example 6: Label Propagation (Community Detection)
// =============================================================================

async function labelPropagationExample() {
    console.log('=== Label Propagation Example ===\n');

    // Create a graph with clear community structure
    const vertices = JSON.stringify([
        // Community 1: Tech enthusiasts
        { id: 'tech1' }, { id: 'tech2' }, { id: 'tech3' }, { id: 'tech4' },
        // Community 2: Sports fans
        { id: 'sport1' }, { id: 'sport2' }, { id: 'sport3' },
        // Bridge node
        { id: 'bridge' }
    ]);

    const edges = JSON.stringify([
        // Dense connections in tech community
        { src: 'tech1', dst: 'tech2' },
        { src: 'tech1', dst: 'tech3' },
        { src: 'tech2', dst: 'tech3' },
        { src: 'tech2', dst: 'tech4' },
        { src: 'tech3', dst: 'tech4' },
        // Dense connections in sports community
        { src: 'sport1', dst: 'sport2' },
        { src: 'sport1', dst: 'sport3' },
        { src: 'sport2', dst: 'sport3' },
        // Bridge connections (sparse)
        { src: 'tech4', dst: 'bridge' },
        { src: 'bridge', dst: 'sport1' }
    ]);

    const graph = new GraphFrame(vertices, edges);

    // Run label propagation for 10 iterations
    const maxIterations = 10;
    const labels = graph.labelPropagation(maxIterations);
    const labelResults = JSON.parse(labels);

    console.log('Community Detection Results:');

    // Group by label
    const communities: Record<string, string[]> = {};
    for (const [vertex, label] of Object.entries(labelResults)) {
        const labelId = String(label);
        if (!communities[labelId]) {
            communities[labelId] = [];
        }
        communities[labelId].push(vertex);
    }

    let commNum = 1;
    for (const [, members] of Object.entries(communities)) {
        console.log(`  Community ${commNum}: [${members.join(', ')}]`);
        commNum++;
    }
    console.log();
}

// =============================================================================
// Example 7: Motif Finding
// =============================================================================

async function motifFindingExample() {
    console.log('=== Motif Finding Example ===\n');

    // Create a social network for motif finding
    const vertices = JSON.stringify([
        { id: 'a' }, { id: 'b' }, { id: 'c' }, { id: 'd' }, { id: 'e' }
    ]);

    const edges = JSON.stringify([
        { src: 'a', dst: 'b' },
        { src: 'b', dst: 'c' },
        { src: 'c', dst: 'a' },  // Triangle: a -> b -> c -> a
        { src: 'c', dst: 'd' },
        { src: 'd', dst: 'e' },
        { src: 'e', dst: 'c' }   // Triangle: c -> d -> e -> c
    ]);

    const graph = new GraphFrame(vertices, edges);

    // Find simple edge pattern
    const edgePattern = '(x)-[e]->(y)';
    const edgeResults = graph.find(edgePattern);
    console.log(`Pattern "${edgePattern}": Found ${JSON.parse(edgeResults).length} matches`);

    // Find 2-hop paths
    const pathPattern = '(x)-[e1]->(y); (y)-[e2]->(z)';
    const pathResults = graph.find(pathPattern);
    console.log(`Pattern "${pathPattern}": Found ${JSON.parse(pathResults).length} matches`);

    // Find triangles
    const trianglePattern = '(a)-[]->(b); (b)-[]->(c); (c)-[]->(a)';
    const triangleResults = graph.find(trianglePattern);
    console.log(`Pattern "${trianglePattern}": Found ${JSON.parse(triangleResults).length} matches`);
    console.log();
}

// =============================================================================
// Run All Examples
// =============================================================================

async function main() {
    console.log('========================================');
    console.log('   GraphFrames SDK Examples');
    console.log('========================================\n');

    try {
        await basicGraphFrameExample();
        await pageRankExample();
        await connectedComponentsExample();
        await shortestPathsExample();
        await triangleCountExample();
        await labelPropagationExample();
        await motifFindingExample();

        console.log('========================================');
        console.log('   All examples completed successfully!');
        console.log('========================================');
    } catch (error) {
        console.error('Error running examples:', error);
        process.exit(1);
    }
}

main();
