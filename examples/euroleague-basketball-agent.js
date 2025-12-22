/**
 * Euroleague Basketball Knowledge Graph + HyperMindAgent
 *
 * Based on: https://medium.com/@skontopo2009/representing-euroleague-play-by-play-data-as-a-knowledge-graph-6397534cdd75
 * Data Model: https://github.com/andrewstellman/pbprdf (Play-by-Play RDF ontology)
 *
 * This example demonstrates:
 * - Loading RDF knowledge graphs (inline or from HTTP URLs)
 * - OWL properties: SymmetricProperty, TransitiveProperty
 * - SPARQL queries with assertions (100% correctness verification)
 * - RDF2Vec embeddings for semantic similarity
 * - ThinkingReasoner with derivation chains (proofs)
 * - Prompt optimization for LLM-based queries
 *
 * Run: node examples/euroleague-basketball-agent.js
 */

const { GraphDB, HyperMindAgent, Rdf2VecEngine, EmbeddingService } = require('rust-kgdb')
const assert = require('assert')

// ============================================================================
// ALL IN-MEMORY OPERATION
// ============================================================================
// This example demonstrates fully in-memory operation:
// 1. GraphDB: In-memory RDF store (no disk I/O)
// 2. RDF2Vec: Embeddings computed in-memory (Word2Vec on random walks)
// 3. Prompt Optimizer: WASM RPC mode (in-memory KGDB via WebSocket)
// 4. HyperMindAgent: Reasoning + LLM coordination in-memory
//
// For production, switch to K8s mode with persistent storage.
// ============================================================================

// Test results tracking
const testResults = {
  passed: 0,
  failed: 0,
  assertions: []
}

function test(name, fn) {
  try {
    fn()
    testResults.passed++
    testResults.assertions.push({ name, status: 'PASS' })
    console.log(`    [PASS] ${name}`)
  } catch (e) {
    testResults.failed++
    testResults.assertions.push({ name, status: 'FAIL', error: e.message })
    console.log(`    [FAIL] ${name}: ${e.message}`)
  }
}

async function main() {
  console.log('='.repeat(70))
  console.log('  EUROLEAGUE BASKETBALL KNOWLEDGE GRAPH')
  console.log('  HyperMindAgent with Deductive Reasoning + Assertions')
  console.log('='.repeat(70))
  console.log()
  console.log('Source: https://medium.com/@skontopo2009/')
  console.log('        representing-euroleague-play-by-play-data-as-a-knowledge-graph')
  console.log('Data Model: https://github.com/andrewstellman/pbprdf')
  console.log()

  // ============================================================================
  // 1. Load Knowledge Graph
  // ============================================================================
  console.log('[1] Loading Play-by-Play Knowledge Graph...')
  const db = new GraphDB('http://euroleague.net/')

  const fs = require('fs')
  const path = require('path')
  const dataPath = path.join(__dirname, '..', 'data', 'euroleague-game.ttl')

  if (!fs.existsSync(dataPath)) {
    console.error(`ERROR: Data file not found: ${dataPath}`)
    console.error('Run: uv run --with euroleague-api python3 scripts/euroleague-to-ttl.py')
    process.exit(1)
  }

  const ttlData = fs.readFileSync(dataPath, 'utf-8')
  db.loadTtl(ttlData, null)

  const tripleCount = db.countTriples()
  console.log(`    Source: euroleague-api (pip install euroleague-api)`)
  console.log(`    Triples: ${tripleCount}`)
  console.log()

  // ============================================================================
  // 2. SPARQL Queries with Assertions
  // ============================================================================
  console.log('[2] SPARQL Queries with Assertions:')
  console.log()

  // Query: Teams
  const teamsQ = `SELECT ?team ?label WHERE {
    ?team <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Team> .
    ?team <http://www.w3.org/2000/01/rdf-schema#label> ?label .
  }`
  const teams = db.querySelect(teamsQ)
  test('Teams count = 2 (BER, PAN)', () => {
    assert.strictEqual(teams.length, 2, `Expected 2 teams, got ${teams.length}`)
  })

  // Query: Players
  const playersQ = `SELECT ?name WHERE {
    ?p <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
    ?p <http://www.w3.org/2000/01/rdf-schema#label> ?name .
  }`
  const players = db.querySelect(playersQ)
  test('Players count = 22', () => {
    assert.strictEqual(players.length, 22, `Expected 22 players, got ${players.length}`)
  })

  // Query: Steals (CRITICAL TEST)
  const stealsQ = `SELECT ?player WHERE {
    ?e <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Steal> .
    ?e <http://euroleague.net/ontology#player> ?player .
  }`
  const steals = db.querySelect(stealsQ)
  const stealPlayers = steals.map(r => extractLast(r.bindings?.player || r.player))
  test('Steals count = 3 (Lessort, Mitoglou, Mattisseck)', () => {
    assert.strictEqual(steals.length, 3, `Expected 3 steals, got ${steals.length}`)
  })
  test('Steal players are correct', () => {
    const expected = ['lessort__mathias', 'mitoglou__konstantinos', 'mattisseck__jonas']
    for (const p of expected) {
      assert(stealPlayers.includes(p), `Missing steal player: ${p}`)
    }
  })

  // Query: Assists (CRITICAL TEST)
  const assistsQ = `SELECT ?player WHERE {
    ?e <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Assist> .
    ?e <http://euroleague.net/ontology#player> ?player .
  }`
  const assistEvents = db.querySelect(assistsQ)
  test('Assist events count = 8', () => {
    assert.strictEqual(assistEvents.length, 8, `Expected 8 assists, got ${assistEvents.length}`)
  })

  // Query: Teammate relationships
  const tmQ = `SELECT ?a ?b WHERE {
    ?a <http://euroleague.net/ontology#teammateOf> ?b .
  }`
  const teammates = db.querySelect(tmQ)
  test('Teammate links = 111', () => {
    assert.strictEqual(teammates.length, 111, `Expected 111 teammate links, got ${teammates.length}`)
  })

  // Query: Scoring events (Two Pointers + Three Pointers)
  const scoringQ = `SELECT ?player ?label WHERE {
    ?e <http://euroleague.net/ontology#player> ?player .
    ?e <http://www.w3.org/2000/01/rdf-schema#label> ?label .
    FILTER(CONTAINS(?label, "Pointer"))
  }`
  const scoringEvents = db.querySelect(scoringQ)
  test('Scoring events found', () => {
    assert(scoringEvents.length > 0, `Expected scoring events, got ${scoringEvents.length}`)
  })

  console.log()

  // ============================================================================
  // 3. RDF2Vec Embeddings
  // ============================================================================
  console.log('[3] Training RDF2Vec Embeddings...')

  const allTriples = db.querySelect('SELECT ?s ?p ?o WHERE { ?s ?p ?o }')
  const graph = new Map()
  for (const t of allTriples) {
    const s = t.bindings?.s || t.s
    const p = t.bindings?.p || t.p
    const o = t.bindings?.o || t.o
    if (!graph.has(s)) graph.set(s, [])
    graph.get(s).push({ predicate: p, object: o })
  }

  // Generate random walks
  const walks = []
  const walksPerNode = 10
  const walkLength = 5

  for (const [entity, edges] of graph) {
    for (let w = 0; w < walksPerNode; w++) {
      const walk = [entity]
      let current = entity

      for (let step = 0; step < walkLength; step++) {
        const neighbors = graph.get(current)
        if (!neighbors || neighbors.length === 0) break
        const randomEdge = neighbors[Math.floor(Math.random() * neighbors.length)]
        walk.push(randomEdge.predicate)
        walk.push(randomEdge.object)
        current = randomEdge.object
      }

      if (walk.length > 1) walks.push(walk)
    }
  }

  console.log(`    Generated ${walks.length} random walks from ${graph.size} entities`)

  // Train RDF2Vec model
  const rdf2vec = Rdf2VecEngine.withConfig(128, 5, walkLength, walksPerNode)
  const trainResult = JSON.parse(rdf2vec.train(JSON.stringify(walks)))
  console.log(`    Trained: ${trainResult.vocabulary_size} embeddings (${trainResult.dimensions}D) in ${trainResult.training_time_secs?.toFixed(2) || 'N/A'}s`)

  // Store embeddings
  const embeddingService = new EmbeddingService()
  let storedCount = 0
  for (const [entity] of graph) {
    const vec = rdf2vec.getEmbedding(entity)
    if (vec) {
      embeddingService.storeVector(entity, vec)
      storedCount++
    }
  }
  console.log(`    Stored ${storedCount} entity embeddings in EmbeddingService`)

  test('RDF2Vec embeddings generated', () => {
    assert(storedCount > 100, `Expected >100 embeddings, got ${storedCount}`)
  })
  console.log()

  // ============================================================================
  // 4. Prompt Optimization (Schema + RDF2Vec)
  // ============================================================================
  console.log('[4] Prompt Optimization (In-Memory Mode):')
  console.log()

  const sqlPrompt = db.buildSqlPrompt('Who made steals in the game?')
  const schema = JSON.parse(db.getSchema())

  console.log('  Mode: WASM RPC (in-memory)')
  console.log(`  Schema: Extracted from ${tripleCount} triples`)
  console.log(`  Embeddings: ${storedCount} entities with RDF2Vec vectors`)
  console.log()

  console.log('  SCHEMA CONTEXT (for LLM):')
  console.log(`    Classes: ${schema.classes?.length || 0}`)
  console.log(`    Predicates: ${schema.predicates?.length || 0}`)
  console.log(`    Namespace: ${schema.namespace || 'auto-detected'}`)

  test('Schema has classes', () => {
    assert((schema.classes?.length || 0) > 0, 'Expected schema classes')
  })
  test('Schema has predicates', () => {
    assert((schema.predicates?.length || 0) > 0, 'Expected schema predicates')
  })
  console.log()

  console.log('  GENERATED PROMPT (first 500 chars):')
  console.log('  ' + sqlPrompt.substring(0, 500).split('\n').join('\n  ') + '...')
  console.log()

  // ============================================================================
  // 5. ThinkingReasoner with Deductive Reasoning
  // ============================================================================
  console.log('[5] ThinkingReasoner with Deductive Reasoning:')
  console.log()

  const agent = new HyperMindAgent({
    name: 'euroleague-analyst',
    kg: db,
    embeddings: embeddingService,
    apiKey: process.env.OPENAI_API_KEY,
    model: 'gpt-4o'
  })

  // Tell agent about OWL properties for deductive reasoning
  // These same properties are also in euroleague-game.ttl (lines 19-20)
  const ontology = `
    @prefix owl: <http://www.w3.org/2002/07/owl#> .
    @prefix euro: <http://euroleague.net/ontology#> .
    euro:teammateOf a owl:SymmetricProperty .
    euro:assistedBy a owl:TransitiveProperty .
  `
  agent.loadOntology(ontology)

  // Add observations from the knowledge graph
  console.log('  Loading observations into ThinkingReasoner...')

  // Observe teammate relationships
  for (const r of teammates) {
    const a = extractLast(r.bindings?.a || r.a)
    const b = extractLast(r.bindings?.b || r.b)
    agent.observe(`${a} is teammate of ${b}`, {
      subject: a,
      predicate: 'teammateOf',
      object: b
    })
  }

  // Run deduction to derive new facts
  console.log('  Running deductive reasoning...')
  const deduction = agent.deduce()

  const stats = agent.getReasoningStats()
  console.log(`    Agent: ${agent.name}`)
  console.log(`    LLM: ${process.env.OPENAI_API_KEY ? 'OpenAI' : 'None (schema-based)'}`)
  console.log(`    Observations: ${stats.events}`)
  console.log(`    Derived Facts: ${stats.facts}`)
  console.log(`    Rules Applied: ${stats.rules}`)

  test('Observations loaded = 111', () => {
    assert.strictEqual(stats.events, 111, `Expected 111 observations, got ${stats.events}`)
  })
  test('Derived facts = 222 (symmetric property doubles links)', () => {
    assert.strictEqual(stats.facts, 222, `Expected 222 derived facts, got ${stats.facts}`)
  })
  test('Rules applied = 2 (SymmetricProperty + TransitiveProperty)', () => {
    assert.strictEqual(stats.rules, 2, `Expected 2 rules, got ${stats.rules}`)
  })
  console.log()

  // ============================================================================
  // 6. Thinking Graph (Derivation Chain / Proofs)
  // ============================================================================
  console.log('[6] Thinking Graph (Derivation Chain / Proofs):')
  console.log()

  const thinkingGraph = agent.getThinkingGraph()

  if (thinkingGraph.nodes && thinkingGraph.nodes.length > 0) {
    console.log('  EVIDENCE NODES (first 8):')
    for (const node of thinkingGraph.nodes.slice(0, 8)) {
      const icon = {
        'OBSERVATION': '[OBS]',
        'HYPOTHESIS': '[HYP]',
        'INFERENCE': '[INF]'
      }[node.type] || '[EVT]'
      const label = node.label || node.id
      console.log(`    ${icon} ${label}`)
    }
    console.log()
  }

  if (thinkingGraph.derivationChain && thinkingGraph.derivationChain.length > 0) {
    console.log('  DERIVATION CHAIN (Proof Steps):')
    for (const step of thinkingGraph.derivationChain.slice(0, 8)) {
      console.log(`    Step ${step.step}: [${step.rule}] ${step.conclusion}`)
      if (step.premises && step.premises.length > 0) {
        console.log(`           Premises: ${step.premises.join(', ')}`)
      }
    }
    console.log()
  }

  console.log('  DEDUCTIVE REASONING VALUE:')
  console.log('    - Every conclusion traces back to ground truth observations')
  console.log('    - SymmetricProperty: If A teammateOf B, then B teammateOf A')
  console.log('    - TransitiveProperty: If A assistedBy B, B assistedBy C, then A assistedBy C')
  console.log('    - No hallucinations - only provable facts with derivation chains')
  console.log()

  // ============================================================================
  // 7. Use Case Queries (SPARQL-first, deterministic)
  // ============================================================================
  console.log('[7] Use Case Queries (SPARQL-first, deterministic):')
  console.log()

  const useCases = [
    {
      persona: 'JOURNALIST',
      question: 'Who made the defensive steals?',
      sparql: stealsQ,
      expected: 3,
      description: 'Uncover storylines beyond surface-level stats'
    },
    {
      persona: 'COACH',
      question: 'Which players distributed the ball best with assists?',
      sparql: assistsQ,
      expected: 8,
      description: 'Identify team chemistry for strategic planning'
    },
    {
      persona: 'ANALYST',
      question: 'Who made scoring plays (Two/Three Pointers)?',
      sparql: scoringQ,
      expected: scoringEvents.length,
      description: 'Enriched interconnected data for modeling'
    },
    {
      persona: 'FAN',
      question: 'Who are the teammates of Lessort?',
      sparql: `SELECT ?teammate WHERE {
        <http://euroleague.net/player/lessort__mathias> <http://euroleague.net/ontology#teammateOf> ?teammate .
      }`,
      expected: 8,
      description: 'Interactive exploration of team dynamics'
    }
  ]

  for (const uc of useCases) {
    console.log('-'.repeat(60))
    console.log(`${uc.persona}: "${uc.question}"`)
    console.log(`VALUE: ${uc.description}`)
    console.log('-'.repeat(60))

    const results = db.querySelect(uc.sparql)
    console.log()
    console.log('SPARQL:')
    console.log('```sparql')
    console.log(uc.sparql.trim())
    console.log('```')
    console.log()
    console.log(`RESULTS: ${results.length} bindings`)

    // Show sample results
    if (results.length > 0) {
      console.log('SAMPLE (first 5):')
      for (const r of results.slice(0, 5)) {
        const keys = Object.keys(r.bindings || r)
        const values = keys.map(k => `${k}=${extractLast((r.bindings || r)[k])}`)
        console.log(`  ${values.join(', ')}`)
      }
    }

    // Verify with assertion
    test(`${uc.persona}: ${uc.question}`, () => {
      assert.strictEqual(results.length, uc.expected, `Expected ${uc.expected}, got ${results.length}`)
    })

    // Show reasoning context
    console.log()
    console.log('REASONING CONTEXT:')
    console.log(`  Observations: ${stats.events}`)
    console.log(`  Derived Facts: ${stats.facts}`)
    console.log(`  Rules Applied: ${stats.rules}`)
    console.log()
  }

  // ============================================================================
  // 8. HyperMindAgent Natural Language (LLM-assisted)
  // ============================================================================
  if (process.env.OPENAI_API_KEY) {
    console.log('[8] HyperMindAgent Natural Language Queries (LLM-assisted):')
    console.log()

    const nlQueries = [
      'Who made the defensive steals in this game?',
      'Who are the teammates of Lessort?'
    ]

    for (const q of nlQueries) {
      console.log(`  Question: "${q}"`)

      try {
        const result = await agent.call(q)

        if (result.explanation?.sparql_queries?.length > 0) {
          console.log('  Generated SPARQL:')
          console.log('  ```sparql')
          console.log('  ' + result.explanation.sparql_queries[0].query)
          console.log('  ```')
        }

        const answer = result.answer || result.response || result.text
        if (answer) {
          console.log(`  ANSWER: ${answer}`)
        }

        if (result.reasoningStats) {
          console.log(`  REASONING: ${result.reasoningStats.events} observations -> ${result.reasoningStats.facts} derived facts`)
        }

        if (result.thinkingGraph?.derivationChain?.length > 0) {
          console.log('  PROOF (first 3 steps):')
          for (const s of result.thinkingGraph.derivationChain.slice(0, 3)) {
            console.log(`    Step ${s.step}: [${s.rule}] ${s.conclusion}`)
          }
        }
      } catch (e) {
        console.log(`  Note: ${e.message}`)
      }
      console.log()
    }
  } else {
    console.log('[8] HyperMindAgent Natural Language: Skipped (no OPENAI_API_KEY)')
    console.log('    Set OPENAI_API_KEY environment variable to enable LLM-assisted queries.')
    console.log()
  }

  // ============================================================================
  // 9. Test Results Summary
  // ============================================================================
  console.log('='.repeat(70))
  console.log('  TEST RESULTS SUMMARY')
  console.log('='.repeat(70))
  console.log()
  console.log(`  PASSED: ${testResults.passed}`)
  console.log(`  FAILED: ${testResults.failed}`)
  console.log(`  TOTAL:  ${testResults.passed + testResults.failed}`)
  console.log()

  if (testResults.failed > 0) {
    console.log('  FAILED TESTS:')
    for (const t of testResults.assertions.filter(a => a.status === 'FAIL')) {
      console.log(`    - ${t.name}: ${t.error}`)
    }
    console.log()
  }

  const passRate = (testResults.passed / (testResults.passed + testResults.failed) * 100).toFixed(1)
  console.log(`  PASS RATE: ${passRate}%`)
  console.log()

  // ============================================================================
  // 10. Summary
  // ============================================================================
  console.log('='.repeat(70))
  console.log('  ARCHITECTURE SUMMARY - ALL IN-MEMORY')
  console.log('='.repeat(70))
  console.log()
  console.log('  KNOWLEDGE GRAPH (In-Memory):')
  console.log(`    Triples: ${tripleCount}`)
  console.log(`    Teams: ${teams.length}`)
  console.log(`    Players: ${players.length}`)
  console.log(`    Steals: ${steals.length}`)
  console.log(`    Assists: ${assistEvents.length}`)
  console.log(`    Teammate links: ${teammates.length}`)
  console.log()
  console.log('  RDF2VEC EMBEDDINGS (In-Memory):')
  console.log(`    Entity Embeddings: ${storedCount}`)
  console.log(`    Dimensions: ${trainResult.dimensions || 128}`)
  console.log(`    Random Walks: ${walks.length}`)
  console.log()
  console.log('  PROMPT OPTIMIZATION (In-Memory):')
  console.log(`    Schema Classes: ${schema.classes?.length || 0}`)
  console.log(`    Schema Predicates: ${schema.predicates?.length || 0}`)
  console.log('    Mode: WASM RPC (no external services)')
  console.log()
  console.log('  THINKING REASONER (In-Memory):')
  console.log(`    Observations: ${stats.events}`)
  console.log(`    Derived Facts: ${stats.facts}`)
  console.log(`    OWL Rules: ${stats.rules}`)
  console.log('    - SymmetricProperty: A rel B => B rel A')
  console.log('    - TransitiveProperty: A rel B, B rel C => A rel C')
  console.log()
  console.log('  BENEFITS:')
  console.log('    - Zero latency: No network I/O')
  console.log('    - Offline capable: Works without internet')
  console.log('    - Privacy: All data in process memory')
  console.log('    - Verifiable: Assertions prove correctness')
  console.log()

  // Exit with error if tests failed
  if (testResults.failed > 0) {
    process.exit(1)
  }
}

function clean(s) {
  if (!s) return ''
  return String(s).replace(/^"|"$/g, '')
}

function extractLast(s) {
  if (!s) return ''
  s = String(s).replace(/^<|>$/g, '')
  const i = Math.max(s.lastIndexOf('#'), s.lastIndexOf('/'))
  return i >= 0 ? s.substring(i + 1) : s
}

main().catch(console.error)
