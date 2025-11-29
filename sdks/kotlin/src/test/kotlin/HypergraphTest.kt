package com.zenya.rustkgdb

import org.junit.jupiter.api.*
import org.junit.jupiter.api.Assertions.*

/**
 * Hypergraph-specific tests for Kotlin SDK.
 *
 * Tests hypergraph operations that go beyond standard RDF triple operations:
 * - Binary edges (2 nodes)
 * - Ternary edges (3 nodes - standard RDF triples)
 * - Quaternary edges (4 nodes - RDF quads with named graphs)
 * - N-ary edges (arbitrary number of nodes)
 * - Hyperedge traversal and pattern matching
 * - Complex multi-edge patterns
 *
 * Matches Rust SDK hypergraph test coverage for cross-language consistency.
 */
@TestInstance(TestInstance.Lifecycle.PER_CLASS)
@TestMethodOrder(MethodOrderer.OrderAnnotation::class)
class HypergraphTest {

    private lateinit var db: GraphDB

    @BeforeEach
    fun setUp() {
        db = GraphDB.inMemory()
    }

    @AfterEach
    fun tearDown() {
        db.clear()
    }

    @Test
    @Order(1)
    @DisplayName("Hypergraph: Binary edge (2 nodes)")
    fun testBinaryEdge() {
        // Binary hyperedge: Two nodes connected by an edge label
        // In RDF: (subject) -[predicate]-> (object)
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://example.org/likes"), // Edge label
                Node.iri("http://example.org/pizza")
            )
            .execute()

        val results = db.query()
            .sparql("SELECT ?who ?what WHERE { ?who <http://example.org/likes> ?what }")
            .execute()

        assertEquals(1, results.size, "Should find 1 binary edge")
    }

    @Test
    @Order(2)
    @DisplayName("Hypergraph: Ternary edge (standard RDF triple)")
    fun testTernaryEdge() {
        // Ternary hyperedge: Standard RDF triple
        // Three nodes: (subject, predicate, object)
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),      // Node 1: Subject
                Node.iri(FOAF.NAME),                        // Node 2: Predicate
                Node.literal("Alice")                       // Node 3: Object
            )
            .execute()

        assertEquals(1, db.count(), "Should have 1 ternary edge (triple)")
    }

    @Test
    @Order(3)
    @DisplayName("Hypergraph: Quaternary edge (named graph)")
    fun testQuaternaryEdge() {
        // Quaternary hyperedge: RDF quad with named graph
        // Four nodes: (subject, predicate, object, graph)
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.NAME),
                Node.literal("Alice")
            )
            .graph(Node.iri("http://example.org/graph1")) // 4th node: Named graph
            .execute()

        assertEquals(1, db.count(), "Should have 1 quaternary edge (quad)")
    }

    @Test
    @Order(4)
    @DisplayName("Hypergraph: Multiple edges from same subject (star pattern)")
    fun testMultipleEdgesSameSubject() {
        // One node (subject) with multiple outgoing hyperedges
        // Star pattern: Alice -[name]-> "Alice"
        //               Alice -[age]-> 30
        //               Alice -[email]-> "alice@example.org"
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.NAME),
                Node.literal("Alice")
            )
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.AGE),
                Node.integer(30)
            )
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("${FOAF.NS}mbox"),
                Node.literal("alice@example.org")
            )
            .execute()

        // Query all properties of Alice
        val results = db.query()
            .sparql("""
                SELECT ?p ?o WHERE {
                    <http://example.org/alice> ?p ?o
                }
            """)
            .execute()

        assertEquals(3, results.size, "Should find 3 outgoing edges")
    }

    @Test
    @Order(5)
    @DisplayName("Hypergraph: Edge traversal (2-hop path)")
    fun testEdgeTraversal() {
        // Hyperedge traversal: Following connected edges
        // Path: Alice -[knows]-> Bob -[knows]-> Charlie
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.KNOWS),
                Node.iri("http://example.org/bob")
            )
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri(FOAF.KNOWS),
                Node.iri("http://example.org/charlie")
            )
            .execute()

        // Find friends of friends (2-hop traversal)
        val results = db.query()
            .sparql("""
                SELECT ?friend_of_friend WHERE {
                    <http://example.org/alice> <${FOAF.KNOWS}> ?friend .
                    ?friend <${FOAF.KNOWS}> ?friend_of_friend
                }
            """)
            .execute()

        assertEquals(1, results.size, "Should find Charlie through Bob")
    }

    @Test
    @Order(6)
    @DisplayName("Hypergraph: Bidirectional edges")
    fun testBidirectionalEdges() {
        // Bidirectional edges: Two edges in opposite directions
        // Alice -[knows]-> Bob
        // Bob -[knows]-> Alice
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.KNOWS),
                Node.iri("http://example.org/bob")
            )
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri(FOAF.KNOWS),
                Node.iri("http://example.org/alice")
            )
            .execute()

        // Find mutual friends
        val results = db.query()
            .sparql("""
                SELECT ?person1 ?person2 WHERE {
                    ?person1 <${FOAF.KNOWS}> ?person2 .
                    ?person2 <${FOAF.KNOWS}> ?person1
                }
            """)
            .execute()

        assertEquals(2, results.size, "Should find 2 mutual relationships")
    }

    @Test
    @Order(7)
    @DisplayName("Hypergraph: Complex pattern (social network)")
    fun testComplexPattern() {
        // Complex hypergraph pattern: Small social network
        db.insert()
            .triple(Node.iri("http://example.org/alice"), Node.iri(FOAF.NAME), Node.literal("Alice"))
            .triple(Node.iri("http://example.org/alice"), Node.iri(FOAF.AGE), Node.integer(30))
            .triple(Node.iri("http://example.org/bob"), Node.iri(FOAF.NAME), Node.literal("Bob"))
            .triple(Node.iri("http://example.org/bob"), Node.iri(FOAF.AGE), Node.integer(25))
            .triple(Node.iri("http://example.org/alice"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/bob"))
            .execute()

        // Complex pattern: Find people who know someone
        val results = db.query()
            .sparql("""
                SELECT ?person ?name ?knows WHERE {
                    ?person <${FOAF.NAME}> ?name .
                    ?person <${FOAF.KNOWS}> ?knows
                }
            """)
            .execute()

        assertEquals(1, results.size, "Alice knows Bob")
    }

    @Test
    @Order(8)
    @DisplayName("Hypergraph: Triangle pattern")
    fun testTrianglePattern() {
        // Triangle pattern: Three nodes with circular connections
        // Alice -> Bob -> Charlie -> Alice
        db.insert()
            .triple(Node.iri("http://example.org/alice"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/bob"))
            .triple(Node.iri("http://example.org/bob"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/charlie"))
            .triple(Node.iri("http://example.org/charlie"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/alice"))
            .execute()

        // Find triangles
        val results = db.query()
            .sparql("""
                SELECT ?a ?b ?c WHERE {
                    ?a <${FOAF.KNOWS}> ?b .
                    ?b <${FOAF.KNOWS}> ?c .
                    ?c <${FOAF.KNOWS}> ?a
                }
            """)
            .execute()

        assertEquals(1, results.size, "Should find triangle pattern")
    }

    @Test
    @Order(9)
    @DisplayName("Hypergraph: Star pattern (one-to-many)")
    fun testStarPattern() {
        // Star pattern: One central node with multiple connections
        // Alice knows: Bob, Charlie, Dave, Eve
        val people = listOf(
            "http://example.org/bob",
            "http://example.org/charlie",
            "http://example.org/dave",
            "http://example.org/eve"
        )

        for (person in people) {
            db.insert()
                .triple(
                    Node.iri("http://example.org/alice"),
                    Node.iri(FOAF.KNOWS),
                    Node.iri(person)
                )
                .execute()
        }

        // Find all connections from center
        val results = db.query()
            .sparql("""
                SELECT ?person WHERE {
                    <http://example.org/alice> <${FOAF.KNOWS}> ?person
                }
            """)
            .execute()

        assertEquals(4, results.size, "Alice should know 4 people")
    }

    @Test
    @Order(10)
    @DisplayName("Hypergraph: Multi-hop traversal (3 hops)")
    fun testMultiHopTraversal() {
        // Multi-hop traversal: 3-hop path
        // Alice -> Bob -> Charlie -> Dave
        db.insert()
            .triple(Node.iri("http://example.org/alice"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/bob"))
            .triple(Node.iri("http://example.org/bob"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/charlie"))
            .triple(Node.iri("http://example.org/charlie"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/dave"))
            .execute()

        // 3-hop query
        val results = db.query()
            .sparql("""
                SELECT ?hop3 WHERE {
                    <http://example.org/alice> <${FOAF.KNOWS}> ?hop1 .
                    ?hop1 <${FOAF.KNOWS}> ?hop2 .
                    ?hop2 <${FOAF.KNOWS}> ?hop3
                }
            """)
            .execute()

        assertEquals(1, results.size, "Should find Dave 3 hops away")
    }

    @Test
    @Order(11)
    @DisplayName("Hypergraph: Multiple edge types between same nodes")
    fun testMultipleEdgeTypes() {
        // Multiple edge types between same nodes
        // Alice -[knows]-> Bob
        // Alice -[worksWith]-> Bob
        // Alice -[livesNear]-> Bob
        db.insert()
            .triple(Node.iri("http://example.org/alice"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/bob"))
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://example.org/worksWith"), Node.iri("http://example.org/bob"))
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://example.org/livesNear"), Node.iri("http://example.org/bob"))
            .execute()

        // Find all relationship types
        val results = db.query()
            .sparql("""
                SELECT ?relationship WHERE {
                    <http://example.org/alice> ?relationship <http://example.org/bob>
                }
            """)
            .execute()

        assertEquals(3, results.size, "Should find 3 different edge types")
    }

    @Test
    @Order(12)
    @DisplayName("Hypergraph: Typed edges (Person to Organization)")
    fun testTypedEdges() {
        // Hyperedges with typed nodes (RDF types)
        // Person nodes connected to Organization nodes
        db.insert()
            .triple(Node.iri("http://example.org/alice"), Node.iri(RDF.TYPE), Node.iri(FOAF.PERSON))
            .triple(Node.iri("http://example.org/acme"), Node.iri(RDF.TYPE), Node.iri("${FOAF.NS}Organization"))
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://example.org/worksFor"), Node.iri("http://example.org/acme"))
            .execute()

        // Find people working for organizations
        val results = db.query()
            .sparql("""
                SELECT ?person ?org WHERE {
                    ?person a <${FOAF.PERSON}> .
                    ?org a <${FOAF.NS}Organization> .
                    ?person <http://example.org/worksFor> ?org
                }
            """)
            .execute()

        assertEquals(1, results.size, "Should find typed edge relationship")
    }

    @Test
    @Order(13)
    @DisplayName("Hypergraph: Property graph pattern")
    fun testPropertyGraphPattern() {
        // Property graph pattern: Nodes with properties and edges
        // Simulates labeled property graph using RDF
        db.insert()
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://example.org/label"), Node.literal("Person"))
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://example.org/firstName"), Node.literal("Alice"))
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://example.org/age"), Node.integer(30))
            .execute()

        // Edge with properties
        db.insert()
            .triple(Node.iri("http://example.org/alice"), Node.iri(FOAF.KNOWS), Node.iri("http://example.org/bob"))
            .execute()

        // Query property graph
        val results = db.query()
            .sparql("""
                SELECT ?firstName ?age WHERE {
                    ?person <http://example.org/firstName> ?firstName .
                    ?person <http://example.org/age> ?age
                }
            """)
            .execute()

        assertEquals(1, results.size, "Should find property graph node")
    }

    @Test
    @Order(14)
    @DisplayName("Hypergraph: Large N-ary simulation (meeting with multiple participants)")
    fun testLargeNArySimulation() {
        // Simulate N-ary relationship using multiple triples
        // Meeting: Alice, Bob, Charlie at location X at time T
        val meeting = Node.blank("meeting1")

        db.insert()
            .triple(meeting, Node.iri(RDF.TYPE), Node.iri("http://example.org/Meeting"))
            .triple(meeting, Node.iri("http://example.org/participant"), Node.iri("http://example.org/alice"))
            .triple(meeting, Node.iri("http://example.org/participant"), Node.iri("http://example.org/bob"))
            .triple(meeting, Node.iri("http://example.org/participant"), Node.iri("http://example.org/charlie"))
            .triple(meeting, Node.iri("http://example.org/location"), Node.literal("Conference Room A"))
            .triple(meeting, Node.iri("http://example.org/time"), Node.literal("2024-01-15T10:00:00"))
            .execute()

        // Query N-ary relationship
        val results = db.query()
            .sparql("""
                SELECT ?participant WHERE {
                    ?meeting a <http://example.org/Meeting> .
                    ?meeting <http://example.org/participant> ?participant
                }
            """)
            .execute()

        assertEquals(3, results.size, "Should find 3 meeting participants")
    }
}
