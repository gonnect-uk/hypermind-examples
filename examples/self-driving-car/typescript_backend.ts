#!/usr/bin/env node
/**
 * Node.js/TypeScript backend for self-driving car demo
 * Uses rust_kgdb TypeScript SDK directly (NAPI-RS bindings)
 *
 * Start with: npm install && npx ts-node typescript_backend.ts
 * Or compile: npx tsc typescript_backend.ts && node typescript_backend.js
 */

import express, { Request, Response } from 'express';
import cors from 'cors';
// @ts-ignore - TypeScript SDK (NAPI-RS bindings)
import { GraphDB } from 'rust-kgdb';

const app = express();
const PORT = 8080;

// Middleware
app.use(cors());
app.use(express.json({ limit: '10mb' }));

// Create in-memory GraphDB instance (2.78 µs lookups!)
let db: any;

try {
    db = new GraphDB('http://gonnect.com/self-driving-car');
    console.log('='.repeat(70));
    console.log('🚀 rust-kgdb TypeScript SDK Backend');
    console.log('='.repeat(70));
    console.log('✅ In-memory GraphDB ready (24 bytes/triple, SPARQL 1.1/1.2)');
    console.log('✅ TypeScript SDK backend server starting on http://localhost:' + PORT);
    console.log('='.repeat(70));
} catch (error) {
    console.error('❌ Failed to initialize GraphDB:', error);
    process.exit(1);
}

/**
 * Health check endpoint
 */
app.get('/health', (req: Request, res: Response) => {
    try {
        const count = db.countTriples();
        res.json({
            status: 'healthy',
            version: 'rust-kgdb v0.1.3 (TypeScript SDK)',
            triples: count,
            backend: 'TypeScript SDK (NAPI-RS + in-memory)'
        });
    } catch (error: any) {
        res.status(500).json({
            status: 'error',
            error: error.message
        });
    }
});

/**
 * Clear all triples
 */
app.post('/clear', (req: Request, res: Response) => {
    try {
        db.clear();
        res.json({ success: true, triples: 0 });
    } catch (error: any) {
        res.status(400).json({
            success: false,
            error: error.message
        });
    }
});

/**
 * Load Turtle RDF data
 */
app.post('/load', (req: Request, res: Response) => {
    try {
        const { turtle_data } = req.body;

        if (!turtle_data) {
            return res.status(400).json({
                success: false,
                error: 'Missing turtle_data parameter'
            });
        }

        // Measure execution time
        const startTime = Date.now();

        // Load into default graph
        db.loadTtl(turtle_data, null);
        const count = db.countTriples();

        const endTime = Date.now();
        const executionTimeMs = endTime - startTime;

        res.json({
            success: true,
            triples_loaded: count,
            execution_time_ms: executionTimeMs,
            message: `Loaded ${count} triples`
        });
    } catch (error: any) {
        res.status(400).json({
            success: false,
            error: error.message
        });
    }
});

/**
 * Execute SPARQL ASK query
 */
app.post('/ask', (req: Request, res: Response) => {
    try {
        const { sparql_query } = req.body;

        if (!sparql_query) {
            return res.status(400).json({
                success: false,
                error: 'Missing sparql_query parameter'
            });
        }

        // Measure execution time in microseconds
        const startTime = process.hrtime.bigint();

        // For ASK queries, use SELECT and check if results exist
        // This is a workaround if queryAsk is not available
        const selectQuery = sparql_query.replace('ASK', 'SELECT *');
        const results = db.querySelect(selectQuery);

        const endTime = process.hrtime.bigint();
        const executionTimeUs = Number(endTime - startTime) / 1000; // nanoseconds to microseconds

        res.json({
            success: true,
            result: results.length > 0,  // ASK returns boolean
            execution_time_us: executionTimeUs
        });
    } catch (error: any) {
        res.status(400).json({
            success: false,
            error: error.message
        });
    }
});

/**
 * Execute SPARQL SELECT query
 */
app.post('/select', (req: Request, res: Response) => {
    try {
        const { sparql_query } = req.body;

        if (!sparql_query) {
            return res.status(400).json({
                success: false,
                error: 'Missing sparql_query parameter'
            });
        }

        // Measure execution time in microseconds
        const startTime = process.hrtime.bigint();

        const results = db.querySelect(sparql_query);

        const endTime = process.hrtime.bigint();
        const executionTimeUs = Number(endTime - startTime) / 1000; // nanoseconds to microseconds

        res.json({
            success: true,
            results: results,
            count: results.length,
            execution_time_us: executionTimeUs
        });
    } catch (error: any) {
        res.status(400).json({
            success: false,
            error: error.message
        });
    }
});

/**
 * Get database statistics
 */
app.get('/stats', (req: Request, res: Response) => {
    try {
        const stats = db.getStats();

        res.json({
            total_triples: stats.totalTriples,
            total_entities: stats.totalEntities,
            dictionary_size: stats.dictionarySize,
            memory_bytes: stats.memoryBytes,
            storage_backend: stats.storageBackend,
            graph_uri: stats.graphUri
        });
    } catch (error: any) {
        res.status(500).json({
            error: error.message
        });
    }
});

// Start server
app.listen(PORT, '0.0.0.0', () => {
    console.log(`\n✅ TypeScript SDK backend listening on http://localhost:${PORT}`);
    console.log(`\n📚 Endpoints:`);
    console.log(`   GET  /health  - Health check`);
    console.log(`   POST /clear   - Clear all triples`);
    console.log(`   POST /load    - Load Turtle data`);
    console.log(`   POST /ask     - SPARQL ASK query`);
    console.log(`   POST /select  - SPARQL SELECT query`);
    console.log(`   GET  /stats   - Database statistics`);
    console.log(`\n🚀 Ready to serve demo! Open DEMO_RUST_KGDB.html in browser.\n`);
});

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('\n🛑 Shutting down TypeScript backend...');
    process.exit(0);
});
