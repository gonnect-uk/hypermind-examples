package direct

import org.junit.jupiter.api.*
import uniffi.gonnect.*

/**
 * Direct UniFFI bindings test - No wrapper layer
 * Uses generated GraphDb class directly from UniFFI
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation::class)
class DirectBindingsTest {

    private lateinit var db: GraphDb

    @BeforeEach
    fun setup() {
        // Use default app graph URI
        db = GraphDb("http://example.org/test-app")
    }

    @AfterEach
    fun cleanup() {
        db.clear()
    }

    @Test
    @Order(1)
    @DisplayName("Basic triple insert and query")
    fun testBasicTripleInsertQuery() {
        // Load TTL data
        val ttl = """
            @prefix foaf: <http://xmlns.com/foaf/0.1/> .
            <http://example.org/alice> foaf:name "Alice" .
        """.trimIndent()

        db.loadTtl(ttl, null)

        // Query
        val results = db.querySelect("SELECT ?name WHERE { ?person <http://xmlns.com/foaf/0.1/name> ?name }")

        Assertions.assertEquals(1, results.size)
        Assertions.assertEquals("Alice", results[0].bindings["name"])
    }

    @Test
    @Order(2)
    @DisplayName("Count triples")
    fun testCountTriples() {
        val ttl = """
            <http://example.org/s1> <http://example.org/p> "o1" .
            <http://example.org/s2> <http://example.org/p> "o2" .
        """.trimIndent()

        db.loadTtl(ttl, null)

        val count = db.countTriples()
        Assertions.assertEquals(2uL, count)
    }

    @Test
    @Order(3)
    @DisplayName("Named graph operations")
    fun testNamedGraphs() {
        val ttl = """
            <http://example.org/data> <http://example.org/value> "42" .
        """.trimIndent()

        db.loadTtl(ttl, "http://example.org/graph1")

        val graphs = db.listGraphs()
        Assertions.assertTrue(graphs.contains("http://example.org/graph1"))
    }

    @Test
    @Order(4)
    @DisplayName("SPARQL CONSTRUCT query")
    fun testConstruct() {
        // Use full URIs to avoid any prefix issues
        val ttl = """
            <http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .
            <http://example.org/bob> <http://example.org/knows> <http://example.org/charlie> .
        """.trimIndent()

        db.loadTtl(ttl, null)

        // Verify data loaded
        val count = db.countTriples()
        println("Loaded $count triples")

        val results = db.query("""
            CONSTRUCT { ?a <http://example.org/friendOf> ?b }
            WHERE { ?a <http://example.org/knows> ?b }
        """.trimIndent())

        println("CONSTRUCT returned ${results.size} triples")
        Assertions.assertEquals(2, results.size)
    }

    @Test
    @Order(5)
    @DisplayName("Get version")
    fun testVersion() {
        val version = getVersion()
        Assertions.assertNotNull(version)
        Assertions.assertTrue(version.isNotEmpty())
    }
}
