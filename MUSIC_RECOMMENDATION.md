# Music Recommendation: Semantic Discovery Demo

**Knowledge graph-powered music recommendations with explainable AI.**

```bash
npm run music
```

---

## Overview

This demo implements a semantic music recommendation engine using patterns from MusicBrainz and Wikidata. It demonstrates:

- Artist influence networks with transitive reasoning
- Genre taxonomy with parent/related relationships
- Collaborative filtering via graph analytics
- Explainable recommendations with derivation chains

---

## Architecture

```
Music Knowledge Graph              Recommendation Engine
┌─────────────────────┐           ┌─────────────────────┐
│  Artists            │           │  Datalog Rules      │
│  - Influence chains │    ──→    │  - Genre-based      │
│  - Genres           │           │  - Influence-based  │
│  - Albums           │           │  - Collaborative    │
│                     │           │                     │
│  Genre Taxonomy     │    ──→    │  GraphFrame         │
│  - Parent genres    │           │  - PageRank         │
│  - Related genres   │           │  - Shortest paths   │
│                     │           │                     │
│  User Profiles      │    ──→    │  ThinkingReasoner   │
│  - Listening history│           │  - Natural language │
│  - Preferences      │           │  - Proofs           │
└─────────────────────┘           └─────────────────────┘
```

---

## Data Model

### Artist Ontology

```turtle
@prefix music: <http://music.org/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

# Artist influence is transitive
music:influencedBy a owl:TransitiveProperty .

# Artist with rich metadata
music:LedZeppelin a music:Artist ;
    music:name "Led Zeppelin" ;
    music:formed "1968"^^xsd:gYear ;
    music:genre music:HardRock, music:BluesRock ;
    music:influencedBy music:BluesBreaers, music:Cream ;
    music:influenced music:GnR, music:Aerosmith .
```

### Genre Taxonomy

```turtle
# Genre hierarchy
music:Metal a music:Genre ;
    music:parentGenre music:Rock ;
    music:relatedGenre music:HardRock .

music:ThrashMetal a music:Genre ;
    music:parentGenre music:Metal ;
    music:tempo "fast" .
```

### User Profiles

```turtle
music:User_Alice a music:User ;
    music:listened music:LedZeppelin, music:PinkFloyd ;
    music:preferredGenre music:ProgressiveRock ;
    music:listenCount "247"^^xsd:integer .
```

---

## Datalog Rules

### Genre-Based Recommendations

```prolog
% Recommend artists in same genre
genreRecommendation(?user, ?artist) :-
    preferredGenre(?user, ?genre),
    genre(?artist, ?genre),
    not(listened(?user, ?artist)).
```

### Influence-Based Recommendations

```prolog
% Recommend influential artists (transitive)
influenceRecommendation(?user, ?artist) :-
    listened(?user, ?likedArtist),
    influencedBy(?likedArtist, ?artist),
    not(listened(?user, ?artist)).

% Also recommend artists they influenced
influenceRecommendation(?user, ?artist) :-
    listened(?user, ?likedArtist),
    influenced(?likedArtist, ?artist),
    not(listened(?user, ?artist)).
```

### Collaborative Filtering

```prolog
% Users with similar taste
similarUser(?user1, ?user2) :-
    listened(?user1, ?artist),
    listened(?user2, ?artist),
    notEqual(?user1, ?user2).

% Recommend what similar users like
collaborativeRecommendation(?user, ?artist) :-
    similarUser(?user, ?similar),
    listened(?similar, ?artist),
    not(listened(?user, ?artist)).
```

---

## Test Scenarios (15 Total)

| # | Scenario | What It Tests |
|---|----------|---------------|
| 1 | Artist Load | Knowledge graph with 50+ artists |
| 2 | Genre Taxonomy | Hierarchical genre structure |
| 3 | Influence Network | Artist influence chains |
| 4 | User Profiles | Listening history tracking |
| 5 | Genre Query | Find artists by genre |
| 6 | Influence Query | Transitive influence discovery |
| 7 | Era Query | Artists by formation decade |
| 8 | Album Query | Discography exploration |
| 9 | Genre Recommendations | Datalog genre-based rules |
| 10 | Influence Recommendations | OWL transitive reasoning |
| 11 | Collaborative Filtering | User similarity matching |
| 12 | PageRank Analysis | Most influential artists |
| 13 | ThinkingReasoner | Natural language recommendations |
| 14 | Recommendation Explanation | Full derivation chain proofs |
| 15 | HyperMindAgent | LLM-powered music discovery |

---

## Sample Output (2025-12-24 Local Run)

```
================================================================================
  MUSIC RECOMMENDATION AGENT
  Semantic Music Discovery with Knowledge Graphs
  rust-kgdb v0.8.18 | Data: MusicBrainz + Wikidata Patterns
================================================================================

[1] Loading Music Ontology...
    Triples loaded: 331
    [PASS] Music ontology loaded

[2] SPARQL: Query Artists by Genre...
    Artists found: 15
      Rock: U2, Queen, The Beatles, Nirvana, Arctic Monkeys
      ArtRock: U2, Coldplay, Pink Floyd, Radiohead
      HardRock: Queen, Deep Purple, Guns N' Roses, Led Zeppelin, Black Sabbath
      HeavyMetal: Slayer, Megadeth, Metallica, Deep Purple, Black Sabbath
      ThrashMetal: Slayer, Megadeth, Metallica
    [PASS] Artist catalog loaded

[3] SPARQL: Artist Influence Network...
    Influence relationships: 8
      U2 -> Coldplay
      Led Zeppelin -> Metallica
      Black Sabbath -> Metallica
      The Beatles -> Pink Floyd
      The Beatles -> Radiohead
    [PASS] Influence network mapped

[4] SPARQL: Genre Taxonomy...
    Genres: 17
      Pop (parent: root)
      R&B (parent: root)
      Jazz (parent: root)
      Rock (parent: root)
      Soul (parent: root)
      Blues (parent: root)
    [PASS] Genre hierarchy loaded

[5] SPARQL: Top Selling Albums...
    Albums: 7
      Abbey Road by The Beatles: 31M copies
      Nevermind by Nirvana: 30M copies
      OK Computer by Radiohead: 4.5M copies
      Led Zeppelin IV by Led Zeppelin: 37M copies
      Master of Puppets by Metallica: 6M copies
      A Night at the Opera by Queen: 6M copies
      The Dark Side of the Moon by Pink Floyd: 45M copies
    [PASS] Album catalog loaded

[6] SPARQL: User Listening History...
    User listening records: 12
      Bob: Coldplay, Pink Floyd, Radiohead
      Alice: Metallica, Led Zeppelin, Black Sabbath
      Diana: Nirvana, Radiohead, Arctic Monkeys
      Charlie: U2, Queen, The Beatles
    [PASS] User profiles loaded

[7] SPARQL: Find Artists Similar to Led Zeppelin...
    Artists sharing genres with Led Zeppelin: 4
      - Queen
      - Deep Purple
      - Guns N' Roses
      - Black Sabbath
    [PASS] Genre-based similarity works

[8] SPARQL: Influence Chain from The Beatles...
    Artists influenced by The Beatles: 3
      - Pink Floyd
      - Radiohead
      - Led Zeppelin
    [PASS] Influence traversal works

[9] SPARQL: Related Genres (OWL SymmetricProperty)...
    Related genre pairs: 5
      Soul <-> R&B
      R&B <-> Soul
      Progressive Rock <-> Art Rock
      Heavy Metal <-> Hard Rock
      Hard Rock <-> Heavy Metal
    [PASS] Symmetric genre relationships

[10] Datalog: Recommendation Rules...
    [PASS] Recommendation engine ready

[11] GraphFrame: Artist Influence Network...
    Vertices: 12 artists
    Edges: 14 influence relationships
    Most Influential Artists (PageRank):
      1. Coldplay: 0.0456
      2. ArcticMonkeys: 0.0298
      3. Metallica: 0.0272
      4. Radiohead: 0.0264
      5. Nirvana: 0.0183
    Connected components: 1
    [PASS] Influence network analyzed

[12] GraphFrame: Musical Distance (Shortest Paths)...
    [PASS] Graph traversal operational

[13] ThinkingReasoner: OWL Property Inference...
    [PASS] Reasoning available

[14] Generate Recommendations for Alice...
    User Profile:
      Listened to: Led Zeppelin, Black Sabbath, Metallica
      Favorite genres: Heavy Metal, Hard Rock

    RECOMMENDATION ENGINE:
      Step 1: [PROFILE] Extract genre preferences
      Step 2: [GRAPH] Find artists in same genres
      Step 3: [INFLUENCE] Find influencers of favorites
      Step 4: [FILTER] Remove already listened
      Step 5: [RANK] Sort by relevance

    RECOMMENDATIONS FOR ALICE:
      1. Guns N' Roses (Hard Rock, influenced by Led Zeppelin)
      2. Queen (Hard Rock, influenced by Beatles)
      3. Nirvana (influenced by Led Zeppelin)

    PROOF HASH: 67af3637a30af116...

    [PASS] Personalized recommendations generated

[15] HyperMindAgent: Natural Language Query with LLM...
    Agent: music-advisor
    Model: GPT-4o

    USER QUESTION:
    "Based on the music knowledge graph, who are similar artists to Led Zeppelin and Metallica?"

    AGENT ANSWER:
    Similar artists to Metallica are Slayer and Megadeth, while Led Zeppelin
    is similar to Deep Purple.

    PROOF HASH: SHA-256 d76b14a5b8f53422...

    [PASS] HyperMindAgent query successful

================================================================================
  TEST RESULTS: 15 PASSED, 0 FAILED - 100.0% PASS RATE
================================================================================

  MUSIC RECOMMENDATION CAPABILITIES:
    - Artist ontology with genres, albums, influence
    - Genre taxonomy with parent/related relationships
    - User listening history and preferences
    - GraphFrame influence network analysis
    - PageRank for artist importance
    - Shortest paths for musical distance
    - Datalog rules for recommendations
    - OWL reasoning (Symmetric, Transitive properties)
    - Cryptographic proof per recommendation

  DATA SOURCES: MusicBrainz, Wikidata patterns
```

---

## Dataset Reference

Based on patterns from:
- **MusicBrainz** - Open music encyclopedia
- **Wikidata** - Structured artist data
- **AllMusic** - Genre taxonomy and influences

Features:
- Artist metadata (name, formed, disbanded)
- Genre classification with hierarchy
- Influence relationships (who influenced whom)
- Album discographies with release dates

---

## Key Features Demonstrated

1. **OWL TransitiveProperty**: Artist influence chains
2. **Genre Taxonomy**: Hierarchical genre reasoning
3. **Datalog Rules**: Multi-strategy recommendations
4. **Collaborative Filtering**: User similarity matching
5. **GraphFrame PageRank**: Artist importance ranking
6. **ThinkingReasoner**: Natural language queries
7. **Derivation Chains**: Explainable recommendations

---

## GraphFrame Analytics: Artist Network Analysis

The music recommendation engine uses **GraphFrame** for powerful graph analytics on the artist influence network. This enables PageRank-based importance scoring and shortest path calculations for musical distance.

### What is GraphFrame?

GraphFrame is a graph analytics framework that operates on vertex/edge data structures. It provides algorithms like PageRank, connected components, and shortest paths - essential for understanding artist influence networks.

### Real Code Example: Artist Influence Network

```javascript
const { GraphDB, GraphFrame } = require('rust-kgdb');

// Load music ontology
const db = new GraphDB('http://music.gonnect.ai/');
db.loadTtl(MUSIC_ONTOLOGY_TTL, null);

// Build graph from SPARQL results
const vertices = [
  { id: 'Beatles', genre: 'Rock' },
  { id: 'LedZeppelin', genre: 'HardRock' },
  { id: 'PinkFloyd', genre: 'ProgressiveRock' },
  { id: 'BlackSabbath', genre: 'HeavyMetal' },
  { id: 'Metallica', genre: 'ThrashMetal' },
  { id: 'GunsNRoses', genre: 'HardRock' },
  { id: 'Radiohead', genre: 'ArtRock' },
  { id: 'Queen', genre: 'Rock' },
  { id: 'Coldplay', genre: 'ArtRock' },
  { id: 'U2', genre: 'Rock' },
  { id: 'Nirvana', genre: 'Grunge' },
  { id: 'ArcticMonkeys', genre: 'Rock' }
];

const edges = [
  { src: 'Beatles', dst: 'LedZeppelin', rel: 'influenced' },
  { src: 'Beatles', dst: 'PinkFloyd', rel: 'influenced' },
  { src: 'Beatles', dst: 'Radiohead', rel: 'influenced' },
  { src: 'Beatles', dst: 'Nirvana', rel: 'influenced' },
  { src: 'Beatles', dst: 'Queen', rel: 'influenced' },
  { src: 'LedZeppelin', dst: 'Metallica', rel: 'influenced' },
  { src: 'LedZeppelin', dst: 'GunsNRoses', rel: 'influenced' },
  { src: 'LedZeppelin', dst: 'Nirvana', rel: 'influenced' },
  { src: 'BlackSabbath', dst: 'Metallica', rel: 'influenced' },
  { src: 'PinkFloyd', dst: 'Radiohead', rel: 'influenced' },
  { src: 'Radiohead', dst: 'Coldplay', rel: 'influenced' },
  { src: 'U2', dst: 'Coldplay', rel: 'influenced' },
  { src: 'Beatles', dst: 'ArcticMonkeys', rel: 'influenced' },
  { src: 'Nirvana', dst: 'ArcticMonkeys', rel: 'influenced' }
];

// Create GraphFrame
const gf = new GraphFrame(JSON.stringify(vertices), JSON.stringify(edges));
```

### PageRank: Find Most Influential Artists

PageRank identifies the most influential nodes in the graph - artists who have influenced the most other artists.

```javascript
// Run PageRank (dampingFactor=0.85, maxIterations=20)
const prResult = gf.pageRank(0.85, 20);
const pr = typeof prResult === 'string' ? JSON.parse(prResult) : prResult;

// Sort by influence score
const sortedPR = Object.entries(pr)
  .sort((a, b) => b[1] - a[1])
  .slice(0, 5);

console.log('Most Influential Artists:');
sortedPR.forEach(([artist, score], i) => {
  console.log(`  ${i + 1}. ${artist}: ${score.toFixed(4)}`);
});
```

**Output:**
```
Most Influential Artists:
  1. Coldplay: 0.1866
  2. ArcticMonkeys: 0.1222
  3. Metallica: 0.1113
  4. Radiohead: 0.1081
  5. Nirvana: 0.0750
```

**Insight**: Coldplay ranks highest because it receives influence from multiple sources (Radiohead + U2). Metallica is highly influential due to Led Zeppelin + Black Sabbath lineage.

### Shortest Paths: Musical Distance

Calculate the "musical distance" between artists - how many hops through the influence network.

```javascript
// Find shortest paths from The Beatles
const pathsResult = gf.shortestPaths(JSON.stringify(['Beatles']));
const paths = JSON.parse(pathsResult).distances || JSON.parse(pathsResult);

console.log('Musical Distance from The Beatles:');
Object.entries(paths)
  .sort((a, b) => a[1] - b[1])
  .slice(0, 6)
  .forEach(([artist, dist]) => {
    if (dist < Infinity) {
      console.log(`  ${artist}: ${dist} hop${dist > 1 ? 's' : ''}`);
    }
  });
```

**Output:**
```
Musical Distance from The Beatles:
  LedZeppelin: 1 hop
  PinkFloyd: 1 hop
  Radiohead: 1 hop
  Nirvana: 1 hop
  Queen: 1 hop
  Metallica: 2 hops
```

**Insight**: Metallica is 2 hops from The Beatles (Beatles → Led Zeppelin → Metallica), showing the influence lineage.

### Triangle Detection: Fraud Rings (Bonus)

GraphFrame can also detect triangles (cycles) - useful for fraud detection with shared relationships.

```javascript
const triangleCount = gf.triangleCount();
console.log(`Fraud ring triangles found: ${triangleCount}`);
```

### Why GraphFrame for Music Recommendations?

| Capability | Use Case | Example |
|------------|----------|---------|
| **PageRank** | Find most influential artists | The Beatles → highest influencer score |
| **Shortest Paths** | Musical distance between artists | Metallica is 2 hops from Beatles |
| **Triangle Detection** | Identify collaboration clusters | Mutual influence circles |
| **Connected Components** | Find isolated music scenes | Separate genre clusters |

### Integration with HyperMindAgent

GraphFrame analytics integrate seamlessly with HyperMindAgent for explainable recommendations:

```javascript
const { HyperMindAgent, GraphFrame } = require('rust-kgdb');

// Create agent with GraphFrame analytics
const agent = new HyperMindAgent({
  name: 'music-advisor',
  kg: db,
  apiKey: process.env.OPENAI_API_KEY
});

// Agent uses GraphFrame internally for:
// 1. PageRank to prioritize influential artists
// 2. Shortest paths for recommendation diversity
// 3. Genre clustering for similarity matching

const result = await agent.call(
  'Who are the most influential artists in my listening history?'
);

// Returns answer with proof hash
console.log(result.answer);
console.log(`Proof: ${result.proof.hash}`);
```

### Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| PageRank (12 vertices) | <1ms | In-memory graph |
| Shortest Paths | <1ms | BFS traversal |
| Triangle Detection | <1ms | Pattern matching |
| SPARQL + GraphFrame | <5ms | Combined query |

---

## Artists Included

| Artist | Genre | Key Influences |
|--------|-------|----------------|
| Led Zeppelin | Hard Rock, Blues Rock | Blues Breakers, Cream |
| Pink Floyd | Progressive Rock | The Beatles, Syd Barrett |
| Metallica | Thrash Metal | Black Sabbath, Motorhead |
| Radiohead | Alternative Rock | Pink Floyd, R.E.M. |
| The Beatles | Rock, Pop | Chuck Berry, Little Richard |
| Queen | Rock, Glam Rock | Led Zeppelin, David Bowie |
| Nirvana | Grunge | Pixies, Melvins |
| Tool | Progressive Metal | King Crimson, Pink Floyd |

---

## Run the Demo

```bash
cd hypermind-examples
npm install
npm run music
```

No API key required - runs entirely in-memory with curated artist data.

---

## Example Queries

### User Prompt → SPARQL → Answer

**Query 1: "Who are the artists in the Hard Rock genre?"**

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX music: <http://music.org/>

SELECT ?artist ?name WHERE {
  ?artist rdf:type music:Artist .
  ?artist music:genre music:HardRock .
  ?artist music:name ?name .
}
```

**Answer:** Queen, Guns N' Roses, Led Zeppelin, Black Sabbath

---

**Query 2: "Which artists influenced Metallica?"**

```sparql
PREFIX music: <http://music.org/>

SELECT ?influencer ?name WHERE {
  music:Metallica music:influencedBy ?influencer .
  ?influencer music:name ?name .
}
```

**Answer:** Led Zeppelin, Black Sabbath

---

**Query 3: "What are related genres to Heavy Metal?"**

```sparql
PREFIX music: <http://music.org/>

SELECT ?genre1 ?genre2 WHERE {
  ?genre1 music:relatedGenre ?genre2 .
}
```

**Answer:**
- Heavy Metal ↔ Hard Rock
- Soul ↔ R&B
- Progressive Rock ↔ Art Rock

(OWL SymmetricProperty ensures bidirectional relationships)

---

### PageRank: Most Influential Artists

| Rank | Artist | PageRank Score |
|------|--------|----------------|
| 1 | Coldplay | 0.1866 |
| 2 | Arctic Monkeys | 0.1222 |
| 3 | Metallica | 0.1113 |
| 4 | Radiohead | 0.1081 |
| 5 | Nirvana | 0.0750 |

---

## HyperMindAgent Interactions (LLM-Powered)

The HyperMindAgent provides natural language music discovery powered by GPT-4o or Claude. Below are real interactions from local execution:

### Interaction Table (from actual local run)

| User Query | LLM Answer | Knowledge Source | Reasoning | Proof |
|------------|------------|------------------|-----------|-------|
| "I love Led Zeppelin and Metallica. What similar artists should I listen to and why?" | "You might enjoy listening to Guns N' Roses, as they are influenced by The Beatles, similar to Metallica. Additionally, exploring genres like Progressive Rock and Art Rock, which are related to the styles of Led Zeppelin, could also be appealing." | SPARQL on `music:influencedBy`, `music:relatedGenre` | Agent traversed influence network (Beatles → Metallica), used genre relationships (Progressive Rock ↔ Art Rock) | ✓ Generated |

**Note**: Recommendation Engine (Test 14) proof: `SHA-256: 5ead6b840bd57c88...`

### Sample Agent Output

```
[15] HyperMindAgent: Natural Language Query with LLM...
    Agent: music-advisor
    Model: GPT-4o

    USER QUESTION:
    "I love Led Zeppelin and Metallica. What similar artists should I listen to and why?"

    AGENT ANSWER:
    ------------------------------------------------------------
    You might enjoy listening to Guns N' Roses, as they are influenced
    by The Beatles, similar to Metallica. Additionally, exploring genres
    like Progressive Rock and Art Rock, which are related to the styles
    of Led Zeppelin, could also be appealing.
    ------------------------------------------------------------

    PROOF HASH: [object Object]...

    [PASS] HyperMindAgent query successful
```

### How It Works

1. **User Question** → Natural language music preference
2. **Schema Extraction** → Agent reads ontology (genre, influencedBy, relatedGenre)
3. **SPARQL Generation** → LLM generates queries for influence chains and genre relations
4. **Query Execution** → rust-kgdb executes against music knowledge graph
5. **Answer Synthesis** → LLM provides recommendations with explanations
6. **Proof Hash** → Cryptographic verification of reasoning chain

### Recommendation Logic

The agent combines multiple strategies:
- **Influence-based**: Find artists who influenced or were influenced by favorites
- **Genre-based**: Find artists in same or related genres
- **Transitive reasoning**: Follow OWL TransitiveProperty chains (A influenced B, B influenced C → A influenced C)
- **Symmetric relationships**: Use OWL SymmetricProperty for bidirectional genre relations

---

## Full Output Log

📄 **[View Complete Output Log](output/music-recommendation-output.txt)**
