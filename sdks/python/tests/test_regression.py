"""Comprehensive Regression Test Suite for Python SDK

This test suite covers:
1. Basic RDF triple operations (CRUD)
2. SPARQL query functionality
3. Node types (IRI, literal, typed literals, language tags)
4. Hypergraph operations (binary, ternary, n-ary edges)
5. Error handling and edge cases
6. Performance with large datasets

Matches Rust SDK test coverage for cross-language consistency.
"""

import pytest
from rust_kgdb import GraphDB, Node


class TestBasicCRUD:
    """Test basic create, read, update, delete operations"""

    @pytest.fixture
    def db(self):
        """Create fresh database for each test"""
        return GraphDB.in_memory()

    def test_create_empty_database(self, db):
        """Test: Empty database initialization"""
        assert db.is_empty()
        assert db.count() == 0

    def test_insert_single_triple(self, db):
        """Test: Insert single RDF triple"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                Node.iri("http://xmlns.com/foaf/0.1/Person")
            ) \
            .execute()

        assert db.count() == 1
        assert not db.is_empty()

    def test_insert_multiple_triples(self, db):
        """Test: Insert multiple triples in single operation"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/name"),
                Node.literal("Alice")
            ) \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/age"),
                Node.integer(30)
            ) \
            .execute()

        assert db.count() == 2

    def test_query_all_triples(self, db):
        """Test: SELECT * query"""
        # Insert test data
        db.insert() \
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                Node.iri("http://example.org/TestClass")
            ) \
            .execute()

        # Query all triples
        results = db.query() \
            .sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }") \
            .execute()

        assert len(results) == 1
        assert not results.is_empty()

    def test_clear_database(self, db):
        """Test: Clear all triples"""
        # Insert data
        db.insert() \
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://example.org/p"),
                Node.literal("value")
            ) \
            .execute()

        assert db.count() == 1

        # Clear
        db.clear()

        assert db.is_empty()
        assert db.count() == 0


class TestNodeTypes:
    """Test all RDF node type constructors"""

    @pytest.fixture
    def db(self):
        return GraphDB.in_memory()

    def test_iri_node(self, db):
        """Test: IRI node creation"""
        node = Node.iri("http://example.org/resource")

        db.insert() \
            .triple(
                node,
                Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                Node.iri("http://example.org/Class")
            ) \
            .execute()

        assert db.count() == 1

    def test_plain_literal(self, db):
        """Test: Plain literal without type or language"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/doc"),
                Node.iri("http://example.org/title"),
                Node.literal("Hello World")
            ) \
            .execute()

        assert db.count() == 1

    def test_typed_literal_integer(self, db):
        """Test: Typed literal with XSD integer"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/person"),
                Node.iri("http://example.org/age"),
                Node.integer(42)
            ) \
            .execute()

        results = db.query() \
            .sparql("SELECT ?age WHERE { <http://example.org/person> <http://example.org/age> ?age }") \
            .execute()

        assert len(results) == 1

    def test_typed_literal_boolean(self, db):
        """Test: Boolean typed literal"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/setting"),
                Node.iri("http://example.org/enabled"),
                Node.boolean(True)
            ) \
            .execute()

        assert db.count() == 1

    def test_typed_literal_double(self, db):
        """Test: Double/float typed literal"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/measurement"),
                Node.iri("http://example.org/value"),
                Node.double(3.14159)
            ) \
            .execute()

        assert db.count() == 1

    def test_language_tagged_literal(self, db):
        """Test: Literal with language tag"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/doc"),
                Node.iri("http://example.org/title"),
                Node.lang_literal("Hello", "en")
            ) \
            .triple(
                Node.iri("http://example.org/doc"),
                Node.iri("http://example.org/title"),
                Node.lang_literal("Bonjour", "fr")
            ) \
            .execute()

        assert db.count() == 2

    def test_blank_node(self, db):
        """Test: Blank node (anonymous resource)"""
        db.insert() \
            .triple(
                Node.blank("b1"),
                Node.iri("http://xmlns.com/foaf/0.1/name"),
                Node.literal("Anonymous")
            ) \
            .execute()

        assert db.count() == 1

    def test_unicode_literals(self, db):
        """Test: Unicode characters in literals"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://example.org/label"),
                Node.literal("Hello 世界 🌍 مرحبا")
            ) \
            .execute()

        results = db.query() \
            .sparql("SELECT ?label WHERE { <http://example.org/test> <http://example.org/label> ?label }") \
            .execute()

        assert len(results) == 1


class TestSPARQLQueries:
    """Test SPARQL 1.1 query functionality"""

    @pytest.fixture
    def populated_db(self):
        """Database with sample FOAF data"""
        db = GraphDB.in_memory()

        # Alice
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                Node.iri("http://xmlns.com/foaf/0.1/Person")
            ) \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/name"),
                Node.literal("Alice")
            ) \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/age"),
                Node.integer(30)
            ) \
            .execute()

        # Bob
        db.insert() \
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                Node.iri("http://xmlns.com/foaf/0.1/Person")
            ) \
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri("http://xmlns.com/foaf/0.1/name"),
                Node.literal("Bob")
            ) \
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri("http://xmlns.com/foaf/0.1/age"),
                Node.integer(25)
            ) \
            .execute()

        # Relationship
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/knows"),
                Node.iri("http://example.org/bob")
            ) \
            .execute()

        return db

    def test_select_all(self, populated_db):
        """Test: SELECT all triples"""
        results = populated_db.query() \
            .sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }") \
            .execute()

        assert len(results) == 7  # 7 triples total

    def test_select_with_pattern(self, populated_db):
        """Test: SELECT with specific pattern"""
        results = populated_db.query() \
            .sparql("""
                SELECT ?person ?name WHERE {
                    ?person <http://xmlns.com/foaf/0.1/name> ?name
                }
            """) \
            .execute()

        assert len(results) == 2  # Alice and Bob

    def test_select_with_type_filter(self, populated_db):
        """Test: SELECT filtering by rdf:type"""
        results = populated_db.query() \
            .sparql("""
                SELECT ?person WHERE {
                    ?person a <http://xmlns.com/foaf/0.1/Person>
                }
            """) \
            .execute()

        assert len(results) == 2

    def test_count_aggregation(self, populated_db):
        """Test: COUNT aggregation"""
        results = populated_db.query() \
            .sparql("""
                SELECT (COUNT(?s) as ?count) WHERE {
                    ?s a <http://xmlns.com/foaf/0.1/Person>
                }
            """) \
            .execute()

        assert len(results) == 1

    def test_empty_query_results(self):
        """Test: Query on empty database returns empty results"""
        db = GraphDB.in_memory()

        results = db.query() \
            .sparql("SELECT ?s WHERE { ?s ?p ?o }") \
            .execute()

        assert results.is_empty()
        assert len(results) == 0


class TestHypergraph:
    """Test hypergraph operations (beyond standard RDF triples)

    Hypergraphs support:
    - Binary edges (2 nodes)
    - Ternary edges (3 nodes - standard RDF triples)
    - N-ary edges (arbitrary number of nodes)
    """

    @pytest.fixture
    def db(self):
        return GraphDB.in_memory()

    def test_binary_hyperedge(self, db):
        """Test: Binary edge (2 nodes) - simple relationship"""
        # In RDF, represented as triple with predicate as middle node
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://example.org/likes"),  # Edge label
                Node.iri("http://example.org/pizza")
            ) \
            .execute()

        results = db.query() \
            .sparql("SELECT ?who ?what WHERE { ?who <http://example.org/likes> ?what }") \
            .execute()

        assert len(results) == 1

    def test_ternary_hyperedge_standard_triple(self, db):
        """Test: Ternary edge (3 nodes) - standard RDF triple"""
        # Subject-Predicate-Object pattern
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),      # Node 1: Subject
                Node.iri("http://xmlns.com/foaf/0.1/name"), # Node 2: Predicate
                Node.literal("Alice")                       # Node 3: Object
            ) \
            .execute()

        assert db.count() == 1

    def test_quaternary_hyperedge_named_graph(self, db):
        """Test: Quaternary edge (4 nodes) - RDF quad with named graph"""
        # Subject-Predicate-Object-Graph pattern
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/name"),
                Node.literal("Alice")
            ) \
            .graph(Node.iri("http://example.org/graph1")) \
            .execute()

        assert db.count() == 1

    def test_hyperedge_multiple_objects(self, db):
        """Test: One subject with multiple predicates/objects"""
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/name"),
                Node.literal("Alice")
            ) \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/age"),
                Node.integer(30)
            ) \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/email"),
                Node.literal("alice@example.org")
            ) \
            .execute()

        # Query all properties of Alice
        results = db.query() \
            .sparql("""
                SELECT ?p ?o WHERE {
                    <http://example.org/alice> ?p ?o
                }
            """) \
            .execute()

        assert len(results) == 3

    def test_hyperedge_traversal(self, db):
        """Test: Traversing connected hyperedges (graph pattern matching)"""
        # Create a path: Alice -> knows -> Bob -> knows -> Charlie
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/knows"),
                Node.iri("http://example.org/bob")
            ) \
            .triple(
                Node.iri("http://example.org/bob"),
                Node.iri("http://xmlns.com/foaf/0.1/knows"),
                Node.iri("http://example.org/charlie")
            ) \
            .execute()

        # Find friends of friends
        results = db.query() \
            .sparql("""
                SELECT ?friend_of_friend WHERE {
                    <http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> ?friend .
                    ?friend <http://xmlns.com/foaf/0.1/knows> ?friend_of_friend
                }
            """) \
            .execute()

        assert len(results) == 1  # Charlie

    def test_complex_hypergraph_pattern(self, db):
        """Test: Complex multi-edge pattern matching"""
        # Create a small social network
        db.insert() \
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://xmlns.com/foaf/0.1/name"), Node.literal("Alice")) \
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://xmlns.com/foaf/0.1/age"), Node.integer(30)) \
            .triple(Node.iri("http://example.org/bob"), Node.iri("http://xmlns.com/foaf/0.1/name"), Node.literal("Bob")) \
            .triple(Node.iri("http://example.org/bob"), Node.iri("http://xmlns.com/foaf/0.1/age"), Node.integer(25)) \
            .triple(Node.iri("http://example.org/alice"), Node.iri("http://xmlns.com/foaf/0.1/knows"), Node.iri("http://example.org/bob")) \
            .execute()

        # Complex query: Find people who know someone
        results = db.query() \
            .sparql("""
                SELECT ?person ?name ?knows WHERE {
                    ?person <http://xmlns.com/foaf/0.1/name> ?name .
                    ?person <http://xmlns.com/foaf/0.1/knows> ?knows
                }
            """) \
            .execute()

        assert len(results) == 1  # Alice knows Bob


class TestErrorHandling:
    """Test error cases and invalid operations"""

    @pytest.fixture
    def db(self):
        return GraphDB.in_memory()

    def test_empty_query_error(self, db):
        """Test: Query without SPARQL string should error"""
        with pytest.raises(Exception):  # Adjust exception type based on implementation
            db.query().execute()

    def test_invalid_sparql_syntax(self, db):
        """Test: Invalid SPARQL syntax should error"""
        with pytest.raises(Exception):
            db.query().sparql("INVALID SPARQL SYNTAX").execute()

    def test_query_on_empty_db_succeeds(self, db):
        """Test: Valid query on empty DB should succeed (just return no results)"""
        results = db.query() \
            .sparql("SELECT ?s WHERE { ?s ?p ?o }") \
            .execute()

        assert results.is_empty()


class TestPerformance:
    """Test performance with larger datasets"""

    @pytest.fixture
    def db(self):
        return GraphDB.in_memory()

    def test_insert_100_triples(self, db):
        """Test: Insert 100 triples"""
        for i in range(100):
            db.insert() \
                .triple(
                    Node.iri(f"http://example.org/entity{i}"),
                    Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                    Node.iri("http://example.org/Entity")
                ) \
                .execute()

        assert db.count() == 100

    def test_query_100_triples(self, db):
        """Test: Query across 100 triples"""
        # Insert data
        for i in range(100):
            db.insert() \
                .triple(
                    Node.iri(f"http://example.org/entity{i}"),
                    Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                    Node.iri("http://example.org/Entity")
                ) \
                .execute()

        # Query all
        results = db.query() \
            .sparql("SELECT ?s WHERE { ?s a <http://example.org/Entity> }") \
            .execute()

        assert len(results) == 100

    def test_bulk_insert_1000_triples(self, db):
        """Test: Bulk insert 1000 triples in batches"""
        # Insert in batches of 10
        for batch in range(100):
            builder = db.insert()
            for i in range(10):
                entity_num = batch * 10 + i
                builder.triple(
                    Node.iri(f"http://example.org/entity{entity_num}"),
                    Node.iri("http://example.org/index"),
                    Node.integer(entity_num)
                )
            builder.execute()

        assert db.count() == 1000


class TestBindingResults:
    """Test query result binding operations"""

    @pytest.fixture
    def populated_db(self):
        """Database with test data"""
        db = GraphDB.in_memory()
        db.insert() \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/name"),
                Node.literal("Alice")
            ) \
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri("http://xmlns.com/foaf/0.1/age"),
                Node.integer(30)
            ) \
            .execute()
        return db

    def test_binding_get_method(self, populated_db):
        """Test: Get variable value from binding"""
        results = populated_db.query() \
            .sparql("SELECT ?name WHERE { <http://example.org/alice> <http://xmlns.com/foaf/0.1/name> ?name }") \
            .execute()

        assert len(results) == 1
        binding = results[0]
        name = binding.get("name")
        assert name is not None
        assert "Alice" in name

    def test_binding_iteration(self, populated_db):
        """Test: Iterate over query results"""
        results = populated_db.query() \
            .sparql("SELECT ?p ?o WHERE { <http://example.org/alice> ?p ?o }") \
            .execute()

        count = 0
        for binding in results:
            assert "p" in binding
            assert "o" in binding
            count += 1

        assert count == 2  # name and age

    def test_binding_variables_property(self, populated_db):
        """Test: Get list of variable names from binding"""
        results = populated_db.query() \
            .sparql("SELECT ?s ?p WHERE { ?s ?p ?o }") \
            .execute()

        if len(results) > 0:
            binding = results[0]
            vars_list = binding.variables
            assert "s" in vars_list
            assert "p" in vars_list
