# Kotlin SDK Coming Soon

The Kotlin SDK extends rust-kgdb to the Android and JVM ecosystems with native language bindings.

## Development Timeline

| Phase | Timeline | Status |
|-------|----------|--------|
| Initial Kotlin bindings | Q2 2025 | Planning |
| Android integration | Q2-Q3 2025 | Pending |
| Full feature parity | Q3 2025 | Pending |
| Production release | Q3 2025 | Pending |

## What We're Building

### Native Kotlin Bindings
Professional-grade Kotlin FFI bindings with full type safety and null safety.

### Android Optimizations
- Memory-efficient for mobile devices
- Battery-aware query execution
- Integration with Android lifecycle
- WorkManager support for background tasks

### JVM Support
- Seamless Maven/Gradle integration
- Spring Boot starter module
- Quarkus integration
- GraalVM native image support

## Use Cases

### Android Knowledge Graph App

```kotlin
class MainActivity : AppCompatActivity() {
    private val viewModel by viewModels<KnowledgeGraphViewModel>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        viewModel.queryResults.observe(this) { results ->
            displayResults(results)
        }

        viewModel.searchGraph("SELECT ?person WHERE { ... }")
    }
}
```

### JVM Microservice

```kotlin
import io.zenya.kgdb.RDFStore
import io.micronaut.http.annotation.Controller
import io.micronaut.http.annotation.Get

@Controller("/api")
class GraphController(private val store: RDFStore) {

    @Get("/search")
    suspend fun search(query: String): List<Binding> {
        return store.sparqlQueryAsync(query)
    }
}
```

### Spring Boot Integration

```kotlin
@Configuration
class GraphDBConfiguration {

    @Bean
    fun rdfStore(): RDFStore {
        return RDFStore.new("rocksdb:///data/kg")
    }

    @Bean
    fun graphService(store: RDFStore): GraphService {
        return GraphService(store)
    }
}

@Service
class GraphService(private val store: RDFStore) {

    suspend fun findSimilar(entity: String, threshold: Double): List<String> {
        val query = """
            PREFIX zenya: <http://zenya.com/>
            SELECT ?similar WHERE {
                ?similar zenya:similarTo ("$entity"^^xsd:string $threshold)
            }
        """.trimIndent()
        return store.sparqlQueryAsync(query).map { it["similar"].toString() }
    }
}
```

## Key Features

### Coroutine Support
Full async/await with Kotlin coroutines:

```kotlin
viewModelScope.launch {
    val results = store.sparqlQueryAsync(query)
    updateUI(results)
}
```

### LiveData Integration
Observable results for reactive UI:

```kotlin
val graphResults: LiveData<List<Binding>> = store.sparqlQueryLive(query)
```

### Type-Safe Builders
Construct SPARQL queries safely:

```kotlin
val query = sparqlSelect {
    select("?person", "?name")
    where {
        triple("?person", "rdf:type", "ex:Person")
        triple("?person", "ex:name", "?name")
        filter("?age > 30")
    }
}

store.sparqlQueryAsync(query)
```

### Pagination Support
Efficient result streaming:

```kotlin
store.sparqlQueryPaginated(query, pageSize = 50).collect { page ->
    displayPage(page)
}
```

## Performance Characteristics

Kotlin SDK maintains rust-kgdb's performance:
- **2.78 µs** per lookup
- **24 bytes** per triple
- **146K triples/sec** bulk insert

Plus Kotlin-specific optimizations:
- **Zero JNI overhead** through careful binding design
- **Memory pooling** for Android
- **GC-friendly** data structures

## Current Workarounds

Until the Kotlin SDK is released, you can:

1. **Use JNI bindings**: Call Rust code directly from Java/Kotlin
2. **REST API**: Deploy server and query via HTTP
3. **Embedded Jena**: Use Apache Jena as temporary solution

## Testing Support

Comprehensive testing utilities:

```kotlin
@ExtendWith(RDFStoreExtension::class)
class KnowledgeGraphTest {

    @Test
    fun `should query known triples`(store: RDFStore) {
        store.insert("""
            PREFIX ex: <http://example.org/>
            INSERT DATA {
                ex:Alice ex:knows ex:Bob
            }
        """)

        val results = store.sparqlQuery("SELECT ?x WHERE { ?x ex:knows ?y }")
        assertEquals(1, results.size)
    }
}
```

## Android Studio Plugin

Planned IDE integration:

```xml
<!-- Kotlin syntax highlighting for SPARQL strings -->
val query = sparql("""
    SELECT ?x WHERE {
        ?x <http://example.org/type> <http://example.org/Class>
    }
""")
```

## FAQ

**Q: When will Kotlin SDK be available?**
A: Target Q3 2025. Android integration phase begins Q2.

**Q: Will it support Android Wear?**
A: Limited support initially (Q4 2025), full support in v1.1.

**Q: Is it free?**
A: Yes, open source under the same license as rust-kgdb.

**Q: What's the minimum Android version?**
A: Android 8.0+ (API level 26)

**Q: Does it work with Java?**
A: Yes, via Kotlin interop, but Kotlin is recommended.

**Q: Will it support Compose?**
A: Yes, Compose integration is planned.

## Contributing

Help shape the Kotlin SDK:

1. Star the [GitHub repository](https://github.com/zenya-graphdb/rust-kgdb)
2. Comment on Kotlin SDK [issues](https://github.com/zenya-graphdb/rust-kgdb/issues)
3. Join the community discussions

## Resources

- [Rust SDK Documentation](../rust/index.md) - Available now
- [Main Repository](https://github.com/zenya-graphdb/rust-kgdb)
- [Issue Tracker](https://github.com/zenya-graphdb/rust-kgdb/issues)
- [Kotlin Coroutines](https://kotlinlang.org/docs/coroutines-overview.html)

## Get Notified

1. Star the GitHub repository
2. Watch releases
3. Subscribe to announcements

We can't wait to bring rust-kgdb to Kotlin and Android developers!
