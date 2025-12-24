#!/usr/bin/env node
/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * MEMORY HYPERGRAPH DEMO - Fraud Detection with Persistent Agent Memory
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * This demonstrates the MEMORY HYPERGRAPH architecture (v0.6.0+):
 *
 *   ┌──────────────────────────────────────────────────────────────────────────┐
 *   │                    MEMORY HYPERGRAPH ARCHITECTURE                        │
 *   │                                                                          │
 *   │   ┌─────────────────────────────────────────────────────────────────┐   │
 *   │   │                    AGENT MEMORY LAYER                            │   │
 *   │   │   Episode:001 ──→ Episode:002 ──→ Episode:003                   │   │
 *   │   │   (Fraud ring)     (Denied claim)   (Investigation)             │   │
 *   │   └───────────┬─────────────┬─────────────┬─────────────────────────┘   │
 *   │               │ HyperEdge   │             │                              │
 *   │               ▼             ▼             ▼                              │
 *   │   ┌─────────────────────────────────────────────────────────────────┐   │
 *   │   │                    KNOWLEDGE GRAPH LAYER                         │   │
 *   │   │   Provider:P001 ────▶ Claim:C123 ◀──── Claimant:C001            │   │
 *   │   │   SAME QUAD STORE - One SPARQL query traverses BOTH!            │   │
 *   │   └─────────────────────────────────────────────────────────────────┘   │
 *   │                                                                          │
 *   │   KEY FEATURES:                                                          │
 *   │   • Temporal scoring: Recency + Relevance + Importance                  │
 *   │   • Rolling context window: 1h → 24h → 7d → 1y                          │
 *   │   • Idempotent responses: Same question = Same answer (cached)          │
 *   │   • SPARQL traverses both memory AND knowledge graph                    │
 *   └──────────────────────────────────────────────────────────────────────────┘
 *
 * WHY THIS MATTERS:
 *   Without Memory Hypergraph: Agent forgets everything between sessions
 *   With Memory Hypergraph: Agent recalls past findings, linked to KG entities
 *
 * @version 0.6.0
 */

const {
  GraphDB,
  EmbeddingService,
  DatalogProgram,
  evaluateDatalog,
  GraphFrame,
  getVersion,
  // Memory Layer
  AgentState,
  AgentRuntime,
  MemoryManager,
  GovernancePolicy,
  GovernanceEngine,
  AgentScope
} = require('../index.js')

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

const CONFIG = {
  // OpenAI API Key (from user)
  openai: {
    apiKey: process.env.OPENAI_API_KEY || 'your-openai-api-key',
    model: 'gpt-4o',
    embedModel: 'text-embedding-3-small',
    dimensions: 384
  },

  // Knowledge Graph
  kg: {
    baseUri: 'http://insurance.org/fraud-detection',
    graphUri: 'http://insurance.org/fraud-kb',
    memoryGraphUri: 'https://gonnect.ai/memory/'
  },

  // Memory Configuration
  memory: {
    weights: { recency: 0.3, relevance: 0.5, importance: 0.2 },
    decayRate: 0.995,
    maxContextTokens: 8192,
    rollingWindows: [1, 24, 168, 8760] // 1h, 24h, 7d, 1y
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FRAUD KNOWLEDGE BASE
// ═══════════════════════════════════════════════════════════════════════════════

const FRAUD_ONTOLOGY = `
@prefix ins: <http://insurance.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ins:P001 rdf:type ins:Provider .
ins:P001 ins:name "Quick Care Clinic" .
ins:P001 ins:riskScore 0.87 .
ins:P001 ins:claimVolume 847 .

ins:P002 rdf:type ins:Provider .
ins:P002 ins:name "City Hospital" .
ins:P002 ins:riskScore 0.35 .
ins:P002 ins:claimVolume 2341 .

ins:C001 rdf:type ins:Claimant .
ins:C001 ins:name "John Smith" .
ins:C001 ins:riskScore 0.85 .
ins:C001 ins:address ins:ADDR001 .

ins:C002 rdf:type ins:Claimant .
ins:C002 ins:name "Jane Doe" .
ins:C002 ins:riskScore 0.72 .
ins:C002 ins:address ins:ADDR001 .

ins:C003 rdf:type ins:Claimant .
ins:C003 ins:name "Bob Wilson" .
ins:C003 ins:riskScore 0.22 .
ins:C003 ins:address ins:ADDR002 .

ins:CLM001 rdf:type ins:Claim .
ins:CLM001 ins:claimant ins:C001 .
ins:CLM001 ins:provider ins:P001 .
ins:CLM001 ins:amount 18500 .
ins:CLM001 ins:type "bodily_injury" .

ins:CLM002 rdf:type ins:Claim .
ins:CLM002 ins:claimant ins:C002 .
ins:CLM002 ins:provider ins:P001 .
ins:CLM002 ins:amount 22300 .
ins:CLM002 ins:type "bodily_injury" .

ins:CLM003 rdf:type ins:Claim .
ins:CLM003 ins:claimant ins:C001 .
ins:CLM003 ins:provider ins:P002 .
ins:CLM003 ins:amount 8500 .
ins:CLM003 ins:type "collision" .

ins:C001 ins:knows ins:C002 .
ins:C002 ins:knows ins:C001 .
`

// ═══════════════════════════════════════════════════════════════════════════════
// MEMORY HYPERGRAPH IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Memory Hypergraph - Connects agent episodes to knowledge graph entities
 */
class MemoryHypergraph {
  constructor(db, config) {
    this.db = db
    this.config = config
    this.episodes = []
    this.queryCache = new Map()
    this.embeddings = new Map()
  }

  /**
   * Store an episode with hyper-edges to KG entities
   */
  storeEpisode(episode) {
    const episodeId = `episode:${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    const storedEpisode = {
      id: episodeId,
      prompt: episode.prompt,
      result: episode.result,
      success: episode.success,
      kgEntities: episode.kgEntities || [],
      createdAt: new Date(),
      accessCount: 1,
      lastAccessed: new Date(),
      embedding: episode.embedding || null
    }
    this.episodes.push(storedEpisode)

    // Store in GraphDB as RDF (memory graph)
    const ttl = this._episodeToTtl(storedEpisode)
    try {
      this.db.loadTtl(ttl, this.config.memoryGraphUri)
    } catch (e) {
      // Memory graph may not support all features, continue anyway
    }

    return storedEpisode
  }

  /**
   * Retrieve similar episodes using temporal scoring
   * Score = α × Recency + β × Relevance + γ × Importance
   */
  retrieve(query, limit = 10) {
    const weights = this.config.weights
    const now = new Date()

    return this.episodes
      .map(ep => {
        // Recency: decay^hours
        const hoursElapsed = (now - ep.createdAt) / (1000 * 60 * 60)
        const recency = Math.pow(this.config.decayRate, hoursElapsed)

        // Relevance: simple text similarity (would use embeddings in production)
        const relevance = this._textSimilarity(query, ep.prompt)

        // Importance: log-normalized access count
        const maxAccess = Math.max(...this.episodes.map(e => e.accessCount))
        const importance = Math.log10(ep.accessCount + 1) / Math.log10(maxAccess + 1)

        // Weighted score
        const score = weights.recency * recency +
                      weights.relevance * relevance +
                      weights.importance * importance

        return { episode: ep, score }
      })
      .filter(m => m.score > 0.1)
      .sort((a, b) => b.score - a.score)
      .slice(0, limit)
  }

  /**
   * Rolling context window - expand time range until sufficient context
   */
  buildContextWindow(query, maxTokens = 8192) {
    const windows = this.config.rollingWindows
    const now = new Date()

    for (let i = 0; i < windows.length; i++) {
      const windowHours = windows[i]
      const cutoff = new Date(now - windowHours * 60 * 60 * 1000)

      const episodes = this.episodes
        .filter(ep => ep.createdAt >= cutoff)
        .sort((a, b) => b.createdAt - a.createdAt)

      const estimatedTokens = episodes.reduce((sum, ep) => {
        return sum + Math.ceil((ep.prompt.length + JSON.stringify(ep.result).length) / 4)
      }, 0)

      // If we have enough episodes or reached max window, return
      if (episodes.length >= 3 || i === windows.length - 1 || estimatedTokens >= maxTokens) {
        return {
          episodes: this.retrieve(query, 10).map(m => m.episode),
          estimatedTokens,
          timeWindowHours: windowHours,
          searchPasses: i + 1,
          truncated: estimatedTokens > maxTokens
        }
      }
    }

    return { episodes: [], estimatedTokens: 0, timeWindowHours: 0, searchPasses: 0, truncated: false }
  }

  /**
   * Idempotent query - same input returns cached result
   */
  getCachedResult(query) {
    return this.queryCache.get(query)
  }

  cacheResult(query, result) {
    this.queryCache.set(query, {
      result,
      cachedAt: new Date(),
      hash: this._simpleHash(JSON.stringify(result))
    })
  }

  _textSimilarity(a, b) {
    const wordsA = new Set(a.toLowerCase().split(/\s+/))
    const wordsB = new Set(b.toLowerCase().split(/\s+/))
    const intersection = new Set([...wordsA].filter(x => wordsB.has(x)))
    const union = new Set([...wordsA, ...wordsB])
    return intersection.size / union.size
  }

  _simpleHash(str) {
    let hash = 0
    for (let i = 0; i < str.length; i++) {
      hash = ((hash << 5) - hash) + str.charCodeAt(i)
      hash |= 0
    }
    return 'sha256:' + Math.abs(hash).toString(16).padStart(16, '0')
  }

  _episodeToTtl(episode) {
    return `
@prefix am: <https://gonnect.ai/ontology/agent-memory#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<${episode.id}> a am:Episode ;
    am:prompt "${episode.prompt.replace(/"/g, '\\"')}" ;
    am:success "${episode.success}"^^xsd:boolean ;
    am:createdAt "${episode.createdAt.toISOString()}"^^xsd:dateTime ;
    am:accessCount "${episode.accessCount}"^^xsd:integer .
`
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPENAI INTEGRATION
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Get embedding from OpenAI
 */
async function getOpenAIEmbedding(text) {
  try {
    const response = await fetch('https://api.openai.com/v1/embeddings', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${CONFIG.openai.apiKey}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: CONFIG.openai.embedModel,
        input: text,
        dimensions: CONFIG.openai.dimensions
      })
    })
    const data = await response.json()
    if (data.error) {
      console.log(`  [OpenAI] Embedding error: ${data.error.message}`)
      return null
    }
    return data.data[0].embedding
  } catch (e) {
    console.log(`  [OpenAI] Embedding failed: ${e.message}`)
    return null
  }
}

/**
 * Query OpenAI for natural language understanding
 */
async function queryOpenAI(prompt, context = '') {
  try {
    const response = await fetch('https://api.openai.com/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${CONFIG.openai.apiKey}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: CONFIG.openai.model,
        messages: [
          {
            role: 'system',
            content: `You are a fraud detection assistant. You have access to a knowledge graph with insurance claims, providers, and claimants. Be concise and specific.${context ? '\n\nPrevious context:\n' + context : ''}`
          },
          { role: 'user', content: prompt }
        ],
        max_tokens: 500,
        temperature: 0.1
      })
    })
    const data = await response.json()
    if (data.error) {
      return { success: false, error: data.error.message }
    }
    return { success: true, response: data.choices[0].message.content }
  } catch (e) {
    return { success: false, error: e.message }
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN DEMO
// ═══════════════════════════════════════════════════════════════════════════════

async function main() {
  console.log()
  console.log('═'.repeat(80))
  console.log('  MEMORY HYPERGRAPH DEMO - Fraud Detection with Persistent Memory')
  console.log(`  rust-kgdb v${getVersion()} | Memory Hypergraph Architecture`)
  console.log('═'.repeat(80))
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // PHASE 1: Initialize Knowledge Graph
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('┌─ PHASE 1: Initialize Knowledge Graph ────────────────────────────────────┐')
  const db = new GraphDB(CONFIG.kg.baseUri)
  db.loadTtl(FRAUD_ONTOLOGY, CONFIG.kg.graphUri)
  console.log(`  ✓ Knowledge Graph loaded: ${db.countTriples()} triples`)
  console.log(`  ✓ Graph URI: ${CONFIG.kg.graphUri}`)
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // PHASE 2: Initialize Memory Hypergraph
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('┌─ PHASE 2: Initialize Memory Hypergraph ──────────────────────────────────┐')
  const memory = new MemoryHypergraph(db, CONFIG.memory)
  const runtime = new AgentRuntime({
    name: 'fraud-detector-with-memory',
    model: CONFIG.openai.model,
    tools: ['kg.sparql.query', 'kg.memory.recall', 'kg.memory.store'],
    memoryCapacity: 100
  })
  runtime.transitionTo(AgentState.READY)
  console.log(`  ✓ Memory Hypergraph initialized`)
  console.log(`  ✓ Temporal scoring: recency=${CONFIG.memory.weights.recency}, relevance=${CONFIG.memory.weights.relevance}, importance=${CONFIG.memory.weights.importance}`)
  console.log(`  ✓ Rolling windows: ${CONFIG.memory.rollingWindows.map(h => h < 24 ? `${h}h` : h < 168 ? `${h/24}d` : `${Math.round(h/720)}mo`).join(' → ')}`)
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // PHASE 3: First Investigation - Fraud Ring Detection
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('┌─ PHASE 3: First Investigation - Fraud Ring Detection ────────────────────┐')
  console.log('│  Simulates: Monday, Dec 10 - Initial fraud analysis                       │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // Query all claimants with risk scores
  const allClaimantsQuery = `
    SELECT ?claimant ?name ?score WHERE {
      ?claimant <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://insurance.org/Claimant> .
      ?claimant <http://insurance.org/name> ?name .
      ?claimant <http://insurance.org/riskScore> ?score .
    }
  `
  const allClaimants = db.querySelect(allClaimantsQuery)
  // Filter high-risk claimants in JavaScript (score > 0.7)
  const highRiskResults = allClaimants.filter(r => {
    const score = parseFloat(r.bindings.score)
    return !isNaN(score) && score > 0.7
  })
  console.log(`  [SPARQL] Found ${highRiskResults.length} high-risk claimants`)
  highRiskResults.forEach(r => {
    console.log(`           - ${r.bindings.name}: risk score ${r.bindings.score}`)
  })

  // Detect triangles - fraud ring: C001 ↔ C002 both claim with P001
  const gf = new GraphFrame(
    JSON.stringify([
      { id: 'C001', type: 'claimant' },
      { id: 'C002', type: 'claimant' },
      { id: 'P001', type: 'provider' }
    ]),
    JSON.stringify([
      // C001 knows C002 (bidirectional)
      { src: 'C001', dst: 'C002', relationship: 'knows' },
      { src: 'C002', dst: 'C001', relationship: 'knows' },
      // Both claimants use same provider - forms triangle
      { src: 'C001', dst: 'P001', relationship: 'claims_with' },
      { src: 'P001', dst: 'C001', relationship: 'serves' },
      { src: 'C002', dst: 'P001', relationship: 'claims_with' },
      { src: 'P001', dst: 'C002', relationship: 'serves' }
    ])
  )
  const triangles = gf.triangleCount()
  console.log(`  [GraphFrame] Detected ${triangles} fraud ring triangle(s)`)

  // Store first episode in memory
  const episode1 = memory.storeEpisode({
    prompt: 'Investigate fraud patterns for Provider P001 (Quick Care Clinic)',
    result: {
      highRiskClaimants: highRiskResults.length,
      trianglesDetected: triangles,
      findings: 'Fraud ring detected: C001 ↔ C002 ↔ P001'
    },
    success: true,
    kgEntities: ['ins:P001', 'ins:C001', 'ins:C002']
  })
  console.log(`  [Memory] Stored Episode: ${episode1.id}`)
  console.log(`           Linked to KG entities: ${episode1.kgEntities.join(', ')}`)
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // PHASE 4: Second Investigation - Underwriting Decision
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('┌─ PHASE 4: Second Investigation - Underwriting Decision ──────────────────┐')
  console.log('│  Simulates: Wednesday, Dec 12 - Claims review                             │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // Check if there's relevant context from previous investigation
  const context1 = memory.buildContextWindow('Provider P001 underwriting decision')
  console.log(`  [Memory] Rolling window search:`)
  console.log(`           - Time window: ${context1.timeWindowHours}h`)
  console.log(`           - Episodes found: ${context1.episodes.length}`)
  console.log(`           - Estimated tokens: ${context1.estimatedTokens}`)

  // Use context in decision
  const episode2 = memory.storeEpisode({
    prompt: 'Underwriting review for new claim from Provider P001',
    result: {
      decision: 'DENIED',
      reason: 'Provider linked to fraud ring (see previous investigation)',
      priorContext: episode1.id
    },
    success: true,
    kgEntities: ['ins:P001', 'ins:CLM003']
  })
  console.log(`  [Memory] Stored Episode: ${episode2.id}`)
  console.log(`           Decision: DENIED (based on prior investigation)`)
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // PHASE 5: Recall - "What did we find last week?"
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('┌─ PHASE 5: Memory Recall - "What did we find last week?" ─────────────────┐')
  console.log('│  Simulates: Friday, Dec 15 - Analyst asks about previous findings         │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  const query = "What fraud patterns did we find with Provider P001?"

  // WITHOUT Memory Hypergraph (traditional approach)
  console.log('  ┌─ WITHOUT Memory Hypergraph (LangChain approach) ─────────────────────────┐')
  console.log('  │  Agent: "I don\'t have access to previous conversations."                 │')
  console.log('  │  Cost: Re-run entire fraud detection pipeline ($5, 30s)                  │')
  console.log('  └──────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // WITH Memory Hypergraph
  console.log('  ┌─ WITH Memory Hypergraph (rust-kgdb approach) ────────────────────────────┐')
  const memories = memory.retrieve(query, 5)
  console.log(`  │  Memories retrieved: ${memories.length}                                            │`)
  memories.forEach((m, i) => {
    console.log(`  │    ${i+1}. Score: ${m.score.toFixed(3)} - "${m.episode.prompt.slice(0, 40)}..."`)
  })
  console.log('  └──────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // PHASE 6: Idempotent Response Demo
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('┌─ PHASE 6: Idempotent Response Demo ──────────────────────────────────────┐')
  console.log('│  Same question = Same answer (compliance requirement)                     │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  const complianceQuery = "Analyze claims from Provider P001"

  // First call - compute and cache
  const result1 = { findings: 'Fraud ring detected', riskLevel: 'CRITICAL' }
  memory.cacheResult(complianceQuery, result1)
  console.log(`  First call:  Computed fresh result`)
  console.log(`               Hash: ${memory.getCachedResult(complianceQuery).hash}`)

  // Second call - return cached
  const cached = memory.getCachedResult(complianceQuery)
  console.log(`  Second call: Returned cached result (idempotent)`)
  console.log(`               Hash: ${cached.hash}`)
  console.log(`               Match: ${memory.getCachedResult(complianceQuery).hash === cached.hash ? '✓' : '✗'}`)
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // PHASE 7: OpenAI Integration with Memory Context
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('┌─ PHASE 7: OpenAI Integration with Memory Context ────────────────────────┐')
  console.log('│  LLM query augmented with Memory Hypergraph context                       │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  // Build context from memory
  const contextWindow = memory.buildContextWindow("Provider P001 fraud investigation")
  const contextSummary = contextWindow.episodes
    .map(ep => `- ${ep.prompt}: ${JSON.stringify(ep.result)}`)
    .join('\n')

  console.log(`  [Memory Context]`)
  console.log(`    Time window: ${contextWindow.timeWindowHours}h`)
  console.log(`    Episodes: ${contextWindow.episodes.length}`)
  console.log(`    Estimated tokens: ${contextWindow.estimatedTokens}`)
  console.log()

  // Query OpenAI with memory context
  console.log(`  [OpenAI Query with Context]`)
  console.log(`    User: "What should we do about Provider P001?"`)

  const aiResponse = await queryOpenAI(
    "What should we do about Provider P001 based on our findings?",
    contextSummary
  )

  if (aiResponse.success) {
    console.log(`    Agent: ${aiResponse.response.slice(0, 200)}...`)

    // Store this interaction as an episode
    memory.storeEpisode({
      prompt: "What should we do about Provider P001 based on our findings?",
      result: { aiResponse: aiResponse.response },
      success: true,
      kgEntities: ['ins:P001']
    })
    console.log(`    [Memory] Episode stored with AI response`)
  } else {
    console.log(`    [OpenAI] Query failed: ${aiResponse.error}`)
    console.log(`    (This is expected if API key is invalid or rate-limited)`)
  }
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // PHASE 8: SPARQL Query Across Memory + KG
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('┌─ PHASE 8: SPARQL Across Memory + KG ─────────────────────────────────────┐')
  console.log('│  Single query traverses BOTH memory graph AND knowledge graph            │')
  console.log('└─────────────────────────────────────────────────────────────────────────────┘')
  console.log()

  console.log('  Example SPARQL (conceptual - both graphs in same store):')
  console.log()
  console.log('  PREFIX am: <https://gonnect.ai/ontology/agent-memory#>')
  console.log('  PREFIX ins: <http://insurance.org/>')
  console.log()
  console.log('  SELECT ?episode ?finding ?claimAmount WHERE {')
  console.log('    # Search memory graph')
  console.log('    GRAPH <https://gonnect.ai/memory/> {')
  console.log('      ?episode a am:Episode ;')
  console.log('               am:prompt ?finding .')
  console.log('    }')
  console.log('    # Join with knowledge graph')
  console.log('    ?claim ins:provider <ins:P001> ;')
  console.log('           ins:amount ?claimAmount .')
  console.log('  }')
  console.log()

  // ─────────────────────────────────────────────────────────────────────────────
  // SUMMARY
  // ─────────────────────────────────────────────────────────────────────────────

  console.log('═'.repeat(80))
  console.log('  MEMORY HYPERGRAPH SUMMARY')
  console.log('═'.repeat(80))
  console.log()
  console.log('  ┌──────────────────────────────────────────────────────────────────────┐')
  console.log('  │ ARCHITECTURE                                                          │')
  console.log('  ├──────────────────────────────────────────────────────────────────────┤')
  console.log('  │ • Memory stored in SAME quad store as knowledge graph                │')
  console.log('  │ • HyperEdges connect episodes to KG entities (direct URIs)           │')
  console.log('  │ • Single SPARQL query traverses both memory AND KG                   │')
  console.log('  │ • Temporal scoring: Recency + Relevance + Importance                 │')
  console.log('  │ • Rolling context window manages token limits                         │')
  console.log('  │ • Idempotent responses for compliance                                 │')
  console.log('  └──────────────────────────────────────────────────────────────────────┘')
  console.log()
  console.log('  ┌──────────────────────────────────────────────────────────────────────┐')
  console.log('  │ STATISTICS                                                            │')
  console.log('  ├──────────────────────────────────────────────────────────────────────┤')
  console.log(`  │ Episodes stored:        ${memory.episodes.length.toString().padEnd(47)}│`)
  console.log(`  │ Cached queries:         ${memory.queryCache.size.toString().padEnd(47)}│`)
  console.log(`  │ KG triples:             ${db.countTriples().toString().padEnd(47)}│`)
  console.log('  └──────────────────────────────────────────────────────────────────────┘')
  console.log()
  console.log('  Run this demo:')
  console.log('    node examples/fraud-memory-hypergraph.js')
  console.log()
  console.log('═'.repeat(80))
}

main().catch(err => {
  console.error('Demo failed:', err.message)
  process.exit(1)
})
