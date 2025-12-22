/**
 * Euroleague Basketball Knowledge Graph + HyperMindAgent
 *
 * Based on: https://medium.com/@skontopo2009/representing-euroleague-play-by-play-data-as-a-knowledge-graph-6397534cdd75
 *
 * Run: OPENAI_API_KEY=your-key node examples/euroleague-basketball-agent.js
 *      (Also works without API key using schema-based generation)
 */

const { GraphDB, HyperMindAgent } = require('rust-kgdb')

// N-Triples format (full URIs) for reliable parsing
const EUROLEAGUE_DATA = `
<http://euroleague.net/ontology#Game> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Class> .
<http://euroleague.net/ontology#Team> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Class> .
<http://euroleague.net/ontology#Player> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Class> .
<http://euroleague.net/ontology#Event> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Class> .
<http://euroleague.net/ontology#Shot> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://euroleague.net/ontology#Event> .
<http://euroleague.net/ontology#Assist> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://euroleague.net/ontology#Event> .
<http://euroleague.net/ontology#Rebound> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://euroleague.net/ontology#Event> .
<http://euroleague.net/ontology#Steal> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://euroleague.net/ontology#Event> .
<http://euroleague.net/ontology#Block> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://euroleague.net/ontology#Event> .
<http://euroleague.net/ontology#Foul> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://euroleague.net/ontology#Event> .
<http://euroleague.net/ontology#teammateOf> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#SymmetricProperty> .
<http://euroleague.net/ontology#assistedBy> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#TransitiveProperty> .
<http://euroleague.net/team/monaco> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Team> .
<http://euroleague.net/team/monaco> <http://www.w3.org/2000/01/rdf-schema#label> "AS Monaco" .
<http://euroleague.net/team/monaco> <http://euroleague.net/ontology#city> "Monaco" .
<http://euroleague.net/team/maccabi> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Team> .
<http://euroleague.net/team/maccabi> <http://www.w3.org/2000/01/rdf-schema#label> "Maccabi Tel Aviv" .
<http://euroleague.net/team/maccabi> <http://euroleague.net/ontology#city> "Tel Aviv" .
<http://euroleague.net/team/panathinaikos> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Team> .
<http://euroleague.net/team/panathinaikos> <http://www.w3.org/2000/01/rdf-schema#label> "Panathinaikos" .
<http://euroleague.net/team/panathinaikos> <http://euroleague.net/ontology#city> "Athens" .
<http://euroleague.net/player/james> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/james> <http://www.w3.org/2000/01/rdf-schema#label> "Mike James" .
<http://euroleague.net/player/james> <http://euroleague.net/ontology#position> "Point Guard" .
<http://euroleague.net/player/james> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/monaco> .
<http://euroleague.net/player/james> <http://euroleague.net/ontology#totalPoints> "456" .
<http://euroleague.net/player/okobo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/okobo> <http://www.w3.org/2000/01/rdf-schema#label> "Elie Okobo" .
<http://euroleague.net/player/okobo> <http://euroleague.net/ontology#position> "Shooting Guard" .
<http://euroleague.net/player/okobo> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/monaco> .
<http://euroleague.net/player/motiejunas> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/motiejunas> <http://www.w3.org/2000/01/rdf-schema#label> "Donatas Motiejunas" .
<http://euroleague.net/player/motiejunas> <http://euroleague.net/ontology#position> "Center" .
<http://euroleague.net/player/motiejunas> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/monaco> .
<http://euroleague.net/player/james> <http://euroleague.net/ontology#teammateOf> <http://euroleague.net/player/okobo> .
<http://euroleague.net/player/james> <http://euroleague.net/ontology#teammateOf> <http://euroleague.net/player/motiejunas> .
<http://euroleague.net/player/okobo> <http://euroleague.net/ontology#teammateOf> <http://euroleague.net/player/motiejunas> .
<http://euroleague.net/player/wilbekin> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/wilbekin> <http://www.w3.org/2000/01/rdf-schema#label> "Scottie Wilbekin" .
<http://euroleague.net/player/wilbekin> <http://euroleague.net/ontology#position> "Point Guard" .
<http://euroleague.net/player/wilbekin> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/maccabi> .
<http://euroleague.net/player/dibartolomeo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/dibartolomeo> <http://www.w3.org/2000/01/rdf-schema#label> "John DiBartolomeo" .
<http://euroleague.net/player/dibartolomeo> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/maccabi> .
<http://euroleague.net/player/zizic> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/zizic> <http://www.w3.org/2000/01/rdf-schema#label> "Ante Zizic" .
<http://euroleague.net/player/zizic> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/maccabi> .
<http://euroleague.net/player/wilbekin> <http://euroleague.net/ontology#teammateOf> <http://euroleague.net/player/dibartolomeo> .
<http://euroleague.net/player/wilbekin> <http://euroleague.net/ontology#teammateOf> <http://euroleague.net/player/zizic> .
<http://euroleague.net/player/sloukas> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/sloukas> <http://www.w3.org/2000/01/rdf-schema#label> "Kostas Sloukas" .
<http://euroleague.net/player/sloukas> <http://euroleague.net/ontology#position> "Point Guard" .
<http://euroleague.net/player/sloukas> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/panathinaikos> .
<http://euroleague.net/player/sloukas> <http://euroleague.net/ontology#totalPoints> "312" .
<http://euroleague.net/player/sloukas> <http://euroleague.net/ontology#totalAssists> "187" .
<http://euroleague.net/player/nunn> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/nunn> <http://www.w3.org/2000/01/rdf-schema#label> "Kendrick Nunn" .
<http://euroleague.net/player/nunn> <http://euroleague.net/ontology#position> "Shooting Guard" .
<http://euroleague.net/player/nunn> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/panathinaikos> .
<http://euroleague.net/player/nunn> <http://euroleague.net/ontology#totalPoints> "456" .
<http://euroleague.net/player/papagiannis> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
<http://euroleague.net/player/papagiannis> <http://www.w3.org/2000/01/rdf-schema#label> "Georgios Papagiannis" .
<http://euroleague.net/player/papagiannis> <http://euroleague.net/ontology#playsFor> <http://euroleague.net/team/panathinaikos> .
<http://euroleague.net/player/sloukas> <http://euroleague.net/ontology#teammateOf> <http://euroleague.net/player/nunn> .
<http://euroleague.net/player/sloukas> <http://euroleague.net/ontology#teammateOf> <http://euroleague.net/player/papagiannis> .
<http://euroleague.net/player/nunn> <http://euroleague.net/ontology#teammateOf> <http://euroleague.net/player/papagiannis> .
<http://euroleague.net/player/nunn> <http://euroleague.net/ontology#assistedBy> <http://euroleague.net/player/sloukas> .
<http://euroleague.net/game/monaco_maccabi_2024> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Game> .
<http://euroleague.net/game/monaco_maccabi_2024> <http://www.w3.org/2000/01/rdf-schema#label> "AS Monaco vs Maccabi Tel Aviv" .
<http://euroleague.net/game/monaco_maccabi_2024> <http://euroleague.net/ontology#homeTeam> <http://euroleague.net/team/monaco> .
<http://euroleague.net/game/monaco_maccabi_2024> <http://euroleague.net/ontology#awayTeam> <http://euroleague.net/team/maccabi> .
<http://euroleague.net/event/e001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Shot> .
<http://euroleague.net/event/e001> <http://www.w3.org/2000/01/rdf-schema#label> "James 3-pointer" .
<http://euroleague.net/event/e001> <http://euroleague.net/ontology#player> <http://euroleague.net/player/james> .
<http://euroleague.net/event/e001> <http://euroleague.net/ontology#team> <http://euroleague.net/team/monaco> .
<http://euroleague.net/event/e001> <http://euroleague.net/ontology#points> "3" .
<http://euroleague.net/event/e001> <http://euroleague.net/ontology#quarter> "2" .
<http://euroleague.net/event/e002> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Assist> .
<http://euroleague.net/event/e002> <http://www.w3.org/2000/01/rdf-schema#label> "Okobo to James" .
<http://euroleague.net/event/e002> <http://euroleague.net/ontology#player> <http://euroleague.net/player/okobo> .
<http://euroleague.net/event/e002> <http://euroleague.net/ontology#assistTo> <http://euroleague.net/player/james> .
<http://euroleague.net/event/e003> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Foul> .
<http://euroleague.net/event/e003> <http://www.w3.org/2000/01/rdf-schema#label> "Wilbekin personal foul" .
<http://euroleague.net/event/e003> <http://euroleague.net/ontology#player> <http://euroleague.net/player/wilbekin> .
<http://euroleague.net/event/e003> <http://euroleague.net/ontology#foulType> "personal" .
<http://euroleague.net/event/e004> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Shot> .
<http://euroleague.net/event/e004> <http://www.w3.org/2000/01/rdf-schema#label> "Wilbekin layup" .
<http://euroleague.net/event/e004> <http://euroleague.net/ontology#player> <http://euroleague.net/player/wilbekin> .
<http://euroleague.net/event/e004> <http://euroleague.net/ontology#points> "2" .
<http://euroleague.net/event/e005> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Rebound> .
<http://euroleague.net/event/e005> <http://www.w3.org/2000/01/rdf-schema#label> "Motiejunas defensive rebound" .
<http://euroleague.net/event/e005> <http://euroleague.net/ontology#player> <http://euroleague.net/player/motiejunas> .
<http://euroleague.net/event/e006> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Steal> .
<http://euroleague.net/event/e006> <http://www.w3.org/2000/01/rdf-schema#label> "DiBartolomeo steal" .
<http://euroleague.net/event/e006> <http://euroleague.net/ontology#player> <http://euroleague.net/player/dibartolomeo> .
<http://euroleague.net/event/e007> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Block> .
<http://euroleague.net/event/e007> <http://www.w3.org/2000/01/rdf-schema#label> "Zizic block" .
<http://euroleague.net/event/e007> <http://euroleague.net/ontology#player> <http://euroleague.net/player/zizic> .
<http://euroleague.net/event/e008> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Shot> .
<http://euroleague.net/event/e008> <http://www.w3.org/2000/01/rdf-schema#label> "James clutch three" .
<http://euroleague.net/event/e008> <http://euroleague.net/ontology#player> <http://euroleague.net/player/james> .
<http://euroleague.net/event/e008> <http://euroleague.net/ontology#points> "3" .
<http://euroleague.net/event/e008> <http://euroleague.net/ontology#isClutch> "true" .
`

async function main() {
  console.log('='.repeat(70))
  console.log('  EUROLEAGUE BASKETBALL KNOWLEDGE GRAPH')
  console.log('  HyperMindAgent with Deductive Reasoning')
  console.log('='.repeat(70))
  console.log()
  console.log('Source: https://medium.com/@skontopo2009/')
  console.log('        representing-euroleague-play-by-play-data-as-a-knowledge-graph')
  console.log()

  // 1. Load Knowledge Graph
  console.log('[1] Loading Play-by-Play Knowledge Graph...')
  const db = new GraphDB('http://euroleague.net/')
  db.loadTtl(EUROLEAGUE_DATA, null)
  const tripleCount = db.countTriples()
  console.log(`    Triples: ${tripleCount}`)
  console.log()

  // 2. Query Teams
  console.log('[2] Teams in the Graph:')
  const teamsQ = `SELECT ?team ?label WHERE {
    ?team <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Team> .
    ?team <http://www.w3.org/2000/01/rdf-schema#label> ?label .
  }`
  const teams = db.querySelect(teamsQ)
  for (const r of teams) {
    const label = r.bindings?.label || r.label
    console.log(`    - ${clean(label)}`)
  }
  console.log()

  // 3. Query Players
  console.log('[3] Players and Teams:')
  const playersQ = `SELECT ?name ?team WHERE {
    ?p <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Player> .
    ?p <http://www.w3.org/2000/01/rdf-schema#label> ?name .
    ?p <http://euroleague.net/ontology#playsFor> ?team .
  }`
  const players = db.querySelect(playersQ)
  for (const r of players) {
    const name = clean(r.bindings?.name || r.name)
    const team = extractLast(r.bindings?.team || r.team)
    console.log(`    - ${name} (${team})`)
  }
  console.log()

  // 4. Play-by-Play Events
  console.log('[4] Play-by-Play Events:')
  const eventsQ = `SELECT ?label ?player WHERE {
    ?e <http://www.w3.org/2000/01/rdf-schema#label> ?label .
    ?e <http://euroleague.net/ontology#player> ?player .
  }`
  const events = db.querySelect(eventsQ)
  for (const r of events) {
    const label = clean(r.bindings?.label || r.label)
    const player = extractLast(r.bindings?.player || r.player)
    console.log(`    - ${label} by ${player}`)
  }
  console.log()

  // 5. Teammate Relationships (SymmetricProperty)
  console.log('[5] Teammate Relationships (owl:SymmetricProperty):')
  const tmQ = `SELECT ?a ?b WHERE {
    ?a <http://euroleague.net/ontology#teammateOf> ?b .
  }`
  const teammates = db.querySelect(tmQ)
  for (const r of teammates) {
    const a = extractLast(r.bindings?.a || r.a)
    const b = extractLast(r.bindings?.b || r.b)
    console.log(`    ${a} <-> ${b}`)
  }
  console.log()
  console.log('    Deduction: If James teammateOf Okobo, then Okobo teammateOf James')
  console.log()

  // 6. Assist Chain (TransitiveProperty)
  console.log('[6] Assist Chain (owl:TransitiveProperty):')
  const assistQ = `SELECT ?player ?by WHERE {
    ?player <http://euroleague.net/ontology#assistedBy> ?by .
  }`
  const assists = db.querySelect(assistQ)
  for (const r of assists) {
    const player = extractLast(r.bindings?.player || r.player)
    const by = extractLast(r.bindings?.by || r.by)
    console.log(`    ${player} <- assisted by <- ${by}`)
  }
  console.log()
  console.log('    Deduction: If Nunn assistedBy Sloukas, and X assistedBy Nunn,')
  console.log('               then X assistedBy Sloukas (transitive chain)')
  console.log()

  // 7. HyperMindAgent with NLP
  console.log('[7] Creating HyperMindAgent...')
  const agent = new HyperMindAgent({
    name: 'euroleague-analyst',
    kg: db,
    apiKey: process.env.OPENAI_API_KEY,
    model: 'gpt-4o'
  })
  console.log(`    Agent: ${agent.name}`)
  console.log(`    LLM: ${process.env.OPENAI_API_KEY ? 'OpenAI' : 'None (schema-based)'}`)
  console.log()

  // 8. Natural Language Questions
  console.log('[8] Natural Language Questions:')
  console.log()

  const questions = [
    'Who are the teammates of Mike James?',
    'Which player scored the clutch shot?',
    'Find all events by Monaco players'
  ]

  for (const q of questions) {
    console.log('-'.repeat(60))
    console.log(`Q: "${q}"`)
    console.log('-'.repeat(60))

    try {
      const result = await agent.call(q)
      console.log()
      console.log('ANSWER:', result.answer || '(see below)')

      if (result.reasoningStats) {
        console.log()
        console.log('REASONING:')
        console.log(`  Observations: ${result.reasoningStats.events || 0}`)
        console.log(`  Facts: ${result.reasoningStats.facts || 0}`)
        console.log(`  Rules: ${result.reasoningStats.rules || 0}`)
      }

      if (result.thinkingGraph?.derivationChain?.length > 0) {
        console.log()
        console.log('DERIVATION CHAIN:')
        for (const s of result.thinkingGraph.derivationChain.slice(0, 3)) {
          console.log(`  Step ${s.step}: ${s.conclusion || s.fact}`)
        }
      }
    } catch (e) {
      console.log(`  Error: ${e.message}`)
    }
    console.log()
  }

  // 9. Summary
  console.log('='.repeat(70))
  console.log('  SUMMARY')
  console.log('='.repeat(70))
  console.log()
  console.log(`  Knowledge Graph: ${tripleCount} triples`)
  console.log(`  Teams: ${teams.length}`)
  console.log(`  Players: ${players.length}`)
  console.log(`  Events: ${events.length}`)
  console.log(`  Teammate links: ${teammates.length}`)
  console.log()
  console.log('  OWL Properties enable automatic reasoning:')
  console.log('  - SymmetricProperty: A rel B => B rel A')
  console.log('  - TransitiveProperty: A rel B, B rel C => A rel C')
  console.log()
  console.log('  For K8s deployment: gonnect.hypermind@gmail.com')
  console.log()
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
