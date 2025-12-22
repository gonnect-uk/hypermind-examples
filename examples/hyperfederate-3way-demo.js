/**
 * HyperFederate 3-Way Federation: KGDB + Snowflake + BigQuery
 *
 * This example demonstrates cross-database federation using the
 * IN-MEMORY WASM runtime. No external servers needed.
 *
 * Architecture:
 * +--------------------------------------------------+
 * |              RpcFederationProxy                   |
 * |              mode: 'inMemory'                     |
 * +--------------------------------------------------+
 *        |              |               |
 *        v              v               v
 *    +-------+     +---------+     +----------+
 *    | KGDB  |     |Snowflake|     | BigQuery |
 *    | (RDF) |     | (mock)  |     | (mock)   |
 *    +-------+     +---------+     +----------+
 *
 * For REAL Snowflake/BigQuery connections (K8s deployment):
 * Contact: gonnect.hypermind@gmail.com
 *
 * Run: node examples/hyperfederate-3way-demo.js
 */

const {
  GraphDB,
  HyperMindAgent,
  RpcFederationProxy
} = require('rust-kgdb')

// Sample RDF data representing customers and orders
const CUSTOMERS_RDF = `
@prefix cust: <http://example.org/customer/> .
@prefix order: <http://example.org/order/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

# OWL Properties for reasoning
cust:referredBy a owl:TransitiveProperty ;
    rdfs:label "referred by (transitive chain)" .

cust:relatedTo a owl:SymmetricProperty ;
    rdfs:label "related to" .

# Customers
cust:C001 a cust:Customer ;
    rdfs:label "Alice Johnson" ;
    cust:segment "Premium" ;
    cust:region "US-West" ;
    cust:creditScore "780"^^xsd:integer .

cust:C002 a cust:Customer ;
    rdfs:label "Bob Smith" ;
    cust:segment "Standard" ;
    cust:region "US-East" ;
    cust:creditScore "720"^^xsd:integer ;
    cust:referredBy cust:C001 .

cust:C003 a cust:Customer ;
    rdfs:label "Carol Davis" ;
    cust:segment "Premium" ;
    cust:region "EU-West" ;
    cust:creditScore "695"^^xsd:integer ;
    cust:referredBy cust:C002 .

cust:C004 a cust:Customer ;
    rdfs:label "David Wilson" ;
    cust:segment "Enterprise" ;
    cust:region "APAC" ;
    cust:creditScore "810"^^xsd:integer .

cust:C005 a cust:Customer ;
    rdfs:label "Eve Martinez" ;
    cust:segment "Standard" ;
    cust:region "US-West" ;
    cust:creditScore "650"^^xsd:integer ;
    cust:referredBy cust:C003 .

# Customer relationships
cust:C001 cust:relatedTo cust:C002 .
cust:C002 cust:relatedTo cust:C003 .

# Orders
order:O001 a order:Order ;
    order:customer cust:C001 ;
    order:amount "15000"^^xsd:decimal ;
    order:status "completed" ;
    order:date "2024-01-15"^^xsd:date .

order:O002 a order:Order ;
    order:customer cust:C002 ;
    order:amount "8500"^^xsd:decimal ;
    order:status "completed" ;
    order:date "2024-02-20"^^xsd:date .

order:O003 a order:Order ;
    order:customer cust:C003 ;
    order:amount "22000"^^xsd:decimal ;
    order:status "pending" ;
    order:date "2024-03-10"^^xsd:date .

order:O004 a order:Order ;
    order:customer cust:C001 ;
    order:amount "45000"^^xsd:decimal ;
    order:status "completed" ;
    order:date "2024-04-05"^^xsd:date .

order:O005 a order:Order ;
    order:customer cust:C004 ;
    order:amount "125000"^^xsd:decimal ;
    order:status "completed" ;
    order:date "2024-05-15"^^xsd:date .
`

async function main() {
  console.log('='.repeat(70))
  console.log('  HYPERFEDERATE 3-WAY FEDERATION DEMO')
  console.log('  KGDB + Snowflake + BigQuery (IN-MEMORY WASM)')
  console.log('='.repeat(70))
  console.log()

  // =========================================================
  // STEP 1: Create In-Memory KGDB
  // =========================================================
  console.log('[1] Creating In-Memory KGDB...')
  const db = new GraphDB('http://example.org/')
  db.loadTtl(CUSTOMERS_RDF, null)
  console.log(`    Loaded ${db.countTriples()} triples`)
  console.log()

  // =========================================================
  // STEP 2: Configure Federation (IN-MEMORY MODE)
  // =========================================================
  console.log('[2] Configuring RpcFederationProxy...')
  console.log()
  console.log('    ┌─────────────────────────────────────────────────┐')
  console.log('    │           RpcFederationProxy                    │')
  console.log('    │           mode: "inMemory"                      │')
  console.log('    │                                                 │')
  console.log('    │  No external server needed.                     │')
  console.log('    │  Runs entirely via NAPI-RS (Rust→Node.js)       │')
  console.log('    └─────────────────────────────────────────────────┘')
  console.log('                 │          │          │')
  console.log('                 ▼          ▼          ▼')
  console.log('           ┌────────┐ ┌──────────┐ ┌──────────┐')
  console.log('           │  KGDB  │ │Snowflake │ │ BigQuery │')
  console.log('           │ (real) │ │ (mock*)  │ │ (mock*)  │')
  console.log('           └────────┘ └──────────┘ └──────────┘')
  console.log()
  console.log('    * Mock connectors simulate external DB responses.')
  console.log('      For real connections, contact gonnect.hypermind@gmail.com')
  console.log()

  const federation = new RpcFederationProxy({
    mode: 'inMemory',  // WASM runtime - no external server
    kg: db,            // In-memory KGDB

    // Connector configuration (credentials from environment)
    // In inMemory mode, these are simulated
    connectors: {
      snowflake: {
        // Real credentials would come from environment:
        // account: process.env.SNOWFLAKE_ACCOUNT,
        // user: process.env.SNOWFLAKE_USER,
        // password: process.env.SNOWFLAKE_PASSWORD,
        database: 'SNOWFLAKE_SAMPLE_DATA',
        schema: 'TPCH_SF1',
        warehouse: 'COMPUTE_WH'
      },
      bigquery: {
        // Real credentials would come from environment:
        // projectId: process.env.GCP_PROJECT_ID,
        // keyFilename: process.env.GOOGLE_APPLICATION_CREDENTIALS,
        projectId: 'demo-project',
        dataset: 'analytics'
      }
    }
  })

  console.log('    Federation configured:')
  console.log(`      Mode: ${federation.getMode()}`)
  console.log(`      In-Memory: ${federation.isInMemory()}`)
  console.log(`      KGDB: ${db.countTriples()} triples`)
  console.log(`      Snowflake: ${federation.connectors?.snowflake?.database || 'configured'}`)
  console.log(`      BigQuery: ${federation.connectors?.bigquery?.projectId || 'configured'}`)
  console.log()

  // =========================================================
  // STEP 3: Create HyperMindAgent
  // =========================================================
  console.log('[3] Creating HyperMindAgent...')
  const agent = new HyperMindAgent({
    name: 'federation-analyst',
    kg: db,
    federate: federation,
    // LLM is optional - works without it via schema-based generation
    apiKey: process.env.OPENAI_API_KEY,
    model: 'gpt-4o'
  })
  console.log(`    Agent: ${agent.name}`)
  console.log(`    ThinkingReasoner: Active`)
  console.log()

  // =========================================================
  // STEP 4: Run Federated Queries
  // =========================================================
  console.log('[4] Running Federated Queries...')
  console.log()

  // Query 1: KGDB only
  console.log('-'.repeat(70))
  console.log('QUERY 1: Find Premium customers (KGDB)')
  console.log('-'.repeat(70))
  const q1 = `
    PREFIX cust: <http://example.org/customer/>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    SELECT ?customer ?name ?segment WHERE {
      ?customer a cust:Customer .
      ?customer rdfs:label ?name .
      ?customer cust:segment ?segment .
      FILTER(?segment = "Premium")
    }
  `
  const r1 = db.querySelect(q1)
  console.log(`  Results: ${r1.length} Premium customers`)
  for (const row of r1) {
    const b = row.bindings || row
    console.log(`    - ${extractLabel(b.name)} (${extractLabel(b.segment)})`)
  }
  console.log()

  // Query 2: Referral chain (transitive property)
  console.log('-'.repeat(70))
  console.log('QUERY 2: Find referral chains (OWL TransitiveProperty)')
  console.log('-'.repeat(70))
  const q2 = `
    PREFIX cust: <http://example.org/customer/>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    SELECT ?customer ?referrer WHERE {
      ?customer cust:referredBy ?referrer .
    }
  `
  const r2 = db.querySelect(q2)
  console.log(`  Referral relationships: ${r2.length}`)
  for (const row of r2) {
    const b = row.bindings || row
    console.log(`    ${extractLabel(b.customer)} <- referred by <- ${extractLabel(b.referrer)}`)
  }
  console.log()
  console.log('  Deductive reasoning (TransitiveProperty):')
  console.log('    C001 -> C002 -> C003 -> C005')
  console.log('    Therefore: C001 indirectly referred C003 and C005')
  console.log()

  // Query 3: Federated query simulation
  console.log('-'.repeat(70))
  console.log('QUERY 3: Cross-Database Federation (KGDB + mock SF/BQ)')
  console.log('-'.repeat(70))
  const federatedResult = await federation.query(`
    SELECT c.customer_id, c.name, c.segment,
           s.total_orders, s.lifetime_value,
           b.risk_score
    FROM kgdb.customers c
    JOIN snowflake.order_summary s ON c.customer_id = s.customer_id
    JOIN bigquery.risk_scores b ON c.customer_id = b.customer_id
    WHERE c.segment = 'Premium'
  `)
  console.log('  Federation Result:')
  console.log(`    Mode: ${federatedResult.mode}`)
  console.log(`    Sources: ${federatedResult.sources}`)
  console.log(`    Rows: ${federatedResult.rows?.length || 'N/A'}`)
  console.log(`    Duration: ${federatedResult.duration}ms`)
  console.log()

  // Query 4: Natural language via HyperMindAgent
  console.log('-'.repeat(70))
  console.log('QUERY 4: Natural Language Query')
  console.log('-'.repeat(70))
  console.log('  Question: "Find customers referred by Alice"')
  console.log()

  try {
    const nlResult = await agent.call('Find customers referred by Alice')

    console.log('  Answer:', nlResult.answer || '(see results)')

    if (nlResult.raw_results?.length > 0) {
      console.log()
      console.log('  Results from SPARQL:')
      for (const r of nlResult.raw_results) {
        if (r.success && Array.isArray(r.result)) {
          for (const row of r.result.slice(0, 5)) {
            const b = row.bindings || row
            const vals = Object.entries(b)
              .map(([k, v]) => `${k}=${extractLabel(v)}`)
              .join(', ')
            console.log(`    ${vals}`)
          }
        }
      }
    }

    if (nlResult.reasoningStats) {
      console.log()
      console.log('  Reasoning Stats:')
      console.log(`    Events: ${nlResult.reasoningStats.events || 0}`)
      console.log(`    Facts: ${nlResult.reasoningStats.facts || 0}`)
      console.log(`    Proofs: ${nlResult.reasoningStats.proofs || 0}`)
    }
  } catch (err) {
    console.log(`  Error: ${err.message}`)
  }
  console.log()

  // =========================================================
  // STEP 5: Summary
  // =========================================================
  console.log('='.repeat(70))
  console.log('  HOW FEDERATION WORKS')
  console.log('='.repeat(70))
  console.log()
  console.log('  IN-MEMORY MODE (This Demo):')
  console.log('  ---------------------------')
  console.log('  - RpcFederationProxy with mode: "inMemory"')
  console.log('  - KGDB runs via NAPI-RS (Rust native addon)')
  console.log('  - External DBs simulated with mock responses')
  console.log('  - No server, no Docker, no Kubernetes')
  console.log('  - Just: npm install && npm run federation:3way')
  console.log()
  console.log('  K8S MODE (Production):')
  console.log('  -----------------------')
  console.log('  - RpcFederationProxy with mode: "rpc"')
  console.log('  - Connects to HyperFederate K8s service')
  console.log('  - Real Snowflake/BigQuery connections')
  console.log('  - Distributed query execution')
  console.log('  - Contact: gonnect.hypermind@gmail.com')
  console.log()
  console.log('  SETUP FOR REAL CONNECTIONS:')
  console.log('  ---------------------------')
  console.log('  export SNOWFLAKE_ACCOUNT=your-account')
  console.log('  export SNOWFLAKE_USER=your-user')
  console.log('  export SNOWFLAKE_PASSWORD=your-password')
  console.log('  export GOOGLE_APPLICATION_CREDENTIALS=/path/to/key.json')
  console.log('  export GCP_PROJECT_ID=your-project')
  console.log()
  console.log('  Then use mode: "rpc" and endpoint: "http://your-k8s-service:30180"')
  console.log()
}

function extractLabel(uri) {
  if (!uri) return ''
  if (typeof uri !== 'string') return String(uri)
  uri = uri.replace(/^<|>$/g, '').replace(/"/g, '')
  const idx = Math.max(uri.lastIndexOf('#'), uri.lastIndexOf('/'))
  return idx >= 0 ? uri.substring(idx + 1) : uri
}

main().catch(console.error)
