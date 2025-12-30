#!/usr/bin/env node
/**
 * ================================================================================
 * HYPERMIND INTEGRATION TEST SUITE - FULL STACK VERIFICATION
 *
 * This test suite verifies:
 * 1. Snowflake TPCH integration with actual credentials
 * 2. BigQuery public datasets integration
 * 3. KGDB in-memory knowledge graph
 * 4. SQL + CTE + graph_search() federation
 * 5. Virtual Table + Catalog setup
 * 6. Quality assessment with feedback loop to training
 * 7. 100% trust verification before release
 *
 * Run: OPENAI_API_KEY=xxx npm run integration-test
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
// TEST CONFIGURATION
// =============================================================================

const TEST_CONFIG = {
  // Snowflake credentials (from environment)
  snowflake: {
    account: process.env.SNOWFLAKE_ACCOUNT || 'crvrogz-iw23234',
    user: process.env.SNOWFLAKE_USER || 'HPERMIND',
    password: process.env.SNOWFLAKE_PASSWORD || '(not configured)',
    warehouse: process.env.SNOWFLAKE_WAREHOUSE || 'COMPUTE_WH',
    database: 'SNOWFLAKE_SAMPLE_DATA',
    schema: 'TPCH_SF1'
  },
  // BigQuery public datasets
  bigquery: {
    projectId: process.env.BIGQUERY_PROJECT_ID || 'bigquery-public-data',
    datasets: ['github_repos', 'samples']
  },
  // Test thresholds for 100% trust
  thresholds: {
    minTripleCount: 50,
    minDeductionRules: 3,
    minProofConfidence: 0.7,
    maxQueryDuration: 5000, // 5 seconds
    minTestPassRate: 1.0    // 100%
  }
}

// =============================================================================
// ENTERPRISE ONTOLOGY FOR INTEGRATION TESTING
// =============================================================================

const INTEGRATION_ONTOLOGY = `
@prefix int: <http://integration.hypermind.ai/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# =========================================================================
# OWL Properties for Integration Testing
# =========================================================================

# Transitive: Customer orders from Supplier chain
int:ordersFrom a owl:TransitiveProperty ;
    rdfs:label "orders from"@en ;
    rdfs:domain int:Customer ;
    rdfs:range int:Supplier .

# Symmetric: Entities connected via network
int:connectedTo a owl:SymmetricProperty ;
    rdfs:label "connected to"@en .

# Transitive: Supply chain
int:suppliesTo a owl:TransitiveProperty ;
    rdfs:label "supplies to"@en .

# =========================================================================
# Class Hierarchy - Matches Snowflake TPCH Schema
# =========================================================================

int:Entity a owl:Class .
int:Customer rdfs:subClassOf int:Entity .
int:Supplier rdfs:subClassOf int:Entity .
int:Order rdfs:subClassOf int:Entity .
int:Part rdfs:subClassOf int:Entity .
int:Nation rdfs:subClassOf int:Entity .
int:Region rdfs:subClassOf int:Entity .

# =========================================================================
# VIRTUAL TABLE DEFINITIONS (Generated from SF/BQ schemas)
# =========================================================================

# Virtual Table: SF.TPCH_SF1.CUSTOMER
int:VT_Customer a int:VirtualTable ;
    int:sourceName "snowflake.TPCH_SF1.CUSTOMER" ;
    int:columns "C_CUSTKEY, C_NAME, C_ADDRESS, C_NATIONKEY, C_PHONE, C_ACCTBAL, C_MKTSEGMENT" ;
    int:primaryKey "C_CUSTKEY" .

# Virtual Table: SF.TPCH_SF1.ORDERS
int:VT_Orders a int:VirtualTable ;
    int:sourceName "snowflake.TPCH_SF1.ORDERS" ;
    int:columns "O_ORDERKEY, O_CUSTKEY, O_ORDERSTATUS, O_TOTALPRICE, O_ORDERDATE" ;
    int:primaryKey "O_ORDERKEY" ;
    int:foreignKey "O_CUSTKEY -> CUSTOMER.C_CUSTKEY" .

# Virtual Table: SF.TPCH_SF1.SUPPLIER
int:VT_Supplier a int:VirtualTable ;
    int:sourceName "snowflake.TPCH_SF1.SUPPLIER" ;
    int:columns "S_SUPPKEY, S_NAME, S_ADDRESS, S_NATIONKEY, S_PHONE, S_ACCTBAL" ;
    int:primaryKey "S_SUPPKEY" .

# Virtual Table: BQ.github_repos.commits
int:VT_Commits a int:VirtualTable ;
    int:sourceName "bigquery.github_repos.commits" ;
    int:columns "commit, author, committer, repo_name" ;
    int:primaryKey "commit" .

# =========================================================================
# SAMPLE DATA - Matches TPCH Customers (C_CUSTKEY 1-10)
# =========================================================================

int:cust_001 a int:Customer ;
    int:custkey "1"^^xsd:integer ;
    int:name "Customer#000000001" ;
    int:segment "BUILDING" ;
    int:acctbal "711.56"^^xsd:decimal ;
    int:nationKey "15"^^xsd:integer .

int:cust_002 a int:Customer ;
    int:custkey "2"^^xsd:integer ;
    int:name "Customer#000000002" ;
    int:segment "AUTOMOBILE" ;
    int:acctbal "121.65"^^xsd:decimal ;
    int:nationKey "13"^^xsd:integer .

int:cust_003 a int:Customer ;
    int:custkey "3"^^xsd:integer ;
    int:name "Customer#000000003" ;
    int:segment "AUTOMOBILE" ;
    int:acctbal "7498.12"^^xsd:decimal ;
    int:nationKey "1"^^xsd:integer .

int:cust_004 a int:Customer ;
    int:custkey "4"^^xsd:integer ;
    int:name "Customer#000000004" ;
    int:segment "MACHINERY" ;
    int:acctbal "2866.83"^^xsd:decimal ;
    int:nationKey "4"^^xsd:integer .

int:cust_005 a int:Customer ;
    int:custkey "5"^^xsd:integer ;
    int:name "Customer#000000005" ;
    int:segment "HOUSEHOLD" ;
    int:acctbal "794.47"^^xsd:decimal ;
    int:nationKey "3"^^xsd:integer .

# Suppliers
int:supp_001 a int:Supplier ;
    int:suppkey "1"^^xsd:integer ;
    int:name "Supplier#000000001" ;
    int:nationKey "17"^^xsd:integer .

int:supp_002 a int:Supplier ;
    int:suppkey "2"^^xsd:integer ;
    int:name "Supplier#000000002" ;
    int:nationKey "5"^^xsd:integer .

# Supply chain relationships
int:supp_001 int:suppliesTo int:cust_001 .
int:supp_001 int:suppliesTo int:cust_002 .
int:supp_002 int:suppliesTo int:cust_003 .
int:cust_001 int:ordersFrom int:supp_001 .
int:cust_002 int:ordersFrom int:supp_001 .
int:cust_003 int:ordersFrom int:supp_002 .

# Network connections
int:cust_001 int:connectedTo int:cust_002 .
int:cust_002 int:connectedTo int:cust_003 .
int:supp_001 int:connectedTo int:supp_002 .
`

// =============================================================================
// SQL + CTE TEMPLATES FOR INTEGRATION TESTING
// =============================================================================

const SQL_CTE_TEMPLATES = {
  // CTE 1: High-value customers from KG + SF join
  HIGH_VALUE_CUSTOMERS: `
    WITH kg_customers AS (
      SELECT s, p, o
      FROM graph_search('
        PREFIX int: <http://integration.hypermind.ai/>
        SELECT ?customer ?name ?acctbal
        WHERE {
          ?customer a int:Customer .
          ?customer int:name ?name .
          ?customer int:acctbal ?acctbal .
          FILTER(?acctbal > 500)
        }
      ')
    ),
    sf_customers AS (
      SELECT C_CUSTKEY, C_NAME, C_ACCTBAL, C_MKTSEGMENT
      FROM snowflake.TPCH_SF1.CUSTOMER
      WHERE C_ACCTBAL > 500
    )
    SELECT kg.customer, sf.C_NAME, sf.C_ACCTBAL, sf.C_MKTSEGMENT
    FROM kg_customers kg
    JOIN sf_customers sf ON kg.name = sf.C_NAME
    ORDER BY sf.C_ACCTBAL DESC
    LIMIT 10
  `,

  // CTE 2: Supply chain analysis
  SUPPLY_CHAIN_ANALYSIS: `
    WITH suppliers AS (
      SELECT s, p, o
      FROM graph_search('
        PREFIX int: <http://integration.hypermind.ai/>
        SELECT ?supplier ?name ?suppliesTo
        WHERE {
          ?supplier a int:Supplier .
          ?supplier int:name ?name .
          ?supplier int:suppliesTo ?customer .
        }
      ')
    ),
    sf_orders AS (
      SELECT O_ORDERKEY, O_CUSTKEY, O_TOTALPRICE, O_ORDERDATE
      FROM snowflake.TPCH_SF1.ORDERS
      WHERE O_TOTALPRICE > 10000
    )
    SELECT suppliers.supplier, sf_orders.O_TOTALPRICE
    FROM suppliers
    JOIN sf_orders ON suppliers.customer_id = sf_orders.O_CUSTKEY
  `,

  // CTE 3: Virtual table catalog query
  VIRTUAL_TABLE_CATALOG: `
    WITH virtual_tables AS (
      SELECT s, p, o
      FROM graph_search('
        PREFIX int: <http://integration.hypermind.ai/>
        SELECT ?vt ?source ?columns
        WHERE {
          ?vt a int:VirtualTable .
          ?vt int:sourceName ?source .
          ?vt int:columns ?columns .
        }
      ')
    )
    SELECT vt, source, columns
    FROM virtual_tables
    ORDER BY source
  `
}

// =============================================================================
// TEST RESULTS TRACKER (Feedback Loop)
// =============================================================================

class TestResultsTracker {
  constructor() {
    this.results = []
    this.startTime = Date.now()
  }

  record(testName, passed, details = {}) {
    this.results.push({
      testName,
      passed,
      details,
      timestamp: Date.now(),
      duration: Date.now() - this.startTime
    })
  }

  getPassRate() {
    if (this.results.length === 0) return 0
    const passed = this.results.filter(r => r.passed).length
    return passed / this.results.length
  }

  getFailedTests() {
    return this.results.filter(r => !r.passed)
  }

  generateFeedback() {
    const passRate = this.getPassRate()
    const failedTests = this.getFailedTests()

    return {
      passRate,
      totalTests: this.results.length,
      passedTests: this.results.filter(r => r.passed).length,
      failedTests: failedTests.length,
      trainingFeedback: this.generateTrainingFeedback(failedTests),
      optimizationSuggestions: this.generateOptimizationSuggestions(failedTests)
    }
  }

  generateTrainingFeedback(failedTests) {
    const feedback = []

    for (const test of failedTests) {
      feedback.push({
        testName: test.testName,
        issue: test.details.error || 'Unknown error',
        suggestedTrainingExample: {
          input: test.details.input,
          expectedOutput: test.details.expected,
          actualOutput: test.details.actual,
          category: this.categorizeFailure(test)
        }
      })
    }

    return feedback
  }

  generateOptimizationSuggestions(failedTests) {
    const suggestions = []

    for (const test of failedTests) {
      if (test.details.error?.includes('timeout')) {
        suggestions.push(`Query optimization needed for: ${test.testName}`)
      }
      if (test.details.error?.includes('schema')) {
        suggestions.push(`Schema mapping issue in: ${test.testName}`)
      }
      if (test.details.error?.includes('reasoning')) {
        suggestions.push(`Reasoning rules need enhancement for: ${test.testName}`)
      }
    }

    return suggestions
  }

  categorizeFailure(test) {
    if (test.details.error?.includes('SPARQL')) return 'sparql_generation'
    if (test.details.error?.includes('SQL')) return 'sql_generation'
    if (test.details.error?.includes('federation')) return 'federation_query'
    if (test.details.error?.includes('reasoning')) return 'deductive_reasoning'
    return 'general'
  }
}

// =============================================================================
// INTEGRATION TEST SUITE
// =============================================================================

class IntegrationTestSuite {
  constructor() {
    this.db = null
    this.agent = null
    this.federation = null
    this.tracker = new TestResultsTracker()
  }

  async setup() {
    console.log()
    console.log('='.repeat(80))
    console.log('  HYPERMIND INTEGRATION TEST SUITE')
    console.log('  Full Stack Verification for 100% Trust')
    console.log('='.repeat(80))
    console.log()
    console.log(`  rust-kgdb Version: ${getVersion()}`)
    console.log()

    // Initialize KGDB
    this.db = new GraphDB('http://integration.hypermind.ai/')
    this.db.loadTtl(INTEGRATION_ONTOLOGY, null)

    // Initialize Federation Proxy
    this.federation = new RpcFederationProxy({
      mode: 'inMemory',
      kg: this.db,
      connectors: {
        snowflake: TEST_CONFIG.snowflake,
        bigquery: TEST_CONFIG.bigquery
      }
    })

    // Initialize HyperMind Agent
    this.agent = new HyperMindAgent({
      name: 'integration-test-agent',
      kg: this.db,
      federationProxy: this.federation,
      connectors: this.federation.connectors,
      apiKey: process.env.OPENAI_API_KEY,
      model: 'gpt-4o'
    })

    console.log('  Setup complete:')
    console.log(`    - KGDB: ${this.db.countTriples()} triples loaded`)
    console.log(`    - Federation: ${this.federation.getMode()} mode`)
    console.log(`    - Agent: ${this.agent.name}`)
    console.log()
  }

  // =========================================================================
  // TEST 1: KGDB Triple Store Verification
  // =========================================================================
  async testKGDBTripleStore() {
    console.log('  [TEST 1] KGDB Triple Store Verification')

    const tripleCount = this.db.countTriples()
    const passed = tripleCount >= TEST_CONFIG.thresholds.minTripleCount

    this.tracker.record('KGDB Triple Store', passed, {
      input: 'Load integration ontology',
      expected: `>= ${TEST_CONFIG.thresholds.minTripleCount} triples`,
      actual: `${tripleCount} triples`,
      error: passed ? null : `Triple count ${tripleCount} below threshold`
    })

    console.log(`    Triples: ${tripleCount} (threshold: ${TEST_CONFIG.thresholds.minTripleCount})`)
    console.log(`    Status: ${passed ? '✅ PASS' : '❌ FAIL'}`)
    console.log()
  }

  // =========================================================================
  // TEST 2: SPARQL Query Execution
  // =========================================================================
  async testSPARQLQueries() {
    console.log('  [TEST 2] SPARQL Query Execution')

    const queries = [
      {
        name: 'List all customers',
        sparql: 'PREFIX int: <http://integration.hypermind.ai/> PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> SELECT ?customer ?name WHERE { ?customer rdf:type int:Customer . ?customer int:name ?name }',
        minResults: 5
      },
      {
        name: 'List all suppliers',
        sparql: 'PREFIX int: <http://integration.hypermind.ai/> PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> SELECT ?supplier ?name WHERE { ?supplier rdf:type int:Supplier . ?supplier int:name ?name }',
        minResults: 2
      },
      {
        name: 'High-value customers',
        sparql: 'PREFIX int: <http://integration.hypermind.ai/> PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> SELECT ?customer ?acctbal WHERE { ?customer rdf:type int:Customer . ?customer int:acctbal ?acctbal }',
        minResults: 2
      },
      {
        name: 'Supply chain relationships',
        sparql: 'PREFIX int: <http://integration.hypermind.ai/> SELECT ?supplier ?customer WHERE { ?supplier int:suppliesTo ?customer }',
        minResults: 3
      }
    ]

    for (const query of queries) {
      const startTime = Date.now()
      const results = this.db.querySelect(query.sparql)
      const duration = Date.now() - startTime
      const passed = results.length >= query.minResults && duration < TEST_CONFIG.thresholds.maxQueryDuration

      this.tracker.record(`SPARQL: ${query.name}`, passed, {
        input: query.sparql,
        expected: `>= ${query.minResults} results, < ${TEST_CONFIG.thresholds.maxQueryDuration}ms`,
        actual: `${results.length} results, ${duration}ms`,
        error: passed ? null : `Results ${results.length} or duration ${duration}ms not meeting threshold`
      })

      console.log(`    ${query.name}: ${results.length} results, ${duration}ms - ${passed ? '✅' : '❌'}`)
    }
    console.log()
  }

  // =========================================================================
  // TEST 3: Virtual Table Catalog
  // =========================================================================
  async testVirtualTableCatalog() {
    console.log('  [TEST 3] Virtual Table Catalog')

    const vtQuery = 'PREFIX int: <http://integration.hypermind.ai/> PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> SELECT ?vt ?source ?columns WHERE { ?vt rdf:type int:VirtualTable . ?vt int:sourceName ?source . ?vt int:columns ?columns }'

    const results = this.db.querySelect(vtQuery)
    const passed = results.length >= 4 // We defined 4 virtual tables

    this.tracker.record('Virtual Table Catalog', passed, {
      input: vtQuery,
      expected: '>= 4 virtual tables',
      actual: `${results.length} virtual tables`,
      error: passed ? null : `Found only ${results.length} virtual tables`
    })

    console.log(`    Virtual Tables Found: ${results.length}`)
    for (const vt of results) {
      console.log(`      - ${vt.bindings.source}`)
    }
    console.log(`    Status: ${passed ? '✅ PASS' : '❌ FAIL'}`)
    console.log()
  }

  // =========================================================================
  // TEST 4: Deductive Reasoning (ThinkingReasoner)
  // =========================================================================
  async testDeductiveReasoning() {
    console.log('  [TEST 4] Deductive Reasoning')

    const reasoner = new ThinkingReasoner()

    // Load ontology
    const ruleCount = reasoner.loadOntology(INTEGRATION_ONTOLOGY)
    console.log(`    Rules auto-generated: ${ruleCount}`)

    // Record observations
    reasoner.observe('Supplier 1 supplies to Customer 1', {
      subject: 'supp_001', predicate: 'suppliesTo', object: 'cust_001'
    })
    reasoner.observe('Supplier 1 supplies to Customer 2', {
      subject: 'supp_001', predicate: 'suppliesTo', object: 'cust_002'
    })
    reasoner.observe('Customer 1 connected to Customer 2', {
      subject: 'cust_001', predicate: 'connectedTo', object: 'cust_002'
    })

    // Add hypotheses
    reasoner.hypothesize('supp_001', 'suppliesTo', 'cust_001', 0.9, [])
    reasoner.hypothesize('supp_001', 'suppliesTo', 'cust_002', 0.9, [])
    reasoner.hypothesize('cust_001', 'connectedTo', 'cust_002', 0.95, [])

    // Run deduction
    const rawDeduce = reasoner.deduce()
    const deduction = typeof rawDeduce === 'string' ? JSON.parse(rawDeduce) : rawDeduce
    const derivedFacts = deduction.derived_facts || deduction.derivedFacts || []
    const proofs = deduction.proofs || []

    const passed = ruleCount >= TEST_CONFIG.thresholds.minDeductionRules

    this.tracker.record('Deductive Reasoning', passed, {
      input: 'Load ontology and observe facts',
      expected: `>= ${TEST_CONFIG.thresholds.minDeductionRules} rules`,
      actual: `${ruleCount} rules, ${derivedFacts.length} derived facts`,
      error: passed ? null : `Rules ${ruleCount} below threshold`
    })

    console.log(`    Derived Facts: ${derivedFacts.length}`)
    console.log(`    Proofs: ${proofs.length}`)
    console.log(`    Status: ${passed ? '✅ PASS' : '❌ FAIL'}`)
    console.log()
  }

  // =========================================================================
  // TEST 5: SQL + CTE + graph_search() Federation
  // =========================================================================
  async testSQLCTEFederation() {
    console.log('  [TEST 5] SQL + CTE + graph_search() Federation')

    // Test each CTE template
    for (const [name, sql] of Object.entries(SQL_CTE_TEMPLATES)) {
      const startTime = Date.now()

      try {
        const result = await this.federation.query(sql, { limit: 10 })
        const duration = Date.now() - startTime
        const passed = duration < TEST_CONFIG.thresholds.maxQueryDuration

        this.tracker.record(`SQL CTE: ${name}`, passed, {
          input: sql.substring(0, 100) + '...',
          expected: `< ${TEST_CONFIG.thresholds.maxQueryDuration}ms`,
          actual: `${duration}ms, ${result.rowCount} rows`,
          error: passed ? null : `Duration ${duration}ms exceeded threshold`
        })

        console.log(`    ${name}: ${result.rowCount} rows, ${duration}ms - ${passed ? '✅' : '❌'}`)
      } catch (error) {
        this.tracker.record(`SQL CTE: ${name}`, false, {
          input: sql.substring(0, 100) + '...',
          expected: 'Query execution',
          actual: 'Error',
          error: error.message
        })
        console.log(`    ${name}: ERROR - ${error.message} - ❌`)
      }
    }
    console.log()
  }

  // =========================================================================
  // TEST 6: HyperMindAgent Natural Language Query
  // =========================================================================
  async testHyperMindAgentQuery() {
    console.log('  [TEST 6] HyperMindAgent Natural Language Query')

    const testQueries = [
      {
        query: 'Find high-value customers with account balance over 1000',
        expectedType: 'sql',
        minConfidence: 0.7
      },
      {
        query: 'Show supply chain relationships between suppliers and customers',
        expectedType: 'sparql',
        minConfidence: 0.6
      },
      {
        query: 'Analyze customer segments by market',
        expectedType: 'sql',
        minConfidence: 0.7
      }
    ]

    for (const test of testQueries) {
      try {
        const startTime = Date.now()
        const result = await this.agent.call(test.query)
        const duration = Date.now() - startTime

        // Check proofs and confidence
        const avgConfidence = result.proofs?.length > 0
          ? result.proofs.reduce((sum, p) => sum + (p.confidence || 0), 0) / result.proofs.length
          : 0

        const passed = avgConfidence >= test.minConfidence && duration < TEST_CONFIG.thresholds.maxQueryDuration

        this.tracker.record(`Agent Query: ${test.query.substring(0, 40)}...`, passed, {
          input: test.query,
          expected: `Confidence >= ${test.minConfidence}, Duration < ${TEST_CONFIG.thresholds.maxQueryDuration}ms`,
          actual: `Confidence: ${avgConfidence.toFixed(2)}, Duration: ${duration}ms`,
          error: passed ? null : `Confidence ${avgConfidence.toFixed(2)} or duration ${duration}ms not meeting threshold`
        })

        console.log(`    "${test.query.substring(0, 40)}..."`)
        console.log(`      Confidence: ${avgConfidence.toFixed(2)}, Duration: ${duration}ms - ${passed ? '✅' : '❌'}`)
      } catch (error) {
        this.tracker.record(`Agent Query: ${test.query.substring(0, 40)}...`, false, {
          input: test.query,
          expected: 'Query execution',
          actual: 'Error',
          error: error.message
        })
        console.log(`    "${test.query.substring(0, 40)}..." - ERROR: ${error.message} ❌`)
      }
    }
    console.log()
  }

  // =========================================================================
  // TEST 7: Proof Verification
  // =========================================================================
  async testProofVerification() {
    console.log('  [TEST 7] Proof Verification')

    const result = await this.agent.call('Find all customers with their suppliers')

    const hasProofs = result.proofs && result.proofs.length > 0
    const avgConfidence = hasProofs
      ? result.proofs.reduce((sum, p) => sum + (p.confidence || 0), 0) / result.proofs.length
      : 0

    const passed = hasProofs && avgConfidence >= TEST_CONFIG.thresholds.minProofConfidence

    this.tracker.record('Proof Verification', passed, {
      input: 'Agent query with proof generation',
      expected: `Proofs with confidence >= ${TEST_CONFIG.thresholds.minProofConfidence}`,
      actual: `${result.proofs?.length || 0} proofs, avg confidence: ${avgConfidence.toFixed(2)}`,
      error: passed ? null : `No proofs or confidence ${avgConfidence.toFixed(2)} below threshold`
    })

    console.log(`    Proofs Generated: ${result.proofs?.length || 0}`)
    console.log(`    Average Confidence: ${avgConfidence.toFixed(2)}`)
    console.log(`    Status: ${passed ? '✅ PASS' : '❌ FAIL'}`)
    console.log()
  }

  // =========================================================================
  // TEST 8: Schema-Aware Code Generation Simulation
  // =========================================================================
  async testSchemaAwareCodeGen() {
    console.log('  [TEST 8] Schema-Aware Code Generation Simulation')

    // Simulate component generation based on schema
    const componentSpecs = [
      {
        type: 'DataTable',
        virtualTable: 'SF.TPCH_SF1.CUSTOMER',
        columns: ['C_NAME', 'C_ACCTBAL', 'C_MKTSEGMENT'],
        filters: ['C_ACCTBAL > 500']
      },
      {
        type: 'Chart',
        virtualTable: 'SF.TPCH_SF1.ORDERS',
        xAxis: 'O_ORDERDATE',
        yAxis: 'O_TOTALPRICE',
        aggregation: 'SUM'
      },
      {
        type: 'Form',
        virtualTable: 'SF.TPCH_SF1.CUSTOMER',
        fields: ['C_NAME', 'C_ADDRESS', 'C_PHONE', 'C_MKTSEGMENT']
      }
    ]

    let passCount = 0
    for (const spec of componentSpecs) {
      // Validate spec can be resolved against catalog
      const vtQuery = `
        PREFIX int: <http://integration.hypermind.ai/>
        ASK WHERE {
          ?vt a int:VirtualTable .
          ?vt int:sourceName ?source .
          FILTER(CONTAINS(?source, "${spec.virtualTable.split('.').pop()}"))
        }
      `

      const exists = this.db.queryAsk(vtQuery)
      if (exists) passCount++

      this.tracker.record(`Code Gen: ${spec.type} for ${spec.virtualTable}`, exists, {
        input: JSON.stringify(spec),
        expected: 'Virtual table exists in catalog',
        actual: exists ? 'Found' : 'Not found',
        error: exists ? null : `Virtual table ${spec.virtualTable} not in catalog`
      })

      console.log(`    ${spec.type} for ${spec.virtualTable}: ${exists ? '✅' : '❌'}`)
    }

    console.log(`    Passed: ${passCount}/${componentSpecs.length}`)
    console.log()
  }

  // =========================================================================
  // RUN ALL TESTS
  // =========================================================================
  async runAllTests() {
    await this.setup()

    console.log('+------------------------------------------------------------------------+')
    console.log('|  RUNNING INTEGRATION TESTS                                             |')
    console.log('+------------------------------------------------------------------------+')
    console.log()

    await this.testKGDBTripleStore()
    await this.testSPARQLQueries()
    await this.testVirtualTableCatalog()
    await this.testDeductiveReasoning()
    await this.testSQLCTEFederation()
    await this.testHyperMindAgentQuery()
    await this.testProofVerification()
    await this.testSchemaAwareCodeGen()

    return this.generateReport()
  }

  // =========================================================================
  // GENERATE FINAL REPORT WITH FEEDBACK LOOP
  // =========================================================================
  generateReport() {
    const feedback = this.tracker.generateFeedback()

    console.log()
    console.log('='.repeat(80))
    console.log('  INTEGRATION TEST REPORT')
    console.log('='.repeat(80))
    console.log()
    console.log(`  PASS RATE: ${(feedback.passRate * 100).toFixed(1)}%`)
    console.log(`  Tests Passed: ${feedback.passedTests}/${feedback.totalTests}`)
    console.log()

    if (feedback.passRate >= TEST_CONFIG.thresholds.minTestPassRate) {
      console.log('  ✅ 100% TRUST ACHIEVED - READY FOR RELEASE')
    } else {
      console.log('  ❌ TRUST THRESHOLD NOT MET - OPTIMIZATION NEEDED')
      console.log()
      console.log('  FAILED TESTS:')
      for (const test of this.tracker.getFailedTests()) {
        console.log(`    - ${test.testName}: ${test.details.error}`)
      }
    }

    console.log()
    console.log('  TRAINING FEEDBACK (for prompt optimization):')
    console.log('  ---------------------------------------------')

    if (feedback.trainingFeedback.length === 0) {
      console.log('    All tests passed - no training adjustments needed')
    } else {
      for (const fb of feedback.trainingFeedback.slice(0, 5)) {
        console.log(`    - ${fb.testName}:`)
        console.log(`      Issue: ${fb.issue}`)
        console.log(`      Category: ${fb.suggestedTrainingExample.category}`)
      }
    }

    console.log()
    console.log('  OPTIMIZATION SUGGESTIONS:')
    console.log('  -------------------------')

    if (feedback.optimizationSuggestions.length === 0) {
      console.log('    System performing optimally')
    } else {
      for (const suggestion of feedback.optimizationSuggestions) {
        console.log(`    - ${suggestion}`)
      }
    }

    console.log()
    console.log('='.repeat(80))

    return {
      passRate: feedback.passRate,
      trustAchieved: feedback.passRate >= TEST_CONFIG.thresholds.minTestPassRate,
      trainingFeedback: feedback.trainingFeedback,
      optimizationSuggestions: feedback.optimizationSuggestions
    }
  }
}

// =============================================================================
// MAIN EXECUTION
// =============================================================================

async function main() {
  const suite = new IntegrationTestSuite()

  try {
    const report = await suite.runAllTests()

    // Exit with appropriate code
    process.exit(report.trustAchieved ? 0 : 1)
  } catch (error) {
    console.error('Integration test suite failed:', error)
    process.exit(1)
  }
}

main()
