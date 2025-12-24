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
=== Music Recommendation Agent Demo ===
Using MusicBrainz/Wikidata artist patterns

✓ Artist ontology loaded: 312 triples
✓ Genre taxonomy: 24 genres with hierarchy
✓ User profiles: 5 users with listening history

--- Recommendations for Alice ---

Genre-Based (Progressive Rock):
  1. Genesis - "Not listened, matches preferred genre"
  2. Yes - "Not listened, matches preferred genre"
  Proof: sha256:4a7b...

Influence-Based (from Pink Floyd):
  1. Radiohead - "Influenced by Pink Floyd"
  2. Muse - "Influenced by Pink Floyd (via Radiohead)"
  Proof: sha256:8c2d...

Collaborative (similar to Bob):
  1. Tool - "Bob also likes Pink Floyd and Tool"
  Proof: sha256:f3e1...

--- GraphFrame Insights ---

Most Influential Artists (PageRank):
  1. The Beatles (0.412)
  2. Led Zeppelin (0.356)
  3. Pink Floyd (0.298)
  4. Black Sabbath (0.267)
  5. Jimi Hendrix (0.234)

Influence Path: Beatles → Radiohead
  Beatles → Pink Floyd → Radiohead (2 hops)

=== 14/14 SCENARIOS PASSED ===
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
