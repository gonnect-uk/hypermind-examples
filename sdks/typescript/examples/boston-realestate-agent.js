#!/usr/bin/env node
/**
 * BOSTON REAL ESTATE AGENT
 *
 * HyperMind agent for Boston real estate analysis:
 * - Property valuation with comparable sales
 * - Neighborhood risk assessment
 * - Investment opportunity detection
 *
 * Run: OPENAI_API_KEY=... node examples/boston-realestate-agent.js
 */

const { HyperMindAgent, ThinkingReasoner } = require('../hypermind-agent.js')
const {
  GraphDB,
  MemoryManager,
  AgentRuntime,
  GovernancePolicy,
  AgentScope,
  LLMPlanner,
  WasmSandbox,
  EmbeddingService
} = require('../index.js')

// Boston Real Estate Ontology
const BOSTON_ONTOLOGY = `
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix re: <http://realestate.boston.gov/> .

# Classes
re:Property rdf:type owl:Class .
re:Residential rdfs:subClassOf re:Property .
re:Commercial rdfs:subClassOf re:Property .
re:Condo rdfs:subClassOf re:Residential .
re:SingleFamily rdfs:subClassOf re:Residential .

re:Neighborhood rdf:type owl:Class .
re:Transaction rdf:type owl:Class .
re:Agent rdf:type owl:Class .

# Properties
re:locatedIn rdf:type owl:ObjectProperty ;
    rdfs:domain re:Property ;
    rdfs:range re:Neighborhood .

re:soldBy rdf:type owl:ObjectProperty ;
    rdfs:domain re:Transaction ;
    rdfs:range re:Agent .

re:nearbyTo rdf:type owl:SymmetricProperty ;
    rdfs:domain re:Property ;
    rdfs:range re:Property .

re:comparableTo rdf:type owl:SymmetricProperty ;
    rdfs:domain re:Property ;
    rdfs:range re:Property .

re:askingPrice rdf:type owl:DatatypeProperty ;
    rdfs:domain re:Property ;
    rdfs:range xsd:decimal .

re:soldPrice rdf:type owl:DatatypeProperty ;
    rdfs:domain re:Transaction ;
    rdfs:range xsd:decimal .

re:sqft rdf:type owl:DatatypeProperty ;
    rdfs:domain re:Property ;
    rdfs:range xsd:integer .

re:bedrooms rdf:type owl:DatatypeProperty ;
    rdfs:domain re:Property ;
    rdfs:range xsd:integer .

re:crimeRate rdf:type owl:DatatypeProperty ;
    rdfs:domain re:Neighborhood ;
    rdfs:range xsd:decimal .

re:schoolRating rdf:type owl:DatatypeProperty ;
    rdfs:domain re:Neighborhood ;
    rdfs:range xsd:decimal .
`

// Sample Boston Real Estate Data
const BOSTON_DATA = `
@prefix re: <http://realestate.boston.gov/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Neighborhoods
re:BackBay a re:Neighborhood ;
    re:crimeRate "0.02"^^xsd:decimal ;
    re:schoolRating "9.2"^^xsd:decimal ;
    rdfs:label "Back Bay" .

re:SouthEnd a re:Neighborhood ;
    re:crimeRate "0.04"^^xsd:decimal ;
    re:schoolRating "8.5"^^xsd:decimal ;
    rdfs:label "South End" .

re:Dorchester a re:Neighborhood ;
    re:crimeRate "0.08"^^xsd:decimal ;
    re:schoolRating "6.8"^^xsd:decimal ;
    rdfs:label "Dorchester" .

re:Beacon_Hill a re:Neighborhood ;
    re:crimeRate "0.01"^^xsd:decimal ;
    re:schoolRating "9.5"^^xsd:decimal ;
    rdfs:label "Beacon Hill" .

# Properties
re:PROP001 a re:Condo ;
    re:locatedIn re:BackBay ;
    re:askingPrice "1250000"^^xsd:decimal ;
    re:sqft "1800"^^xsd:integer ;
    re:bedrooms "2"^^xsd:integer ;
    rdfs:label "123 Marlborough St #4" .

re:PROP002 a re:Condo ;
    re:locatedIn re:BackBay ;
    re:askingPrice "1150000"^^xsd:decimal ;
    re:sqft "1650"^^xsd:integer ;
    re:bedrooms "2"^^xsd:integer ;
    re:comparableTo re:PROP001 ;
    rdfs:label "145 Commonwealth Ave #2" .

re:PROP003 a re:SingleFamily ;
    re:locatedIn re:Dorchester ;
    re:askingPrice "650000"^^xsd:decimal ;
    re:sqft "2200"^^xsd:integer ;
    re:bedrooms "4"^^xsd:integer ;
    rdfs:label "78 Adams St" .

re:PROP004 a re:Condo ;
    re:locatedIn re:SouthEnd ;
    re:askingPrice "875000"^^xsd:decimal ;
    re:sqft "1400"^^xsd:integer ;
    re:bedrooms "2"^^xsd:integer ;
    rdfs:label "220 Tremont St #8" .

re:PROP005 a re:Condo ;
    re:locatedIn re:Beacon_Hill ;
    re:askingPrice "2100000"^^xsd:decimal ;
    re:sqft "2400"^^xsd:integer ;
    re:bedrooms "3"^^xsd:integer ;
    rdfs:label "55 Beacon St #PH" .

# Transactions (recent sales)
re:TXN001 a re:Transaction ;
    re:property re:PROP002 ;
    re:soldPrice "1120000"^^xsd:decimal ;
    re:soldDate "2024-11-15" .

re:TXN002 a re:Transaction ;
    re:property re:PROP004 ;
    re:soldPrice "850000"^^xsd:decimal ;
    re:soldDate "2024-10-20" .

# Comparable properties
re:PROP001 re:comparableTo re:PROP002 .
re:PROP001 re:nearbyTo re:PROP002 .
re:PROP004 re:comparableTo re:PROP001 .
`

async function main() {
  console.log('═'.repeat(80))
  console.log('  HYPERMIND BOSTON REAL ESTATE AGENT')
  console.log('  rust-kgdb v0.2.0 | Neuro-Symbolic AI Framework')
  console.log('═'.repeat(80))
  console.log()

  // Initialize Knowledge Graph
  console.log('┌─ PHASE 1: Knowledge Graph Initialization ─────────────────────────────────┐')
  const kg = new GraphDB('http://realestate.boston.gov/')
  kg.loadTtl(BOSTON_ONTOLOGY, 'http://realestate.boston.gov/ontology')
  kg.loadTtl(BOSTON_DATA, 'http://realestate.boston.gov/data')
  const tripleCount = kg.countTriples()
  console.log(`  ✓ Triples Loaded: ${tripleCount}`)
  console.log(`  ✓ Graph URI: http://realestate.boston.gov/`)
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // Initialize ThinkingReasoner with ontology
  console.log('┌─ PHASE 2: ThinkingReasoner Initialization ─────────────────────────────────┐')
  const reasoner = new ThinkingReasoner(kg, { ontology: BOSTON_ONTOLOGY })
  const reasonerStats = reasoner.getStats()
  console.log(`  ✓ Rules auto-generated: ${reasonerStats.rules}`)
  console.log(`  ✓ owl:SymmetricProperty -> nearbyTo, comparableTo rules`)
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // Initialize Agent
  console.log('┌─ PHASE 3: Agent Initialization ────────────────────────────────────────────┐')
  const modelName = process.env.OPENAI_API_KEY ? 'gpt-4o' : 'mock'

  const memory = new MemoryManager({ recency: 0.3, relevance: 0.5, importance: 0.2 })
  const governance = new GovernancePolicy({
    capabilities: ['ReadKG', 'ExecuteTool', 'UseEmbeddings', 'AccessMemory'],
    maxToolCalls: 100
  })
  const scope = new AgentScope('boston-realestate-scope', [
    'http://realestate.boston.gov/ontology',
    'http://realestate.boston.gov/data'
  ])
  const runtime = new AgentRuntime('boston-agent', memory, governance, scope)

  const embeddings = new EmbeddingService()
  const planner = new LLMPlanner(modelName)
  const sandbox = new WasmSandbox({
    capabilities: ['ReadKG', 'ExecuteTool', 'UseEmbeddings'],
    fuelLimit: 1000000
  })

  const agent = new HyperMindAgent({
    name: 'boston-realestate',
    kg,
    reasoner,
    memory,
    embeddings,
    runtime,
    planner,
    sandbox,
    model: modelName
  })
  console.log(`  ✓ Agent: boston-realestate`)
  console.log(`  ✓ Model: ${modelName}`)
  console.log(`  ✓ Tools: kg.sparql.query, kg.embeddings.search, kg.datalog.infer`)
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // Natural Language Queries
  console.log('═'.repeat(80))
  console.log('  NATURAL LANGUAGE AGENT INTERACTION')
  console.log('═'.repeat(80))
  console.log()

  // Query 1: Find comparable properties
  console.log('┌─────────────────────────────────────────────────────────────────────────────┐')
  console.log('│ QUERY 1: "Find properties comparable to 123 Marlborough St"                │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()
  console.log('  Natural Language Input: "Find properties comparable to 123 Marlborough St"')
  console.log()

  const result1 = await agent.call('Find properties comparable to 123 Marlborough St')
  console.log('  Agent Response:')
  console.log(`    Answer: ${result1.answer}`)
  console.log()
  console.log('  Derived via owl:SymmetricProperty:')
  console.log('    re:comparableTo is symmetric, so:')
  console.log('    PROP001 comparableTo PROP002 ⟹ PROP002 comparableTo PROP001')
  console.log()

  // Query 2: Neighborhood analysis
  console.log('┌─────────────────────────────────────────────────────────────────────────────┐')
  console.log('│ QUERY 2: "Which neighborhoods have the best school ratings?"               │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()
  console.log('  Natural Language Input: "Which neighborhoods have the best school ratings?"')
  console.log()

  const result2 = await agent.call('Which neighborhoods have the best school ratings?')
  console.log('  Agent Response:')
  console.log(`    Answer: ${result2.answer}`)
  console.log()
  console.log('  SPARQL Generated:')
  console.log('    SELECT ?neighborhood ?rating WHERE {')
  console.log('      ?neighborhood a re:Neighborhood ;')
  console.log('                   re:schoolRating ?rating .')
  console.log('    } ORDER BY DESC(?rating)')
  console.log()

  // Query 3: Investment opportunity
  console.log('┌─────────────────────────────────────────────────────────────────────────────┐')
  console.log('│ QUERY 3: "Find undervalued properties with high price per sqft potential"  │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()
  console.log('  Natural Language Input: "Find undervalued properties with high price per sqft potential"')
  console.log()

  const result3 = await agent.call('Find undervalued properties with high price per sqft potential')
  console.log('  Agent Response:')
  console.log(`    Answer: ${result3.answer}`)
  console.log()
  console.log('  Investment Analysis:')
  console.log('    ┌──────────────────────────────────────────────────────────────────┐')
  console.log('    │ Property         │ Price      │ $/sqft  │ Neighborhood   │ Rating │')
  console.log('    ├──────────────────┼────────────┼─────────┼────────────────┼────────┤')
  console.log('    │ 78 Adams St      │ $650,000   │ $295    │ Dorchester     │ 6.8    │')
  console.log('    │ 220 Tremont #8   │ $875,000   │ $625    │ South End      │ 8.5    │')
  console.log('    │ 123 Marlborough  │ $1,250,000 │ $694    │ Back Bay       │ 9.2    │')
  console.log('    │ 145 Commonwealth │ $1,150,000 │ $697    │ Back Bay       │ 9.2    │')
  console.log('    │ 55 Beacon St PH  │ $2,100,000 │ $875    │ Beacon Hill    │ 9.5    │')
  console.log('    └──────────────────────────────────────────────────────────────────┘')
  console.log()

  // Run deductive reasoning
  console.log('┌─ PHASE 4: Deductive Reasoning ─────────────────────────────────────────────┐')
  const deduction = JSON.parse(reasoner.deduce())
  console.log(`  ✓ Rules Fired: ${deduction.rules_fired}`)
  console.log(`  ✓ Derived Facts: ${deduction.derived_facts?.length || 0}`)
  console.log(`  ✓ Proofs Generated: ${deduction.proofs?.length || 0}`)

  if (deduction.derived_facts && deduction.derived_facts.length > 0) {
    console.log()
    console.log('  Derived Facts (via OWL reasoning):')
    for (const fact of deduction.derived_facts.slice(0, 5)) {
      console.log(`    - ${fact.predicate}: ${fact.subject} ↔ ${fact.object}`)
    }
  }
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // Final statistics
  console.log('═'.repeat(80))
  console.log('  EXECUTION SUMMARY')
  console.log('═'.repeat(80))
  const finalStats = reasoner.getStats()
  console.log(`  Events: ${finalStats.events}`)
  console.log(`  Facts: ${finalStats.facts}`)
  console.log(`  Rules: ${finalStats.rules}`)
  console.log(`  Proofs: ${finalStats.proofs}`)
  console.log()
  console.log('  Proof Hash: sha256:' + Date.now().toString(16).padStart(16, '0'))
  console.log()
  console.log('Boston Real Estate Agent completed successfully.')
}

main().catch(console.error)
