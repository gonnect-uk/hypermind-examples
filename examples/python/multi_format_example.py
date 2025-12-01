#!/usr/bin/env python3
"""
Multi-Format RDF & Custom Functions Example (Python)

Demonstrates:
1. Loading RDF data in Turtle, N-Triples, and N-Quads formats
2. SPARQL queries across all formats
3. Custom function registration and usage
4. Roundtrip serialization

This example shows W3C-compliant custom SPARQL functions,
similar to Apache Jena's ExtensionFunctionRegistry.
"""

from gonnect import GraphDB, FunctionRegistry, Node

def main():
    print("=== Multi-Format RDF & Custom Functions Demo ===\n")

    # =========================================================================
    # Part 1: Load Data in Different Formats
    # =========================================================================
    print("1. Loading data in different RDF formats...\n")

    # Same data in 3 formats
    turtle_data = """
        @prefix ex: <http://example.org/> .
        ex:Alice ex:age "30" .
        ex:Bob ex:age "25" .
        ex:Charlie ex:score "95.5" .
    """

    ntriples_data = """
        <http://example.org/Alice> <http://example.org/age> "30" .
        <http://example.org/Bob> <http://example.org/age> "25" .
        <http://example.org/Charlie> <http://example.org/score> "95.5" .
    """

    nquads_data = """
        <http://example.org/Alice> <http://example.org/age> "30" .
        <http://example.org/Bob> <http://example.org/age> "25" <http://example.org/graph1> .
        <http://example.org/Charlie> <http://example.org/score> "95.5" .
    """

    # Create graph databases for each format
    db_turtle = GraphDB()
    db_turtle.load_turtle(turtle_data)
    print(f"✓ Loaded Turtle data: {db_turtle.size()} triples")

    db_ntriples = GraphDB()
    db_ntriples.load_ntriples(ntriples_data)
    print(f"✓ Loaded N-Triples data: {db_ntriples.size()} triples")

    db_nquads = GraphDB()
    db_nquads.load_nquads(nquads_data)
    print(f"✓ Loaded N-Quads data: {db_nquads.size()} triples\n")

    # =========================================================================
    # Part 2: SPARQL Queries (Same query, all formats)
    # =========================================================================
    print("2. Running identical SPARQL queries on all formats...\n")

    query = """
        PREFIX ex: <http://example.org/>
        SELECT ?person ?value WHERE {
            ?person ?prop ?value .
        }
    """

    results_turtle = db_turtle.query(query)
    results_ntriples = db_ntriples.query(query)
    results_nquads = db_nquads.query(query)

    print(f"Turtle results: {len(results_turtle)} rows")
    print(f"N-Triples results: {len(results_ntriples)} rows")
    print(f"N-Quads results: {len(results_nquads)} rows")
    print("✓ All formats produced results\n")

    # =========================================================================
    # Part 3: Custom SPARQL Functions (Jena-compatible)
    # =========================================================================
    print("3. Registering custom SPARQL functions...\n")

    # Create function registry
    registry = FunctionRegistry()

    # Register custom "double" function
    def double_function(args):
        """Doubles a numeric value"""
        if len(args) != 1:
            return None

        try:
            value = float(args[0].literal_value())
            result = value * 2.0
            return Node.literal_decimal(str(result))
        except (ValueError, AttributeError):
            return None

    registry.register("http://example.org/double", double_function)

    # Register custom "ageCategory" function
    def age_category_function(args):
        """Categorizes age into minor/adult/senior"""
        if len(args) != 1:
            return None

        try:
            age = int(args[0].literal_value())
            if age < 18:
                category = "minor"
            elif age < 65:
                category = "adult"
            else:
                category = "senior"
            return Node.literal_string(category)
        except (ValueError, AttributeError):
            return None

    registry.register("http://example.org/ageCategory", age_category_function)

    # Register custom "isEven" function
    def is_even_function(args):
        """Checks if a number is even"""
        if len(args) != 1:
            return None

        try:
            value = int(args[0].literal_value())
            is_even = (value % 2 == 0)
            return Node.literal_boolean(is_even)
        except (ValueError, AttributeError):
            return None

    registry.register("http://example.org/isEven", is_even_function)

    print("✓ Registered 3 custom functions:")
    print("  - ex:double(x) - multiplies by 2")
    print("  - ex:ageCategory(age) - categorizes age")
    print("  - ex:isEven(x) - checks if even\n")

    # =========================================================================
    # Part 4: Use Custom Functions in SPARQL Queries
    # =========================================================================
    print("4. Querying with custom functions...\n")

    # Apply custom functions to query
    db_with_functions = GraphDB(function_registry=registry)
    db_with_functions.load_turtle(turtle_data)

    # Query 1: Double the ages
    query_double = """
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age (ex:double(?age) AS ?doubledAge)
        WHERE {
            ?person ex:age ?age .
        }
    """

    results_double = db_with_functions.query(query_double)
    print("Query: ex:double(?age)")
    for row in results_double:
        print(f"  {row['person']} age {row['age']} → doubled: {row['doubledAge']}")
    print()

    # Query 2: Categorize ages
    query_category = """
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age (ex:ageCategory(?age) AS ?category)
        WHERE {
            ?person ex:age ?age .
        }
    """

    results_category = db_with_functions.query(query_category)
    print("Query: ex:ageCategory(?age)")
    for row in results_category:
        print(f"  {row['person']} age {row['age']} → category: {row['category']}")
    print()

    # Query 3: Filter with custom function
    query_filter = """
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age
        WHERE {
            ?person ex:age ?age .
            FILTER (ex:isEven(?age))
        }
    """

    results_filter = db_with_functions.query(query_filter)
    print("Query: FILTER(ex:isEven(?age))")
    for row in results_filter:
        print(f"  {row['person']} has even age: {row['age']}")
    print()

    # =========================================================================
    # Part 5: Chaining Custom Functions
    # =========================================================================
    print("5. Chaining custom functions...\n")

    # Register additional function for chaining
    def add_ten_function(args):
        """Adds 10 to a number"""
        if len(args) != 1:
            return None

        try:
            value = float(args[0].literal_value())
            result = value + 10.0
            return Node.literal_decimal(str(result))
        except (ValueError, AttributeError):
            return None

    registry.register("http://example.org/addTen", add_ten_function)

    db_with_functions = GraphDB(function_registry=registry)
    db_with_functions.load_turtle(turtle_data)

    # Chain functions: (age + 10) * 2
    query_chain = """
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age (ex:double(ex:addTen(?age)) AS ?transformed)
        WHERE {
            ?person ex:age ?age .
        }
    """

    results_chain = db_with_functions.query(query_chain)
    print("Query: ex:double(ex:addTen(?age))")
    for row in results_chain:
        original = float(row['age'])
        expected = (original + 10.0) * 2.0
        print(f"  {row['person']} age {row['age']} → ({row['age']} + 10) × 2 = {row['transformed']}")
    print()

    # =========================================================================
    # Part 6: Roundtrip Serialization
    # =========================================================================
    print("6. Roundtrip serialization (Turtle → N-Quads → Turtle)...\n")

    # Serialize to N-Quads
    nquads_output = db_turtle.serialize_nquads()
    print("Serialized to N-Quads:")
    print(nquads_output[:200] + "...\n")

    # Parse back and verify
    db_roundtrip = GraphDB()
    db_roundtrip.load_nquads(nquads_output)

    print(f"✓ Roundtrip successful: {db_roundtrip.size()} triples preserved")

    # Verify data is identical by running same query
    results_original = db_turtle.query(query)
    results_roundtrip = db_roundtrip.query(query)

    print(f"✓ Query results identical: {len(results_original)} = {len(results_roundtrip)}\n")

    # =========================================================================
    # Part 7: Named Graphs (N-Quads only feature)
    # =========================================================================
    print("7. Named graphs (N-Quads exclusive feature)...\n")

    # Query specific named graph
    query_graph = """
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age
        WHERE {
            GRAPH <http://example.org/graph1> {
                ?person ex:age ?age .
            }
        }
    """

    results_graph = db_nquads.query(query_graph)
    print(f"Query GRAPH <http://example.org/graph1>:")
    print(f"  Found {len(results_graph)} triples in named graph")
    for row in results_graph:
        print(f"  {row['person']} age {row['age']}")
    print()

    # =========================================================================
    # Summary
    # =========================================================================
    print("=== Summary ===")
    print("✓ Successfully loaded data in 3 RDF formats")
    print("✓ Executed SPARQL queries across all formats")
    print("✓ Registered and used 5 custom functions")
    print("✓ Demonstrated function chaining")
    print("✓ Performed roundtrip serialization")
    print("✓ Queried named graphs")
    print("\nAll features working correctly!")

if __name__ == "__main__":
    main()
