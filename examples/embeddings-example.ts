/**
 * Embeddings Example for rust-kgdb TypeScript SDK
 *
 * Demonstrates vector embedding capabilities including:
 * - Storing and retrieving embeddings
 * - Similarity search with HNSW
 * - Composite multi-provider embeddings
 * - Aggregation strategies (RRF, voting, max)
 * - 1-hop ARCADE neighbor cache
 */

import { EmbeddingService, AggregationStrategy } from 'rust-kgdb';

// =============================================================================
// Example 1: Basic Embedding Storage
// =============================================================================

async function basicEmbeddingExample() {
    console.log('=== Basic Embedding Storage ===\n');

    const embeddingService = new EmbeddingService();

    // Store embeddings for entities
    const entities = [
        { id: 'http://example.org/apple', name: 'Apple Inc' },
        { id: 'http://example.org/microsoft', name: 'Microsoft Corp' },
        { id: 'http://example.org/google', name: 'Google LLC' },
        { id: 'http://example.org/amazon', name: 'Amazon.com' },
        { id: 'http://example.org/tesla', name: 'Tesla Inc' }
    ];

    // Generate mock embeddings (384 dimensions)
    for (const entity of entities) {
        // In production, use actual embedding providers
        const embedding = generateMockEmbedding(384, entity.id);
        embeddingService.storeVector(entity.id, embedding);
        console.log(`Stored embedding for ${entity.name} (${embedding.length} dims)`);
    }

    // Retrieve an embedding
    const appleEmbedding = embeddingService.getVector('http://example.org/apple');
    if (appleEmbedding) {
        console.log(`\nRetrieved Apple embedding: [${appleEmbedding.slice(0, 5).join(', ')}...]`);
    }
    console.log();
}

// =============================================================================
// Example 2: Similarity Search with HNSW
// =============================================================================

async function similaritySearchExample() {
    console.log('=== Similarity Search with HNSW ===\n');

    const embeddingService = new EmbeddingService();

    // Create a product catalog with embeddings
    const products = [
        { id: 'product/laptop-1', category: 'electronics', name: 'MacBook Pro' },
        { id: 'product/laptop-2', category: 'electronics', name: 'ThinkPad X1' },
        { id: 'product/phone-1', category: 'electronics', name: 'iPhone 15' },
        { id: 'product/phone-2', category: 'electronics', name: 'Galaxy S24' },
        { id: 'product/tablet-1', category: 'electronics', name: 'iPad Pro' },
        { id: 'product/shirt-1', category: 'clothing', name: 'Cotton T-Shirt' },
        { id: 'product/jeans-1', category: 'clothing', name: 'Denim Jeans' },
        { id: 'product/book-1', category: 'books', name: 'AI Handbook' },
        { id: 'product/book-2', category: 'books', name: 'ML Guide' }
    ];

    // Store embeddings with category-aware vectors
    for (const product of products) {
        const embedding = generateCategoryEmbedding(384, product.category, product.name);
        embeddingService.storeVector(product.id, embedding);
    }

    console.log(`Indexed ${products.length} products\n`);

    // Find similar products to MacBook Pro
    const queryId = 'product/laptop-1';
    const k = 5;
    const threshold = 0.5;

    console.log(`Finding top ${k} products similar to MacBook Pro:`);
    const similar = embeddingService.findSimilar(queryId, k, threshold);
    const results = JSON.parse(similar);

    for (const result of results) {
        const product = products.find(p => p.id === result.entity);
        console.log(`  ${product?.name}: similarity=${result.similarity.toFixed(4)}`);
    }
    console.log();
}

// =============================================================================
// Example 3: Composite Multi-Provider Embeddings
// =============================================================================

async function compositeEmbeddingExample() {
    console.log('=== Composite Multi-Provider Embeddings ===\n');

    const embeddingService = new EmbeddingService();

    // Entity with multiple embedding representations
    const entityId = 'http://example.org/apple-inc';

    // Store embeddings from multiple providers
    const compositeEmbeddings = {
        // OpenAI text-embedding-3-small (1536 dims, but we simulate with 384)
        openai: generateProviderEmbedding(384, 'openai', 'Apple Inc technology company'),

        // Voyage AI voyage-2 (1024 dims, simulated)
        voyage: generateProviderEmbedding(384, 'voyage', 'Apple Inc technology company'),

        // Cohere embed-v3 (1024 dims, simulated)
        cohere: generateProviderEmbedding(384, 'cohere', 'Apple Inc technology company'),

        // Local RDF2Vec structural embedding
        rdf2vec: generateProviderEmbedding(384, 'rdf2vec', 'http://example.org/apple-inc')
    };

    // Store as composite embedding
    embeddingService.storeComposite(entityId, JSON.stringify(compositeEmbeddings));

    console.log('Stored composite embedding with providers:');
    for (const provider of Object.keys(compositeEmbeddings)) {
        console.log(`  - ${provider}: ${compositeEmbeddings[provider].length} dimensions`);
    }

    // Retrieve composite embedding
    const retrieved = embeddingService.getComposite(entityId);
    if (retrieved) {
        const composite = JSON.parse(retrieved);
        console.log(`\nRetrieved composite with ${Object.keys(composite.embeddings).length} providers`);
    }
    console.log();
}

// =============================================================================
// Example 4: Multi-Provider Similarity with Aggregation
// =============================================================================

async function aggregationExample() {
    console.log('=== Multi-Provider Similarity with Aggregation ===\n');

    const embeddingService = new EmbeddingService();

    // Create entities with composite embeddings
    const entities = [
        'http://example.org/apple',
        'http://example.org/google',
        'http://example.org/microsoft',
        'http://example.org/amazon',
        'http://example.org/meta'
    ];

    // Store composite embeddings for each
    for (const entityId of entities) {
        const composite = {
            openai: generateProviderEmbedding(384, 'openai', entityId),
            voyage: generateProviderEmbedding(384, 'voyage', entityId),
            cohere: generateProviderEmbedding(384, 'cohere', entityId)
        };
        embeddingService.storeComposite(entityId, JSON.stringify(composite));
    }

    console.log(`Stored composite embeddings for ${entities.length} entities\n`);

    const queryEntity = 'http://example.org/apple';
    const k = 3;
    const threshold = 0.3;

    // Test different aggregation strategies
    const strategies: AggregationStrategy[] = ['rrf', 'max', 'voting'];

    for (const strategy of strategies) {
        console.log(`Aggregation Strategy: ${strategy.toUpperCase()}`);

        const results = embeddingService.findSimilarComposite(
            queryEntity,
            k,
            threshold,
            strategy
        );

        const parsed = JSON.parse(results);
        for (const result of parsed) {
            console.log(`  ${result.entity}: score=${result.similarity.toFixed(4)}`);
        }
        console.log();
    }
}

// =============================================================================
// Example 5: 1-Hop ARCADE Neighbor Cache
// =============================================================================

async function arcadeNeighborExample() {
    console.log('=== 1-Hop ARCADE Neighbor Cache ===\n');

    const embeddingService = new EmbeddingService();

    // Build a knowledge graph with edges
    const edges = [
        ['alice', 'knows', 'bob'],
        ['alice', 'knows', 'charlie'],
        ['bob', 'knows', 'david'],
        ['charlie', 'knows', 'eve'],
        ['david', 'works_with', 'eve']
    ];

    // Add edges to ARCADE cache
    for (const [src, , dst] of edges) {
        embeddingService.addEdge(src, dst);
    }

    console.log(`Added ${edges.length} edges to ARCADE cache\n`);

    // Query 1-hop neighbors
    const testEntities = ['alice', 'bob', 'charlie', 'eve'];

    for (const entity of testEntities) {
        const neighbors = embeddingService.getNeighbors(entity, 'both');
        const neighborList = JSON.parse(neighbors);
        console.log(`${entity}'s neighbors: [${neighborList.join(', ')}]`);
    }

    // Find similar neighbors (combining structure + embeddings)
    console.log('\nFinding similar 1-hop neighbors for Alice:');
    const similarNeighbors = embeddingService.findSimilarNeighbors('alice', 10, 0.3);
    const parsed = JSON.parse(similarNeighbors);
    for (const result of parsed) {
        console.log(`  ${result.entity}: similarity=${result.similarity.toFixed(4)}`);
    }
    console.log();
}

// =============================================================================
// Example 6: Embedding Statistics and Metrics
// =============================================================================

async function metricsExample() {
    console.log('=== Embedding Statistics and Metrics ===\n');

    const embeddingService = new EmbeddingService();

    // Populate with test data
    for (let i = 0; i < 100; i++) {
        const entityId = `entity-${i}`;
        const embedding = generateMockEmbedding(384, entityId);
        embeddingService.storeVector(entityId, embedding);
    }

    // Get service metrics
    const metricsJson = embeddingService.getMetrics();
    const metrics = JSON.parse(metricsJson);

    console.log('Embedding Service Metrics:');
    console.log(`  Total embeddings: ${metrics.embedding_count}`);
    console.log(`  HNSW index size: ${metrics.hnsw_size}`);
    console.log(`  Storage size (bytes): ${metrics.storage_bytes}`);

    // Get cache statistics
    const cacheStatsJson = embeddingService.getCacheStats();
    const cacheStats = JSON.parse(cacheStatsJson);

    console.log('\nARCADE Cache Statistics:');
    console.log(`  Cache entries: ${cacheStats.entries}`);
    console.log(`  Hit rate: ${(cacheStats.hit_rate * 100).toFixed(1)}%`);
    console.log(`  Memory usage: ${cacheStats.memory_bytes} bytes`);
    console.log();
}

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Generate a deterministic mock embedding based on entity ID
 */
function generateMockEmbedding(dimensions: number, entityId: string): number[] {
    const hash = simpleHash(entityId);
    const embedding: number[] = [];

    for (let i = 0; i < dimensions; i++) {
        // Generate pseudo-random values based on hash and index
        const value = Math.sin(hash + i * 0.1) * 0.5 + 0.5;
        embedding.push(value);
    }

    // Normalize to unit length
    const norm = Math.sqrt(embedding.reduce((sum, v) => sum + v * v, 0));
    return embedding.map(v => v / norm);
}

/**
 * Generate embedding with category bias
 */
function generateCategoryEmbedding(dimensions: number, category: string, name: string): number[] {
    const categoryHash = simpleHash(category);
    const nameHash = simpleHash(name);
    const embedding: number[] = [];

    for (let i = 0; i < dimensions; i++) {
        // Combine category and name influence
        const categoryInfluence = Math.sin(categoryHash + i * 0.05) * 0.3;
        const nameInfluence = Math.sin(nameHash + i * 0.1) * 0.5;
        const value = 0.5 + categoryInfluence + nameInfluence;
        embedding.push(Math.max(0, Math.min(1, value)));
    }

    // Normalize
    const norm = Math.sqrt(embedding.reduce((sum, v) => sum + v * v, 0));
    return embedding.map(v => v / norm);
}

/**
 * Generate provider-specific embedding
 */
function generateProviderEmbedding(dimensions: number, provider: string, text: string): number[] {
    const providerHash = simpleHash(provider);
    const textHash = simpleHash(text);
    const embedding: number[] = [];

    for (let i = 0; i < dimensions; i++) {
        // Each provider has slightly different embedding characteristics
        const providerBias = Math.cos(providerHash + i * 0.02) * 0.2;
        const textValue = Math.sin(textHash + i * 0.08) * 0.6;
        const value = 0.5 + providerBias + textValue;
        embedding.push(Math.max(0, Math.min(1, value)));
    }

    // Normalize
    const norm = Math.sqrt(embedding.reduce((sum, v) => sum + v * v, 0));
    return embedding.map(v => v / norm);
}

/**
 * Simple string hash function
 */
function simpleHash(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = ((hash << 5) - hash) + char;
        hash = hash & hash; // Convert to 32bit integer
    }
    return Math.abs(hash);
}

// =============================================================================
// Run All Examples
// =============================================================================

async function main() {
    console.log('========================================');
    console.log('   Embeddings SDK Examples');
    console.log('========================================\n');

    try {
        await basicEmbeddingExample();
        await similaritySearchExample();
        await compositeEmbeddingExample();
        await aggregationExample();
        await arcadeNeighborExample();
        await metricsExample();

        console.log('========================================');
        console.log('   All examples completed successfully!');
        console.log('========================================');
    } catch (error) {
        console.error('Error running examples:', error);
        process.exit(1);
    }
}

main();
