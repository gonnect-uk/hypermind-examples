#!/usr/bin/env node
/**
 * ================================================================================
 * HYPERFEDERATE + HYPERMIND AGENT INTEGRATION DEMO
 *
 * Demonstrates:
 * - HyperMindAgent with AUTOMATIC deductive reasoning (ThinkingReasoner)
 * - Cross-database federation with Snowflake TPCH data
 * - Proof-carrying outputs with cryptographic hashes
 * - BRAIN ontology for fraud detection
 *
 * Run: SNOWFLAKE_ACCOUNT=crvrogz-iw23234 SNOWFLAKE_USER=HPERMIND \
 *      SNOWFLAKE_PASSWORD=xxx node examples/hyperfederate-hypermind-demo.js
 *
 * ================================================================================
 */

const {
  GraphDB,
  HyperMindAgent,
  ThinkingReasoner,
  RpcFederationProxy,
  getVersion
} = require('rust-kgdb')

// =============================================================================
// BRAIN ONTOLOGY - Business Reasoning & AI Intelligence Network
// =============================================================================

const BRAIN_ONTOLOGY = `
@prefix brain: <http://brain.gonnect.ai/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# =========================================================================
# OWL Properties (Auto-generate Datalog rules for ThinkingReasoner)
# =========================================================================

# Transitive: A transfers B, B transfers C => A transfers C (fraud chain)
brain:transfers a owl:TransitiveProperty ;
    rdfs:label "transfers funds to"@en ;
    rdfs:domain brain:Entity ;
    rdfs:range brain:Entity .

# Symmetric: A relatedTo B => B relatedTo A
brain:relatedTo a owl:SymmetricProperty ;
    rdfs:label "is related to"@en .

brain:connectedVia a owl:SymmetricProperty .

# Transitive: orders from same supplier chain
brain:ordersFrom a owl:TransitiveProperty ;
    rdfs:label "orders from supplier"@en .

# =========================================================================
# Class Hierarchy
# =========================================================================

brain:Entity a owl:Class .
brain:Customer rdfs:subClassOf brain:Entity .
brain:Provider rdfs:subClassOf brain:Entity .
brain:Supplier rdfs:subClassOf brain:Entity .

brain:Transaction a owl:Class .
brain:Order rdfs:subClassOf brain:Transaction .
brain:Claim rdfs:subClassOf brain:Transaction .

brain:HighRiskClaim rdfs:subClassOf brain:Claim .
brain:FraudPattern a owl:Class .
brain:CircularPayment rdfs:subClassOf brain:FraudPattern .

# =========================================================================
# SAMPLE DATA: TPCH-style Customers with Fraud Patterns
# =========================================================================

# Customers (from Snowflake TPCH CUSTOMER table concept)
brain:cust001 a brain:Customer ;
    brain:customerId "C-001" ;
    brain:name "Customer#000000001" ;
    brain:accountBalance "711.56"^^xsd:decimal ;
    brain:segment "BUILDING" ;
    brain:riskScore "0.15"^^xsd:decimal .

brain:cust002 a brain:Customer ;
    brain:customerId "C-002" ;
    brain:name "Customer#000000002" ;
    brain:accountBalance "121.65"^^xsd:decimal ;
    brain:segment "AUTOMOBILE" ;
    brain:riskScore "0.22"^^xsd:decimal .

brain:cust003 a brain:Customer ;
    brain:customerId "C-003" ;
    brain:name "Customer#000000003" ;
    brain:accountBalance "7498.12"^^xsd:decimal ;
    brain:segment "MACHINERY" ;
    brain:riskScore "0.85"^^xsd:decimal .

brain:cust004 a brain:Customer ;
    brain:customerId "C-004" ;
    brain:name "Customer#000000004" ;
    brain:accountBalance "-866.22"^^xsd:decimal ;
    brain:segment "FURNITURE" ;
    brain:riskScore "0.91"^^xsd:decimal .

# Suppliers (from Snowflake TPCH SUPPLIER table concept)
brain:supp001 a brain:Supplier ;
    brain:entityId "S-001" ;
    brain:name "Supplier#000000001" ;
    brain:riskScore "0.08"^^xsd:decimal .

brain:supp002 a brain:Supplier ;
    brain:entityId "S-002" ;
    brain:name "Supplier#000000002" ;
    brain:riskScore "0.62"^^xsd:decimal .

brain:supp003 a brain:Supplier ;
    brain:entityId "S-003" ;
    brain:name "Supplier#000000003" ;
    brain:riskScore "0.95"^^xsd:decimal .

# =========================================================================
# CIRCULAR PAYMENT FRAUD PATTERN (The AI Must Deduce This!)
# =========================================================================

brain:cust001 brain:transfers brain:cust002 .
brain:cust002 brain:transfers brain:cust003 .
brain:cust003 brain:transfers brain:cust001 .  # Closes the circle!

# Supplier collusion
brain:supp002 brain:transfers brain:supp003 .
brain:supp003 brain:transfers brain:supp002 .

# Entity relationships
brain:cust001 brain:relatedTo brain:cust002 .
brain:cust002 brain:connectedVia brain:cust003 .
brain:supp002 brain:relatedTo brain:supp003 .
`

// =============================================================================
// NOTE: RpcFederationProxy now supports TWO modes:
//
// 1. IN-MEMORY (WASM): GraphDB runs in-process via NAPI-RS - NO external server
//    const federation = new RpcFederationProxy({ mode: 'inMemory', kg: myGraphDB })
//
// 2. RPC MODE: Connects to remote HyperFederate server for distributed queries
//    const federation = new RpcFederationProxy({ mode: 'rpc', endpoint: 'http://...' })
// =============================================================================

// =============================================================================
// MAIN DEMONSTRATION
// =============================================================================

async function main() {
  const startTime = Date.now()

  console.log()
  console.log('='.repeat(80))
  console.log('  HYPERFEDERATE + HYPERMIND AGENT INTEGRATION')
  console.log('  Neuro-Symbolic AI with Cross-Database Federation')
  console.log('='.repeat(80))
  console.log()
  console.log(`  rust-kgdb Version: ${getVersion()}`)
  console.log()

  // =========================================================================
  // STEP 1: Create Knowledge Graph with BRAIN Ontology
  // =========================================================================

  console.log('+------------------------------------------------------------------------+')
  console.log('|  STEP 1: Loading BRAIN Ontology into KGDB                              |')
  console.log('+------------------------------------------------------------------------+')
  console.log()

  const db = new GraphDB('http://brain.gonnect.ai/')
  db.loadTtl(BRAIN_ONTOLOGY, null)

  const tripleCount = db.countTriples()
  console.log(`  Knowledge Graph: ${tripleCount} triples loaded`)
  console.log('  OWL Properties:')
  console.log('    - brain:transfers (TransitiveProperty) -> Fraud chain detection')
  console.log('    - brain:relatedTo (SymmetricProperty) -> Network analysis')
  console.log('    - brain:ordersFrom (TransitiveProperty) -> Supplier collusion')
  console.log()

  // =========================================================================
  // STEP 2: Configure RpcFederationProxy in IN-MEMORY MODE (WASM)
  // =========================================================================

  console.log('+------------------------------------------------------------------------+')
  console.log('|  STEP 2: Configuring RpcFederationProxy (IN-MEMORY WASM MODE)          |')
  console.log('+------------------------------------------------------------------------+')
  console.log()

  // IN-MEMORY MODE: GraphDB runs in-process via NAPI-RS
  // No external HyperFederate server needed!
  const federation = new RpcFederationProxy({
    mode: 'inMemory',              // ★ WASM mode - runs locally via NAPI-RS
    kg: db,                        // ★ Pass GraphDB instance for in-memory execution
    connectors: {
      snowflake: {
        account: process.env.SNOWFLAKE_ACCOUNT || 'crvrogz-iw23234',
        user: process.env.SNOWFLAKE_USER || 'HPERMIND',
        password: process.env.SNOWFLAKE_PASSWORD || '(not set)',
        warehouse: process.env.SNOWFLAKE_WAREHOUSE || 'COMPUTE_WH',
        database: 'SNOWFLAKE_SAMPLE_DATA',
        schema: 'TPCH_SF1'
      }
    }
  })

  console.log(`  Mode: ${federation.getMode()} (WASM - no external server)`)
  console.log(`  In-Memory: ${federation.isInMemory()}`)
  console.log(`  GraphDB: ${db.countTriples()} triples loaded`)
  console.log(`  Snowflake: ${federation.connectors.snowflake.database}.${federation.connectors.snowflake.schema}`)
  console.log()

  // =========================================================================
  // STEP 3: Create HyperMindAgent with ThinkingReasoner
  // =========================================================================

  console.log('+------------------------------------------------------------------------+')
  console.log('|  STEP 3: Creating HyperMindAgent with ThinkingReasoner                 |')
  console.log('+------------------------------------------------------------------------+')
  console.log()

  const agent = new HyperMindAgent({
    name: 'brain-fraud-detector',
    kg: db,
    apiKey: process.env.OPENAI_API_KEY,  // Optional: for LLM summarization
    model: 'gpt-4o'
  })

  const stats = agent.getReasoningStats()
  console.log('  HyperMindAgent Created:')
  console.log(`    Name: ${agent.name}`)
  console.log(`    Rules auto-generated: ${stats.rules}`)
  console.log(`    Facts loaded: ${stats.facts}`)
  console.log(`    ThinkingReasoner: Active`)
  console.log()

  // =========================================================================
  // STEP 4: Execute Agent Query with AUTOMATIC Deductive Reasoning
  // =========================================================================

  console.log('+------------------------------------------------------------------------+')
  console.log('|  STEP 4: Executing Query with AUTOMATIC Deductive Reasoning           |')
  console.log('+------------------------------------------------------------------------+')
  console.log()

  console.log('  QUERY: "Find circular payment patterns indicating fraud"')
  console.log()

  const result = await agent.call('Find circular payment patterns indicating fraud')

  // =========================================================================
  // STEP 5: Display Results
  // =========================================================================

  console.log('+------------------------------------------------------------------------+')
  console.log('|  STEP 5: RESULTS (with Deductive Proofs)                               |')
  console.log('+------------------------------------------------------------------------+')
  console.log()

  console.log('  ANSWER:')
  console.log('  -------')
  console.log(`  ${result.answer}`)
  console.log()

  console.log('  REASONING STATS:')
  console.log('  ----------------')
  console.log(`    Events: ${result.reasoningStats?.events || 0}`)
  console.log(`    Facts: ${result.reasoningStats?.facts || 0}`)
  console.log(`    Rules: ${result.reasoningStats?.rules || 0}`)
  console.log(`    Proofs: ${result.reasoningStats?.proofs || 0}`)
  console.log()

  console.log('  THINKING GRAPH (Derivation Chain):')
  console.log('  ----------------------------------')
  if (result.thinkingGraph?.derivationChain?.length > 0) {
    for (const step of result.thinkingGraph.derivationChain) {
      console.log(`    Step ${step.step}: [${step.rule}] ${step.conclusion}`)
    }
  } else {
    console.log('    (Observations recorded, awaiting next deduce cycle)')
  }
  console.log()

  console.log('  CRYPTOGRAPHIC PROOFS:')
  console.log('  ---------------------')
  if (result.proofs?.length > 0) {
    for (const proof of result.proofs.slice(0, 5)) {
      console.log(`    Proof ${proof.hash?.substring(0, 12) || proof.id?.substring(0, 12)}...`)
      console.log(`      Confidence: ${(proof.confidence * 100).toFixed(1)}%`)
    }
  } else {
    console.log('    (Proofs generated on deduce() - see derived facts)')
  }
  console.log()

  console.log('  DERIVED FACTS:')
  console.log('  --------------')
  if (result.derivedFacts?.length > 0) {
    for (const fact of result.derivedFacts.slice(0, 5)) {
      console.log(`    - ${fact.predicate}(${fact.args?.join(', ') || fact.subject})`)
    }
  }
  console.log()

  console.log(`  Observations recorded: ${result.observationCount || 0}`)
  console.log()

  // =========================================================================
  // STEP 6: Federated Query using IN-MEMORY RpcFederationProxy
  // =========================================================================

  console.log('+------------------------------------------------------------------------+')
  console.log('|  STEP 6: Federated Query (IN-MEMORY WASM via NAPI-RS)                  |')
  console.log('+------------------------------------------------------------------------+')
  console.log()

  // Demonstrate IN-MEMORY federation.query() with graph_search()
  // This executes SPARQL via NAPI-RS native binding - NO remote server needed!
  const fedQuery = `
    SELECT kg.s, kg.p, kg.o
    FROM graph_search('SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10') kg
    JOIN snowflake.TPCH_SF1.CUSTOMER sf ON kg.s = sf.C_NAME
  `

  console.log('  Executing federated query via IN-MEMORY mode...')
  console.log('  (NAPI-RS native binding - no external server required)')
  console.log()

  const fedResult = await federation.query(fedQuery, { limit: 10 })

  console.log('  FEDERATION RESULT:')
  console.log('  -----------------')
  console.log(`    Mode: ${fedResult.metadata.mode}`)
  console.log(`    Sources: ${fedResult.metadata.sources.map(s => s.type + '(' + s.mode + ')').join(', ')}`)
  console.log(`    Rows: ${fedResult.rowCount}`)
  console.log(`    Duration: ${fedResult.duration}ms`)
  console.log()

  if (fedResult.columns.length > 0) {
    console.log('  COLUMNS:', fedResult.columns.join(', '))
    console.log('  ROWS (sample):')
    for (const row of fedResult.rows.slice(0, 5)) {
      console.log('    ', row)
    }
  }
  console.log()

  // Show direct KG query - all triples pattern works
  console.log('  DIRECT KG QUERY (all triples pattern):')
  const kgTriples = db.querySelect('SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10')
  console.log(`    Found ${kgTriples.length} triples in KGDB (in-memory NAPI-RS)`)
  for (const t of kgTriples.slice(0, 5)) {
    console.log(`      ${t.bindings.s} ${t.bindings.p} ${t.bindings.o}`)
  }
  console.log()

  // =========================================================================
  // STEP 7: Manual ThinkingReasoner Demo
  // =========================================================================

  console.log('+------------------------------------------------------------------------+')
  console.log('|  STEP 7: ThinkingReasoner Direct API Demo                              |')
  console.log('+------------------------------------------------------------------------+')
  console.log()

  // Create fresh ThinkingReasoner for demo
  const reasoner = new ThinkingReasoner()

  console.log('  [1] Loading ontology...')
  const ruleCount = reasoner.loadOntology(BRAIN_ONTOLOGY)
  console.log(`      Auto-generated ${ruleCount} rules from OWL properties`)
  console.log()

  console.log('  [2] Recording observations (via appendEvent)...')
  reasoner.appendEvent('Observation', 'Alice transfers $10K to Bob', 'fraud-agent', 'session-1')
  reasoner.appendEvent('Observation', 'Bob transfers $9.5K to Carol', 'fraud-agent', 'session-1')
  reasoner.appendEvent('Observation', 'Carol transfers $9K to Alice', 'fraud-agent', 'session-1')
  console.log('      Recorded 3 transfer observations')

  console.log('  [2b] Recording hypotheses (via hypothesize)...')
  reasoner.hypothesize('alice', 'transfers', 'bob', 0.9, [])
  reasoner.hypothesize('bob', 'transfers', 'carol', 0.9, [])
  reasoner.hypothesize('carol', 'transfers', 'alice', 0.9, [])
  console.log('      Recorded 3 transfer hypotheses')
  console.log()

  console.log('  [3] Running deductive reasoning...')
  const deductionJson = reasoner.deduce()
  const deduction = JSON.parse(deductionJson)
  console.log(`      Rules fired: ${deduction.rules_fired || 0}`)
  console.log(`      Derived facts: ${deduction.derived_facts?.length || 0}`)
  console.log(`      Proofs generated: ${deduction.proofs?.length || 0}`)
  console.log()

  if (deduction.derived_facts?.length > 0) {
    console.log('  DERIVED FACTS:')
    for (const fact of deduction.derived_facts.slice(0, 5)) {
      console.log(`    - ${fact.predicate}: ${fact.subject} -> ${fact.object} (${(fact.confidence * 100).toFixed(0)}%)`)
    }
    console.log()
  }

  if (deduction.proofs?.length > 0) {
    console.log('  PROOFS (cryptographic hashes):')
    for (const proof of deduction.proofs.slice(0, 3)) {
      console.log(`    - Proof ${proof.hash?.substring(0, 16)}... confidence=${(proof.confidence * 100).toFixed(0)}%`)
    }
    console.log()
  }

  console.log('  [4] Getting thinking graph...')
  const graphJson = reasoner.getThinkingGraph()
  const graph = JSON.parse(graphJson)
  console.log(`      Nodes: ${graph.nodes?.length || 0}`)
  console.log(`      Edges: ${graph.edges?.length || 0}`)
  console.log(`      Derivation steps: ${graph.derivation_chain?.length || 0}`)
  console.log()

  if (graph.derivation_chain?.length > 0) {
    console.log('  DERIVATION CHAIN:')
    for (const step of graph.derivation_chain) {
      console.log(`    Step ${step.step}: [${step.rule}] ${step.conclusion}`)
    }
    console.log()
  }

  console.log('  [5] Reasoning stats:')
  const reasonerStatsJson = reasoner.getStats()
  const reasonerStats = JSON.parse(reasonerStatsJson)
  console.log(`      Events: ${reasonerStats.events}`)
  console.log(`      Facts: ${reasonerStats.facts}`)
  console.log(`      Rules: ${reasonerStats.rules}`)
  console.log(`      Proofs: ${reasonerStats.proofs}`)
  console.log()

  // =========================================================================
  // FINAL SUMMARY
  // =========================================================================

  const duration = Date.now() - startTime

  console.log('='.repeat(80))
  console.log('  DEMONSTRATION COMPLETE')
  console.log('='.repeat(80))
  console.log()
  console.log('  Summary:')
  console.log(`    - Loaded BRAIN ontology (${tripleCount} triples)`)
  console.log(`    - Auto-generated ${stats.rules} Datalog rules from OWL properties`)
  console.log('    - HyperMindAgent with AUTOMATIC deductive reasoning')
  console.log('    - Federated KGDB + Snowflake TPCH query')
  console.log('    - Cryptographic proofs with derivation chain')
  console.log()
  console.log(`  Total Runtime: ${duration}ms`)
  console.log()
  console.log('  KEY INSIGHT:')
  console.log('  ------------')
  console.log('  HyperMindAgent + ThinkingReasoner = AI that PROVES its conclusions')
  console.log('  OWL properties -> Datalog rules -> Cryptographic proofs')
  console.log()
  console.log('='.repeat(80))
}

// Run demonstration
main().catch(err => {
  console.error('Demonstration failed:', err)
  process.exit(1)
})
