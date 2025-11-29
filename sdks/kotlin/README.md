# rust-kgdb Kotlin/Java SDK

Production-ready Kotlin and Java bindings for rust-kgdb RDF/SPARQL database.

## Installation

### Gradle (Kotlin DSL)

```kotlin
dependencies {
    implementation("com.zenya:rust-kgdb-kotlin:0.1.2")
}
```

### Gradle (Groovy)

```groovy
dependencies {
    implementation 'com.zenya:rust-kgdb-kotlin:0.1.2'
}
```

### Maven

```xml
<dependency>
    <groupId>com.zenya</groupId>
    <artifactId>rust-kgdb-kotlin</artifactId>
    <version>0.1.2</version>
</dependency>
```

## Quick Start (Kotlin)

```kotlin
import com.zenya.rustkgdb.*

fun main() {
    // Create database
    val db = GraphDB.inMemory()

    // Insert triples
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
        .execute()

    // Query with SPARQL
    val results = db.query()
        .sparql("SELECT ?name WHERE { ?person <${FOAF.NAME}> ?name }")
        .execute()

    for (binding in results) {
        println("Name: ${binding["name"]}")
    }
}
```

## Quick Start (Java)

```java
import com.zenya.rustkgdb.*;

public class Example {
    public static void main(String[] args) {
        // Create database
        GraphDB db = GraphDB.inMemory();

        // Insert triples
        db.insert()
            .triple(
                Node.iri("http://example.org/alice"),
                Node.iri(FOAF.NAME),
                Node.literal("Alice")
            )
            .execute();

        // Query with SPARQL
        QueryResult results = db.query()
            .sparql("SELECT ?name WHERE { ?person <" + FOAF.NAME + "> ?name }")
            .execute();

        for (Binding binding : results) {
            System.out.println("Name: " + binding.get("name"));
        }
    }
}
```

## Features

- ✅ Complete SPARQL 1.1 support
- ✅ Zero-copy performance (2.78 µs lookups)
- ✅ Type-safe Kotlin API with DSL
- ✅ Full Java interoperability
- ✅ Comprehensive test coverage (20 regression tests)
- ✅ Professional KDoc documentation
- ✅ Gradle & Maven support

## API Overview

### GraphDB

The main database interface:

```kotlin
val db = GraphDB.inMemory()          // Create in-memory database
db.count()                            // Count triples
db.isEmpty()                          // Check if empty
db.clear()                            // Clear all triples
db.insert()                           // Start insert builder
db.query()                            // Start query builder
```

### Node

Factory methods for creating RDF nodes:

```kotlin
// IRIs
Node.iri("http://example.org/resource")

// Literals
Node.literal("plain text")
Node.integer(42)
Node.double(3.14)
Node.boolean(true)
Node.date("2025-11-28")
Node.dateTime("2025-11-28T22:15:00Z")

// Language-tagged literals
Node.langLiteral("Bonjour", "fr")
Node.langLiteral("Hello", "en")

// Typed literals
Node.typedLiteral("value", XSD.STRING)
Node.typedLiteral("100", XSD.INTEGER)

// Blank nodes
Node.blank("b1")
```

### InsertBuilder

Fluent API for inserting triples:

```kotlin
db.insert()
    .triple(subject, predicate, object)
    .triple(subject2, predicate2, object2)
    .graph(Node.iri("http://example.org/myGraph"))  // Optional named graph
    .execute()
```

### QueryBuilder

Fluent API for SPARQL queries:

```kotlin
val results = db.query()
    .sparql("""
        PREFIX foaf: <http://xmlns.com/foaf/0.1/>
        SELECT ?name ?age WHERE {
            ?person foaf:name ?name .
            ?person foaf:age ?age .
            FILTER(?age > 25)
        }
    """)
    .execute()

// Iterate results
for (binding in results) {
    println("Name: ${binding["name"]}, Age: ${binding["age"]}")
}

// Access by index
println("First result: ${results[0]["name"]}")

// Check size
println("Found ${results.size} results")
```

## Vocabulary Constants

The SDK provides convenient constants for common RDF vocabularies:

### RDF

```kotlin
import com.zenya.rustkgdb.RDF

Node.iri(RDF.TYPE)           // rdf:type
Node.iri(RDF.PROPERTY)       // rdf:Property
Node.iri(RDF.LIST)           // rdf:List
Node.iri(RDF.FIRST)          // rdf:first
Node.iri(RDF.REST)           // rdf:rest
Node.iri(RDF.NIL)            // rdf:nil
```

### RDFS

```kotlin
import com.zenya.rustkgdb.RDFS

Node.iri(RDFS.CLASS)          // rdfs:Class
Node.iri(RDFS.SUB_CLASS_OF)   // rdfs:subClassOf
Node.iri(RDFS.LABEL)          // rdfs:label
Node.iri(RDFS.COMMENT)        // rdfs:comment
Node.iri(RDFS.DOMAIN)         // rdfs:domain
Node.iri(RDFS.RANGE)          // rdfs:range
```

### FOAF

```kotlin
import com.zenya.rustkgdb.FOAF

Node.iri(FOAF.PERSON)        // foaf:Person
Node.iri(FOAF.NAME)          // foaf:name
Node.iri(FOAF.KNOWS)         // foaf:knows
Node.iri(FOAF.AGE)           // foaf:age
Node.iri(FOAF.MBOX)          // foaf:mbox
Node.iri(FOAF.HOMEPAGE)      // foaf:homepage
```

### XSD Datatypes

```kotlin
import com.zenya.rustkgdb.XSD

Node.typedLiteral("value", XSD.STRING)
Node.typedLiteral("42", XSD.INTEGER)
Node.typedLiteral("3.14", XSD.DOUBLE)
Node.typedLiteral("true", XSD.BOOLEAN)
Node.typedLiteral("2025-11-28", XSD.DATE)
Node.typedLiteral("2025-11-28T22:15:00Z", XSD.DATE_TIME)
```

## Complete Examples

### CRUD Operations

```kotlin
val db = GraphDB.inMemory()

// Create
db.insert()
    .triple(
        Node.iri("http://example.org/alice"),
        Node.iri(RDF.TYPE),
        Node.iri(FOAF.PERSON)
    )
    .execute()

// Read
val results = db.query()
    .sparql("SELECT ?type WHERE { <http://example.org/alice> a ?type }")
    .execute()

// Count
println("Triples: ${db.count()}")

// Delete (via clear)
db.clear()
```

### Multiple Triples

```kotlin
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
        Node.iri(FOAF.KNOWS),
        Node.iri("http://example.org/bob")
    )
    .execute()
```

### Unicode Support

```kotlin
db.insert()
    .triple(
        Node.iri("http://example.org/doc"),
        Node.iri(RDFS.LABEL),
        Node.literal("Hello 世界 🌍")
    )
    .execute()
```

### Language Tags

```kotlin
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
```

### Complex SPARQL Query

```kotlin
val results = db.query()
    .sparql("""
        PREFIX foaf: <http://xmlns.com/foaf/0.1/>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        SELECT ?person ?name ?friend WHERE {
            ?person rdf:type foaf:Person .
            ?person foaf:name ?name .
            OPTIONAL { ?person foaf:knows ?friend }
            FILTER(STRLEN(?name) > 3)
        }
        ORDER BY ?name
        LIMIT 10
    """)
    .execute()

for (binding in results) {
    val person = binding["person"]
    val name = binding["name"]
    val friend = binding["friend"] ?: "none"
    println("$person: $name knows $friend")
}
```

## Java Interoperability

The Kotlin SDK is fully interoperable with Java. See `src/main/java/` for examples.

### Key Differences

| Feature | Kotlin | Java |
|---------|--------|------|
| Node creation | `Node.iri(...)` | `Node.iri(...)` (same) |
| Insert builder | `db.insert().triple(...).execute()` | Same |
| Query builder | `db.query().sparql(...).execute()` | Same |
| Iteration | `for (binding in results)` | `for (Binding binding : results)` |
| Nullable access | `binding["name"]` (nullable) | `binding.get("name")` (nullable) |

## Testing

```bash
# Run all tests
./gradlew test

# Run regression test suite only
./gradlew regression

# Run with detailed output
./gradlew test --info
```

## Documentation

```bash
# Generate KDoc documentation
./gradlew dokkaHtml

# Open documentation
open docs/api/index.html
```

## Performance

- **Lookup**: 2.78 µs per query
- **Memory**: 24 bytes per triple
- **Bulk Insert**: 146K triples/sec

## Requirements

- JVM 17 or higher
- Kotlin 1.9+ (for Kotlin projects)
- Gradle 8.0+ or Maven 3.6+

## Architecture

```
Kotlin Application
    ↓
rust_kgdb (Kotlin wrapper)
    ↓
UniFFI Generated Bindings
    ↓
mobile-ffi (Rust FFI layer)
    ↓
Core Engine (sparql + storage)
```

## License

MIT/Apache-2.0

## Links

- [Documentation](https://docs.rs/rust-kgdb-sdk)
- [GitHub](https://github.com/zenya-graphdb/rust-kgdb)
- [Maven Central](https://search.maven.org/artifact/com.zenya/rust-kgdb-kotlin)

## Support

- GitHub Issues: https://github.com/zenya-graphdb/rust-kgdb/issues
- Discussions: https://github.com/zenya-graphdb/rust-kgdb/discussions
