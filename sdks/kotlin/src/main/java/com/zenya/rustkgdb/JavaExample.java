package com.zenya.rustkgdb;

/**
 * Java interoperability example for rust-kgdb Kotlin SDK.
 *
 * This class demonstrates how to use the Kotlin SDK from Java code.
 * All Kotlin APIs are fully accessible from Java with idiomatic syntax.
 *
 * ## Usage
 *
 * ```bash
 * javac -cp "build/libs/*" src/main/java/com/zenya/rustkgdb/JavaExample.java
 * java -cp "build/libs/*:src/main/java" com.zenya.rustkgdb.JavaExample
 * ```
 */
public class JavaExample {

    public static void main(String[] args) {
        System.out.println("=== rust-kgdb Java Interop Example ===\n");

        // Create in-memory database
        GraphDB db = GraphDB.inMemory();
        System.out.println("✓ Created in-memory database");

        // Example 1: Basic CRUD
        basicCrudExample(db);

        // Example 2: Multiple triples
        multipleTriples Example(db);

        // Example 3: SPARQL queries
        sparqlQueryExample(db);

        // Example 4: Typed literals
        typedLiteralsExample(db);

        // Example 5: Language tags
        languageTagsExample(db);

        System.out.println("\n=== All examples completed successfully! ===");
    }

    /**
     * Example 1: Basic CRUD operations.
     */
    private static void basicCrudExample(GraphDB db) {
        System.out.println("\n--- Example 1: Basic CRUD ---");

        // Insert a triple
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(RDF.TYPE),
                Node.iri(FOAF.PERSON)
            )
            .execute();

        System.out.println("✓ Inserted 1 triple");
        System.out.println("  Triple count: " + db.count());

        // Clear for next example
        db.clear();
    }

    /**
     * Example 2: Inserting multiple triples.
     */
    private static void multipleTriplesExample(GraphDB db) {
        System.out.println("\n--- Example 2: Multiple Triples ---");

        // Insert multiple triples in one operation
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
            .execute();

        System.out.println("✓ Inserted 3 triples");
        System.out.println("  Triple count: " + db.count());
    }

    /**
     * Example 3: SPARQL queries.
     */
    private static void sparqlQueryExample(GraphDB db) {
        System.out.println("\n--- Example 3: SPARQL Queries ---");

        // Query for all names
        QueryResult results = db.query()
            .sparql("SELECT ?name WHERE { ?person <" + FOAF.NAME + "> ?name }")
            .execute();

        System.out.println("✓ Executed SPARQL query");
        System.out.println("  Results: " + results.getSize());

        // Iterate over results
        for (Binding binding : results) {
            String name = binding.get("name");
            System.out.println("  - Name: " + name);
        }

        // Alternative: index access
        if (!results.isEmpty()) {
            Binding firstResult = results.get(0);
            System.out.println("  First result (via index): " + firstResult.get("name"));
        }

        db.clear();
    }

    /**
     * Example 4: Typed literals.
     */
    private static void typedLiteralsExample(GraphDB db) {
        System.out.println("\n--- Example 4: Typed Literals ---");

        // Insert various typed literals
        db.insert()
            .triple(
                Node.iri("http://example.org/product"),
                Node.iri("http://example.org/price"),
                Node.doubleValue(19.99)
            )
            .triple(
                Node.iri("http://example.org/product"),
                Node.iri("http://example.org/quantity"),
                Node.integer(42)
            )
            .triple(
                Node.iri("http://example.org/product"),
                Node.iri("http://example.org/available"),
                Node.booleanValue(true)
            )
            .execute();

        System.out.println("✓ Inserted typed literals");
        System.out.println("  Types: double, integer, boolean");

        db.clear();
    }

    /**
     * Example 5: Language-tagged literals.
     */
    private static void languageTagsExample(GraphDB db) {
        System.out.println("\n--- Example 5: Language Tags ---");

        // Insert language-tagged literals
        db.insert()
            .triple(
                Node.iri("http://example.org/doc"),
                Node.iri(RDFS.LABEL),
                Node.langLiteral("Hello", "en")
            )
            .triple(
                Node.iri("http://example.org/doc"),
                Node.iri(RDFS.LABEL),
                Node.langLiteral("Bonjour", "fr")
            )
            .triple(
                Node.iri("http://example.org/doc"),
                Node.iri(RDFS.LABEL),
                Node.langLiteral("Hola", "es")
            )
            .execute();

        System.out.println("✓ Inserted language-tagged literals");
        System.out.println("  Languages: en, fr, es");

        db.clear();
    }

    /**
     * Example 6: Complex SPARQL query with FILTER.
     */
    private static void complexQueryExample(GraphDB db) {
        System.out.println("\n--- Example 6: Complex SPARQL ---");

        // Insert test data
        db.insert()
            .triple(Node.iri("http://example.org/alice"), Node.iri(FOAF.AGE), Node.integer(30))
            .triple(Node.iri("http://example.org/bob"), Node.iri(FOAF.AGE), Node.integer(25))
            .triple(Node.iri("http://example.org/charlie"), Node.iri(FOAF.AGE), Node.integer(35))
            .execute();

        // Query with FILTER
        QueryResult results = db.query()
            .sparql(
                "SELECT ?person ?age WHERE { " +
                "  ?person <" + FOAF.AGE + "> ?age . " +
                "  FILTER(?age > 28) " +
                "}"
            )
            .execute();

        System.out.println("✓ Executed query with FILTER");
        System.out.println("  Found " + results.getSize() + " people over 28");

        for (Binding binding : results) {
            System.out.println("  - " + binding.get("person") + ": " + binding.get("age"));
        }

        db.clear();
    }

    /**
     * Example 7: Using XSD datatypes explicitly.
     */
    private static void xsdDatatypesExample(GraphDB db) {
        System.out.println("\n--- Example 7: XSD Datatypes ---");

        // Using XSD constants
        db.insert()
            .triple(
                Node.iri("http://example.org/data"),
                Node.iri("http://example.org/stringValue"),
                Node.typedLiteral("text", XSD.STRING)
            )
            .triple(
                Node.iri("http://example.org/data"),
                Node.iri("http://example.org/intValue"),
                Node.typedLiteral("100", XSD.INTEGER)
            )
            .triple(
                Node.iri("http://example.org/data"),
                Node.iri("http://example.org/dateValue"),
                Node.typedLiteral("2025-11-28", XSD.DATE)
            )
            .execute();

        System.out.println("✓ Inserted XSD typed literals");
        System.out.println("  XSD types: string, integer, date");

        db.clear();
    }

    /**
     * Example 8: Blank nodes.
     */
    private static void blankNodesExample(GraphDB db) {
        System.out.println("\n--- Example 8: Blank Nodes ---");

        // Create anonymous nodes
        Node person1 = Node.blank("person1");
        Node person2 = Node.blank("person2");

        db.insert()
            .triple(person1, Node.iri(FOAF.NAME), Node.literal("Anonymous 1"))
            .triple(person2, Node.iri(FOAF.NAME), Node.literal("Anonymous 2"))
            .triple(person1, Node.iri(FOAF.KNOWS), person2)
            .execute();

        System.out.println("✓ Inserted blank nodes");
        System.out.println("  Triples: " + db.count());

        db.clear();
    }

    /**
     * Example 9: Named graphs.
     */
    private static void namedGraphsExample(GraphDB db) {
        System.out.println("\n--- Example 9: Named Graphs ---");

        // Insert into named graph
        db.insert()
            .graph(Node.iri("http://example.org/myGraph"))
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.NAME),
                Node.literal("Alice")
            )
            .execute();

        System.out.println("✓ Inserted into named graph");
        System.out.println("  Graph: http://example.org/myGraph");

        db.clear();
    }

    /**
     * Example 10: Vocabulary constants.
     */
    private static void vocabularyExample() {
        System.out.println("\n--- Example 10: Vocabulary Constants ---");

        System.out.println("RDF vocabulary:");
        System.out.println("  rdf:type = " + RDF.TYPE);
        System.out.println("  rdf:Property = " + RDF.PROPERTY);

        System.out.println("\nRDFS vocabulary:");
        System.out.println("  rdfs:Class = " + RDFS.CLASS);
        System.out.println("  rdfs:label = " + RDFS.LABEL);

        System.out.println("\nFOAF vocabulary:");
        System.out.println("  foaf:Person = " + FOAF.PERSON);
        System.out.println("  foaf:name = " + FOAF.NAME);

        System.out.println("\nXSD datatypes:");
        System.out.println("  xsd:string = " + XSD.STRING);
        System.out.println("  xsd:integer = " + XSD.INTEGER);
        System.out.println("  xsd:boolean = " + XSD.BOOLEAN);
    }
}
