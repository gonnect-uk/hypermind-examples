# Federation Setup Guide

**Query KGDB + Snowflake + BigQuery in a single SQL statement**

```bash
npm run federation
```

---

## Overview

HyperFederate enables cross-database queries using SQL with embedded `graph_search()` UDFs:

```sql
-- Single query across KGDB + Snowflake + BigQuery
WITH kg_entities AS (
  SELECT * FROM graph_search('
    SELECT ?customer ?riskScore WHERE {
      ?customer a <Customer> .
      ?customer <riskScore> ?riskScore .
    }
  ')
),
transactions AS (
  SELECT * FROM snowflake.transactions WHERE amount > 10000
),
claims AS (
  SELECT * FROM bigquery.claims WHERE status = 'pending'
)
SELECT kg.customer, kg.riskScore, t.amount, c.claim_id
FROM kg_entities kg
JOIN transactions t ON kg.customer = t.customer_id
JOIN claims c ON t.transaction_id = c.transaction_ref
```

---

## Runtime Modes

| Mode | Usage | Servers Needed |
|------|-------|----------------|
| **In-Memory** | Demo, testing | None |
| **K8s RPC** | Production | HyperFederate cluster |

### In-Memory Mode (Default)

No database credentials needed. Uses mock tables:

```javascript
const { RpcFederationProxy } = require('rust-kgdb')

const federation = new RpcFederationProxy({
  mode: 'inMemory',
  tables: {
    'snowflake.transactions': [
      { transaction_id: 'T001', customer_id: 'C001', amount: 15000 },
      { transaction_id: 'T002', customer_id: 'C002', amount: 8500 }
    ],
    'bigquery.claims': [
      { claim_id: 'CLM001', transaction_ref: 'T001', status: 'pending' }
    ]
  }
})
```

### K8s RPC Mode (Production)

Connects to real databases via HyperFederate server:

```javascript
const federation = new RpcFederationProxy({
  mode: 'rpc',
  endpoint: 'http://hyperfederate-server:50051'
})
```

---

## Database Credentials Setup

### Environment Variables

```bash
# Snowflake
export SNOWFLAKE_ACCOUNT=your-account
export SNOWFLAKE_USER=your-user
export SNOWFLAKE_PASSWORD=your-password
export SNOWFLAKE_WAREHOUSE=your-warehouse
export SNOWFLAKE_DATABASE=your-database
export SNOWFLAKE_SCHEMA=your-schema

# BigQuery
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
export BIGQUERY_PROJECT=your-project-id
export BIGQUERY_DATASET=your-dataset

# DuckDB (for local analytics)
export DUCKDB_PATH=/path/to/database.duckdb

# PostgreSQL
export POSTGRES_HOST=localhost
export POSTGRES_PORT=5432
export POSTGRES_USER=your-user
export POSTGRES_PASSWORD=your-password
export POSTGRES_DATABASE=your-database
```

### Kubernetes Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: hyperfederate-credentials
  namespace: hypermind-data
type: Opaque
stringData:
  SNOWFLAKE_ACCOUNT: "your-account"
  SNOWFLAKE_USER: "your-user"
  SNOWFLAKE_PASSWORD: "your-password"
  SNOWFLAKE_WAREHOUSE: "COMPUTE_WH"
  SNOWFLAKE_DATABASE: "ANALYTICS"
  SNOWFLAKE_SCHEMA: "PUBLIC"
  BIGQUERY_PROJECT: "your-project-id"
  BIGQUERY_DATASET: "your-dataset"
---
apiVersion: v1
kind: Secret
metadata:
  name: bigquery-service-account
  namespace: hypermind-data
type: Opaque
data:
  # Base64 encoded service account JSON
  service-account.json: |
    ewogICJ0eXBlIjogInNlcnZpY2VfYWNjb3VudCIsCiAgInByb2plY3RfaWQi...
```

### HyperFederate Server Configuration

```yaml
# config/federation-config.yaml
connectors:
  snowflake:
    type: snowflake
    account: ${SNOWFLAKE_ACCOUNT}
    user: ${SNOWFLAKE_USER}
    password: ${SNOWFLAKE_PASSWORD}
    warehouse: ${SNOWFLAKE_WAREHOUSE}
    database: ${SNOWFLAKE_DATABASE}
    schema: ${SNOWFLAKE_SCHEMA}

  bigquery:
    type: bigquery
    project_id: ${BIGQUERY_PROJECT}
    dataset: ${BIGQUERY_DATASET}
    credentials_path: /secrets/service-account.json

  duckdb:
    type: duckdb
    path: ${DUCKDB_PATH:-/data/analytics.duckdb}

  postgres:
    type: postgres
    host: ${POSTGRES_HOST}
    port: ${POSTGRES_PORT}
    user: ${POSTGRES_USER}
    password: ${POSTGRES_PASSWORD}
    database: ${POSTGRES_DATABASE}
```

---

## Running Federation Examples

### Demo Mode (No Credentials)

```bash
# Uses in-memory mock tables
npm run federation

# 3-way federation demo
npm run federation:3way
```

### Production Mode

```bash
# Set credentials first
source .env

# Run with real databases
FEDERATION_MODE=rpc npm run federation
```

---

## Example: 3-Way Federation

Query KGDB knowledge graph + Snowflake transactions + BigQuery analytics:

```javascript
const { GraphDB, HyperMindAgent, RpcFederationProxy } = require('rust-kgdb')

// 1. Create KGDB with entity knowledge
const db = new GraphDB('http://company.com/')
db.loadTtl(`
  @prefix ex: <http://company.com/> .
  ex:customer001 a ex:HighRiskCustomer ;
      ex:riskScore 0.92 ;
      ex:name "John Doe" .
`)

// 2. Setup federation proxy
const federation = new RpcFederationProxy({
  mode: process.env.FEDERATION_MODE || 'inMemory',
  endpoint: process.env.HYPERFEDERATE_ENDPOINT || 'http://localhost:50051',
  tables: {
    'snowflake.transactions': mockTransactions,
    'bigquery.analytics': mockAnalytics
  }
})

// 3. Create agent with federation
const agent = new HyperMindAgent({
  name: 'fraud-analyzer',
  kg: db,
  federation: federation,
  apiKey: process.env.OPENAI_API_KEY
})

// 4. Query across all databases
const llmConfig = { provider: 'openai', apiKey: process.env.OPENAI_API_KEY, model: 'gpt-4o' }
const result = agent.ask(
  'Find high-risk customers with large transactions and recent claims',
  llmConfig
)

console.log(result.reasoning)     // LLM reasoning for the approach
console.log(result.answer)        // Natural language answer
console.log(result.proofHash)     // SHA-256 verification hash
```

---

## Available UDFs

### Table Functions (FROM clause)

| UDF | Description |
|-----|-------------|
| `graph_search(sparql)` | Execute SPARQL on KGDB, return as table |
| `vector_search(query, k, threshold)` | Semantic similarity search |
| `pagerank(iterations, damping)` | PageRank algorithm |
| `connected_components()` | Find connected subgraphs |
| `shortest_paths(source, hops)` | Path finding |
| `datalog_reason(program, goal)` | Datalog inference |
| `motif_search(pattern)` | Graph pattern matching |

### Scalar Functions (SELECT/WHERE)

| UDF | Description |
|-----|-------------|
| `similar_to(entity, threshold)` | Find similar entities |
| `neighbors(entity, hops)` | Get N-hop neighbors |
| `entity_type(entity)` | Get rdf:type |
| `entity_properties(entity)` | Get all properties as JSON |

---

## Troubleshooting

### Connection Issues

```bash
# Test Snowflake connection
snowsql -a $SNOWFLAKE_ACCOUNT -u $SNOWFLAKE_USER

# Test BigQuery connection
bq --project_id=$BIGQUERY_PROJECT ls $BIGQUERY_DATASET

# Test HyperFederate server
curl http://hyperfederate-server:50051/health
```

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `SNOWFLAKE_ACCOUNT not set` | Missing env var | Set credentials in .env |
| `BigQuery auth failed` | Bad service account | Check GOOGLE_APPLICATION_CREDENTIALS path |
| `Connection refused` | Server not running | Start HyperFederate server or use inMemory mode |

---

## See Also

- [Euroleague Basketball Analytics](EUROLEAGUE_ANALYTICS.md)
- [Boston Real Estate Analytics](BOSTON_REALESTATE.md)
- [US Legal Case Analysis](LEGAL_CASE.md)
