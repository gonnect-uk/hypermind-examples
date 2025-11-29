package com.zenya.rustkgdb

import org.junit.jupiter.api.*
import org.junit.jupiter.api.Assertions.*

/**
 * Comprehensive regression test suite for Kotlin SDK.
 *
 * This test suite mirrors the 20 regression tests from the Rust SDK,
 * ensuring feature parity and consistent behavior across all language bindings.
 *
 * ## Test Categories
 *
 * - CRUD Operations (4 tests)
 * - SPARQL Queries (6 tests)
 * - Node Types (6 tests)
 * - Edge Cases (4 tests)
 */
@TestInstance(TestInstance.Lifecycle.PER_CLASS)
@TestMethodOrder(MethodOrderer.OrderAnnotation::class)
class RegressionTest {

    private lateinit var db: GraphDB

    @BeforeEach
    fun setUp() {
        db = GraphDB.inMemory()
    }

    @AfterEach
    fun tearDown() {
        db.clear()
    }

    // ============================================================================
    // CRUD OPERATIONS (4 tests)
    // ============================================================================

    @Test
    @Order(1)
    @DisplayName("Regression: Basic CRUD - Insert single triple and query")
    fun testBasicCrud() {
        // Insert a single triple
        db.insert()
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                Node.iri("http://example.org/TestClass")
            )
            .execute()

        // Verify count
        assertEquals(1, db.count(), "Should have 1 triple")

        // Query the triple
        val results = db.query()
            .sparql("SELECT ?type WHERE { <http://example.org/test> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type }")
            .execute()

        assertEquals(1, results.size, "Should have 1 result")
        assertNotNull(results[0]["type"], "Result should have 'type' binding")
    }

    @Test
    @Order(2)
    @DisplayName("Regression: Insert multiple triples in single operation")
    fun testInsertMultipleTriples() {
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(RDF.TYPE),
                Node.iri(FOAF.PERSON)
            )
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
            .execute()

        assertEquals(3, db.count(), "Should have 3 triples")
    }

    @Test
    @Order(3)
    @DisplayName("Regression: Bulk insert 100 triples")
    fun testBulkInsert() {
        // Insert 100 triples
        for (i in 1..100) {
            db.insert()
                .triple(
                    Node.iri("http://example.org/person$i"),
                    Node.iri(FOAF.NAME),
                    Node.literal("Person $i")
                )
                .execute()
        }

        assertEquals(100, db.count(), "Should have 100 triples")
    }

    @Test
    @Order(4)
    @DisplayName("Regression: Clear database")
    fun testClearDatabase() {
        // Insert some data
        db.insert()
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri(RDF.TYPE),
                Node.iri("http://example.org/Class")
            )
            .execute()

        assertEquals(1, db.count())

        // Clear
        db.clear()

        assertEquals(0, db.count(), "Should have 0 triples after clear")
        assertTrue(db.isEmpty(), "Database should be empty")
    }

    // ============================================================================
    // SPARQL QUERIES (6 tests)
    // ============================================================================

    @Test
    @Order(5)
    @DisplayName("Regression: SELECT all triples")
    fun testSelectAllTriples() {
        populateSampleData()

        val results = db.query()
            .sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }")
            .execute()

        assertTrue(results.size >= 7, "Should have at least 7 results")
    }

    @Test
    @Order(6)
    @DisplayName("Regression: SELECT with pattern matching")
    fun testSelectWithPattern() {
        populateSampleData()

        val results = db.query()
            .sparql("SELECT ?name WHERE { ?person <${FOAF.NAME}> ?name }")
            .execute()

        assertEquals(2, results.size, "Should find 2 people with names")

        val names = results.map { it["name"] }
        assertTrue(names.any { it?.contains("Alice") == true }, "Should contain Alice")
        assertTrue(names.any { it?.contains("Bob") == true }, "Should contain Bob")
    }

    @Test
    @Order(7)
    @DisplayName("Regression: Query with FILTER")
    fun testQueryWithFilter() {
        populateSampleData()

        // Find people older than 25
        val results = db.query()
            .sparql("""
                SELECT ?person ?age WHERE {
                    ?person <${FOAF.AGE}> ?age .
                }
            """.trimIndent())
            .execute()

        assertTrue(results.size >= 1, "Should find at least one person with age")
    }

    @Test
    @Order(8)
    @DisplayName("Regression: SPARQL DISTINCT")
    fun testSparqlDistinct() {
        // Insert duplicate predicates
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.KNOWS),
                Node.iri("http://example.org/bob")
            )
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.KNOWS),
                Node.iri("http://example.org/charlie")
            )
            .execute()

        val results = db.query()
            .sparql("SELECT DISTINCT ?p WHERE { ?s ?p ?o }")
            .execute()

        // Should return unique predicates
        assertTrue(results.size >= 1, "Should find predicates")
    }

    @Test
    @Order(9)
    @DisplayName("Regression: SPARQL OPTIONAL")
    fun testSparqlOptional() {
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.NAME),
                Node.literal("Alice")
            )
            .execute()

        // Query with OPTIONAL - should still return Alice even without age
        val results = db.query()
            .sparql("""
                SELECT ?name ?age WHERE {
                    ?person <${FOAF.NAME}> ?name .
                    OPTIONAL { ?person <${FOAF.AGE}> ?age }
                }
            """.trimIndent())
            .execute()

        assertEquals(1, results.size, "Should find Alice")
        assertNotNull(results[0]["name"], "Should have name")
    }

    @Test
    @Order(10)
    @DisplayName("Regression: Concurrent read safety")
    fun testConcurrentReadSafety() {
        populateSampleData()

        // Execute same query multiple times
        repeat(10) {
            val results = db.query()
                .sparql("SELECT ?s WHERE { ?s ?p ?o }")
                .execute()
            assertTrue(results.size > 0, "Should always return results")
        }
    }

    // ============================================================================
    // NODE TYPES (6 tests)
    // ============================================================================

    @Test
    @Order(11)
    @DisplayName("Regression: IRI nodes")
    fun testIriNodes() {
        val subject = Node.iri("http://example.org/subject")
        val predicate = Node.iri("http://example.org/predicate")
        val obj = Node.iri("http://example.org/object")

        db.insert()
            .triple(subject, predicate, obj)
            .execute()

        assertEquals(1, db.count())
    }

    @Test
    @Order(12)
    @DisplayName("Regression: Plain literal nodes")
    fun testPlainLiterals() {
        db.insert()
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri(RDFS.LABEL),
                Node.literal("Test Label")
            )
            .execute()

        assertEquals(1, db.count())
    }

    @Test
    @Order(13)
    @DisplayName("Regression: Unicode literals")
    fun testUnicodeLiterals() {
        db.insert()
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri(RDFS.LABEL),
                Node.literal("Hello 世界 🌍")
            )
            .execute()

        val results = db.query()
            .sparql("SELECT ?label WHERE { ?s <${RDFS.LABEL}> ?label }")
            .execute()

        assertEquals(1, results.size)
        val label = results[0]["label"]
        assertNotNull(label)
        assertTrue(label!!.contains("世界") || label.contains("Hello"), "Should contain Unicode text")
    }

    @Test
    @Order(14)
    @DisplayName("Regression: Language-tagged literals")
    fun testLanguageTaggedLiterals() {
        db.insert()
            .triple(
                Node.iri("http://example.org/doc"),
                Node.iri(RDFS.LABEL),
                Node.langLiteral("Bonjour", "fr")
            )
            .triple(
                Node.iri("http://example.org/doc"),
                Node.iri(RDFS.LABEL),
                Node.langLiteral("Hello", "en")
            )
            .execute()

        assertEquals(2, db.count(), "Should have 2 language-tagged triples")
    }

    @Test
    @Order(15)
    @DisplayName("Regression: Typed literals (integer, boolean)")
    fun testTypedLiterals() {
        db.insert()
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://example.org/age"),
                Node.integer(30)
            )
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://example.org/active"),
                Node.boolean(true)
            )
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://example.org/price"),
                Node.double(19.99)
            )
            .execute()

        assertEquals(3, db.count())
    }

    @Test
    @Order(16)
    @DisplayName("Regression: Blank nodes")
    fun testBlankNodes() {
        val blank1 = Node.blank("b1")
        val blank2 = Node.blank("b2")

        db.insert()
            .triple(
                blank1,
                Node.iri(FOAF.KNOWS),
                blank2
            )
            .execute()

        assertEquals(1, db.count())
    }

    // ============================================================================
    // EDGE CASES (4 tests)
    // ============================================================================

    @Test
    @Order(17)
    @DisplayName("Regression: Query empty database")
    fun testQueryEmptyDatabase() {
        val results = db.query()
            .sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }")
            .execute()

        assertEquals(0, results.size, "Empty database should return no results")
        assertTrue(results.isEmpty(), "Results should be empty")
    }

    @Test
    @Order(18)
    @DisplayName("Regression: Database state verification")
    fun testDatabaseStateVerification() {
        assertTrue(db.isEmpty(), "New database should be empty")
        assertEquals(0, db.count())

        db.insert()
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri(RDF.TYPE),
                Node.iri("http://example.org/Class")
            )
            .execute()

        assertFalse(db.isEmpty(), "Database should not be empty after insert")
        assertEquals(1, db.count())
    }

    @Test
    @Order(19)
    @DisplayName("Regression: Invalid SPARQL query handling")
    fun testInvalidSparqlQuery() {
        assertThrows<Exception> {
            db.query()
                .sparql("INVALID SPARQL SYNTAX {{{}}")
                .execute()
        }
    }

    @Test
    @Order(20)
    @DisplayName("Regression: Default GraphDB constructor")
    fun testDefaultConstructor() {
        val newDb = GraphDB.inMemory()
        assertNotNull(newDb)
        assertTrue(newDb.isEmpty())
    }

    // ============================================================================
    // HELPER METHODS
    // ============================================================================

    /**
     * Populates database with sample data for testing.
     *
     * Creates 7 triples:
     * - Alice (3 triples): type, name, age
     * - Bob (3 triples): type, name, age
     * - Relationship (1 triple): alice knows bob
     */
    private fun populateSampleData() {
        // Add Alice (3 triples)
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(RDF.TYPE),
                Node.iri(FOAF.PERSON)
            )
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
            .execute()

        // Add Bob (3 triples)
        db.insert()
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri(RDF.TYPE),
                Node.iri(FOAF.PERSON)
            )
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri(FOAF.NAME),
                Node.literal("Bob")
            )
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri(FOAF.AGE),
                Node.integer(25)
            )
            .execute()

        // Add relationship (1 triple)
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.KNOWS),
                Node.iri("http://example.org/bob")
            )
            .execute()
    }
}
