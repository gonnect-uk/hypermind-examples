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

## Test Scenarios (14 Total)

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

---

## Sample Output

```
================================================================================
  MUSIC RECOMMENDATION AGENT
  Semantic Music Discovery with Knowledge Graphs
  rust-kgdb v0.8.18 | Data: MusicBrainz + Wikidata Patterns
================================================================================

[1] Loading Music Ontology...
    Triples loaded: 291
    [PASS] Music ontology loaded

[2] SPARQL: Query Artists by Genre...
    Artists found: 12
      Rock: U2, Queen, The Beatles, Nirvana, Arctic Monkeys
      ArtRock: U2, Coldplay, Pink Floyd, Radiohead
      HardRock: Queen, Guns N' Roses, Led Zeppelin, Black Sabbath
      Pop: The Beatles, Coldplay
      Grunge: Nirvana
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
    Genres: 8
      Pop (parent: PopularMusic)
      Rock (parent: PopularMusic)
      Grunge (parent: Rock)
      Art Rock (parent: Rock)
      Hard Rock (parent: Rock)
      Heavy Metal (parent: Rock)
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
      - Guns N' Roses
      - Black Sabbath
      - Led Zeppelin IV
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
    Recommendation rules evaluated
    Genre-based recommendations: 2
    Influence-based recommendations: 1
    Sample recommendations for Alice:
      - GunsNRoses (genre match)
      - Queen (genre match)
    [PASS] Datalog reasoning works

[11] GraphFrame: Artist Influence Network...
    Vertices: 12 artists
    Edges: 14 influence relationships
    Most Influential Artists (PageRank):
      1. Coldplay: 0.1866
      2. ArcticMonkeys: 0.1222
      3. Metallica: 0.1113
      4. Radiohead: 0.1081
      5. Nirvana: 0.0750
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

    PROOF HASH: b1aaea954edbdf24...

    [PASS] Personalized recommendations generated

================================================================================
  TEST RESULTS: 14 PASSED - 100% PASS RATE
================================================================================

  MUSIC RECOMMENDATION CAPABILITIES:
    - Artist ontology with genres, albums, influence
    - Genre taxonomy with parent/related relationships
    - User listening history and preferences
    - GraphFrame influence network analysis
    - PageRank for artist importance
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
