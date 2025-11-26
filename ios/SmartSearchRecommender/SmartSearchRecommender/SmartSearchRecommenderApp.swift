//
// SmartSearchRecommenderApp.swift
// SmartSearchRecommender
//
// App entry point - uses Gonnect NanoGraphDB (Rust FFI)
//

import SwiftUI

@main
struct SmartSearchRecommenderApp: App {
    init() {
        // Initialize Rust logging for debugging
        initializeLogging()
    }

    var body: some Scene {
        WindowGroup {
            MainContentView()
        }
    }
}

// MARK: - Main Content View (Rust FFI based)

struct MainContentView: View {
    @State private var graphDB: GraphDb?
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var queryResults: [QueryResult] = []
    @State private var selectedTab = 0

    var body: some View {
        NavigationView {
            ZStack {
                // Background gradient
                LinearGradient(
                    gradient: Gradient(colors: [Color(.systemBackground), Color(.systemGray6)]),
                    startPoint: .top,
                    endPoint: .bottom
                )
                .ignoresSafeArea()

                if isLoading {
                    LoadingView()
                } else if let error = errorMessage {
                    ErrorView(message: error, onRetry: loadDatabase)
                } else {
                    mainContent
                }
            }
            .navigationTitle("Smart Search")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    if let db = graphDB {
                        HStack(spacing: 4) {
                            Image(systemName: "cylinder.fill")
                                .foregroundColor(Color(hex: "#FF9500"))
                            Text("\(db.countTriples())")
                                .font(.caption.bold())
                                .foregroundColor(.secondary)
                        }
                    }
                }
            }
        }
        .task {
            loadDatabase()
        }
    }

    private var mainContent: some View {
        ScrollView {
            VStack(spacing: 24) {
                // Hero Stats Card
                if let db = graphDB {
                    HeroStatsCard(graphDB: db)
                }

                // Content Categories Card
                if let db = graphDB {
                    ContentCategoriesCard(graphDB: db)
                }

                // Query Actions
                QueryActionsCard(onQuery: executeQuery)

                // Results
                if !queryResults.isEmpty {
                    QueryResultsCard(results: queryResults)
                }

                // Reasoning Explanation
                ReasoningCard(graphDB: graphDB)

                // Performance Stats
                PerformanceCard()
            }
            .padding()
        }
    }

    private func loadDatabase() {
        isLoading = true
        errorMessage = nil

        DispatchQueue.global(qos: .userInitiated).async {
            do {
                let db = GraphDb()

                // Load movies catalog TTL from bundle
                if let url = Bundle.main.url(forResource: "movies_catalog", withExtension: "ttl") {
                    let content = try String(contentsOf: url, encoding: .utf8)
                    try db.loadTtl(ttlContent: content, graphName: "http://zenya.com/movies")
                }

                DispatchQueue.main.async {
                    self.graphDB = db
                    self.isLoading = false
                }
            } catch {
                DispatchQueue.main.async {
                    self.errorMessage = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }

    private func executeQuery(_ queryType: String) {
        guard let db = graphDB else { return }

        let sparql: String
        switch queryType {
        case "movies":
            sparql = """
                SELECT ?movie ?title ?year ?director WHERE {
                    ?movie <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Movie> .
                    ?movie <http://schema.org/name> ?title .
                    OPTIONAL { ?movie <http://schema.org/datePublished> ?year }
                    OPTIONAL {
                        ?movie <http://schema.org/director> ?dirRef .
                        ?dirRef <http://schema.org/name> ?director
                    }
                }
                LIMIT 20
            """
        case "actors":
            sparql = """
                SELECT ?person ?name WHERE {
                    ?person <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
                    ?person <http://schema.org/name> ?name .
                    ?movie <http://schema.org/actor> ?person .
                }
                LIMIT 20
            """
        case "genres":
            sparql = """
                SELECT DISTINCT ?genre (COUNT(?movie) AS ?count) WHERE {
                    ?movie <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Movie> .
                    ?movie <http://schema.org/genre> ?genre .
                }
                GROUP BY ?genre
                ORDER BY DESC(?count)
            """
        default:
            return
        }

        DispatchQueue.global(qos: .userInitiated).async {
            do {
                let results = try db.querySelect(sparql: sparql)
                DispatchQueue.main.async {
                    self.queryResults = results
                }
            } catch {
                DispatchQueue.main.async {
                    self.errorMessage = error.localizedDescription
                }
            }
        }
    }
}

// MARK: - Hero Stats Card

struct HeroStatsCard: View {
    let graphDB: GraphDb

    var body: some View {
        VStack(spacing: 16) {
            HStack {
                Image(systemName: "sparkles")
                    .font(.title2)
                    .foregroundColor(.white)
                Text("Content Catalog")
                    .font(.headline)
                    .foregroundColor(.white)
                Spacer()
            }

            HStack(spacing: 20) {
                StatItem(
                    value: "\(graphDB.countTriples())",
                    label: "Triples",
                    icon: "cylinder.fill"
                )
                StatItem(
                    value: "\(graphDB.countEntities())",
                    label: "Entities",
                    icon: "circle.grid.3x3.fill"
                )
                StatItem(
                    value: "\(graphDB.countByType(typeFilter: "Movie"))",
                    label: "Movies",
                    icon: "film.fill"
                )
                StatItem(
                    value: "\(graphDB.countByType(typeFilter: "Person"))",
                    label: "People",
                    icon: "person.2.fill"
                )
            }
        }
        .padding(20)
        .background(
            LinearGradient(
                gradient: Gradient(colors: [Color(hex: "#FF9500"), Color(hex: "#FF6B00")]),
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .cornerRadius(16)
        .shadow(color: Color(hex: "#FF9500").opacity(0.3), radius: 10, y: 5)
    }
}

struct StatItem: View {
    let value: String
    let label: String
    let icon: String

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.title3)
                .foregroundColor(.white.opacity(0.8))
            Text(value)
                .font(.title2.bold())
                .foregroundColor(.white)
            Text(label)
                .font(.caption)
                .foregroundColor(.white.opacity(0.8))
        }
        .frame(maxWidth: .infinity)
    }
}

// MARK: - Content Categories Card

struct ContentCategoriesCard: View {
    let graphDB: GraphDb

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Image(systemName: "square.grid.2x2.fill")
                    .foregroundColor(.orange)
                Text("Content Types")
                    .font(.headline)
                Spacer()
            }

            VStack(spacing: 12) {
                CategoryBar(
                    label: "Movies",
                    count: Int(graphDB.countByType(typeFilter: "Movie")),
                    color: .blue,
                    icon: "film.fill"
                )
                CategoryBar(
                    label: "Actors",
                    count: Int(graphDB.countByType(typeFilter: "Person")),
                    color: .green,
                    icon: "person.fill"
                )
                CategoryBar(
                    label: "Genres",
                    count: 10, // Estimate
                    color: .purple,
                    icon: "tag.fill"
                )
            }
        }
        .padding(20)
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 10, y: 5)
    }
}

struct CategoryBar: View {
    let label: String
    let count: Int
    let color: Color
    let icon: String

    var body: some View {
        HStack {
            Image(systemName: icon)
                .foregroundColor(color)
                .frame(width: 24)
            Text(label)
                .font(.subheadline)
            Spacer()
            Text("\(count)")
                .font(.headline.bold())
                .foregroundColor(color)
        }
        .padding(.vertical, 8)
        .padding(.horizontal, 12)
        .background(color.opacity(0.1))
        .cornerRadius(8)
    }
}

// MARK: - Query Actions Card

struct QueryActionsCard: View {
    let onQuery: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundColor(.blue)
                Text("SPARQL Queries")
                    .font(.headline)
                Spacer()
            }

            VStack(spacing: 10) {
                QueryButton(
                    title: "Browse Movies",
                    subtitle: "Discover films with metadata",
                    icon: "film.fill",
                    color: .blue
                ) { onQuery("movies") }

                QueryButton(
                    title: "Find Actors",
                    subtitle: "Search cast members",
                    icon: "person.2.fill",
                    color: .green
                ) { onQuery("actors") }

                QueryButton(
                    title: "Explore Genres",
                    subtitle: "Content by category",
                    icon: "tag.fill",
                    color: .purple
                ) { onQuery("genres") }
            }
        }
        .padding(20)
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 10, y: 5)
    }
}

struct QueryButton: View {
    let title: String
    let subtitle: String
    let icon: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack {
                Image(systemName: icon)
                    .font(.title3)
                    .foregroundColor(color)
                    .frame(width: 32)

                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.subheadline.bold())
                        .foregroundColor(.primary)
                    Text(subtitle)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                Image(systemName: "chevron.right")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .padding(12)
            .background(Color(.systemGray6))
            .cornerRadius(10)
        }
    }
}

// MARK: - Query Results Card

struct QueryResultsCard: View {
    let results: [QueryResult]

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Image(systemName: "list.bullet.rectangle.fill")
                    .foregroundColor(.green)
                Text("Query Results")
                    .font(.headline)
                Spacer()
                Text("\(results.count) results")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            ForEach(Array(results.enumerated()), id: \.offset) { index, result in
                ResultRow(result: result, index: index)
            }
        }
        .padding(20)
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 10, y: 5)
    }
}

struct ResultRow: View {
    let result: QueryResult
    let index: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            ForEach(Array(result.bindings.sorted(by: { $0.key < $1.key })), id: \.key) { key, value in
                HStack {
                    Text(key)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .frame(width: 80, alignment: .leading)
                    Text(formatValue(value))
                        .font(.subheadline)
                        .lineLimit(2)
                }
            }
        }
        .padding(12)
        .background(Color(.systemGray6))
        .cornerRadius(8)
    }

    private func formatValue(_ value: String) -> String {
        if let lastSlash = value.lastIndex(of: "/") {
            return String(value[value.index(after: lastSlash)...])
        }
        if let lastHash = value.lastIndex(of: "#") {
            return String(value[value.index(after: lastHash)...])
        }
        return value
    }
}

// MARK: - Reasoning Card

struct ReasoningCard: View {
    let graphDB: GraphDb?
    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Button(action: { withAnimation(.spring()) { isExpanded.toggle() } }) {
                HStack {
                    Image(systemName: "brain.head.profile")
                        .foregroundColor(Color(hex: "#FF9500"))
                    Text("Recommendation Engine")
                        .font(.headline)
                        .foregroundColor(.primary)
                    Spacer()
                    Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                        .foregroundColor(.secondary)
                }
            }

            if isExpanded {
                VStack(alignment: .leading, spacing: 16) {
                    ReasoningStep(
                        number: 1,
                        title: "Schema.org Ontology",
                        description: "Content modeled with standard vocab",
                        example: "Movie, Person, Genre",
                        color: .blue
                    )

                    ReasoningStep(
                        number: 2,
                        title: "Relationship Graphs",
                        description: "Actors, directors, genres linked",
                        example: "Movie -> actor -> Person",
                        color: .green
                    )

                    ReasoningStep(
                        number: 3,
                        title: "Pattern Matching",
                        description: "SPARQL finds related content",
                        example: "?movie schema:genre ?genre",
                        color: .orange
                    )

                    ReasoningStep(
                        number: 4,
                        title: "Smart Recommendations",
                        description: "Graph traversal for suggestions",
                        example: "Similar movies via shared actors",
                        color: .red
                    )

                    if let db = graphDB {
                        let stats = db.getStats()
                        HStack {
                            VStack(alignment: .leading) {
                                Text("Live Statistics")
                                    .font(.caption.bold())
                                Text("\(stats.totalTriples) triples - \(stats.totalEntities) entities")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            Spacer()
                            Text("Sub-ms")
                                .font(.caption.bold())
                                .foregroundColor(Color(hex: "#FF9500"))
                        }
                        .padding(12)
                        .background(Color(hex: "#FF9500").opacity(0.1))
                        .cornerRadius(8)
                    }
                }
            }
        }
        .padding(20)
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 10, y: 5)
    }
}

struct ReasoningStep: View {
    let number: Int
    let title: String
    let description: String
    let example: String
    let color: Color

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Text("\(number)")
                .font(.caption.bold())
                .foregroundColor(.white)
                .frame(width: 24, height: 24)
                .background(color)
                .cornerRadius(12)

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.subheadline.bold())
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text(example)
                    .font(.caption)
                    .foregroundColor(color)
                    .padding(.vertical, 4)
            }
        }
    }
}

// MARK: - Performance Card

struct PerformanceCard: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Image(systemName: "bolt.fill")
                    .foregroundColor(.yellow)
                Text("NanoGraphDB Performance")
                    .font(.headline)
                Spacer()
            }

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                PerfStat(label: "Lookup", value: "2.78 us")
                PerfStat(label: "Insert", value: "146K/sec")
                PerfStat(label: "Memory", value: "24 B/triple")
                PerfStat(label: "vs RDFox", value: "35-180x")
            }
        }
        .padding(20)
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 10, y: 5)
    }
}

struct PerfStat: View {
    let label: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
            Text(value)
                .font(.subheadline.bold())
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(Color(.systemGray6))
        .cornerRadius(8)
    }
}

// MARK: - Loading View

struct LoadingView: View {
    var body: some View {
        VStack(spacing: 20) {
            ProgressView()
                .scaleEffect(1.5)
            Text("Loading Movie Catalog...")
                .font(.headline)
                .foregroundColor(.secondary)
            Text("Powered by Rust NanoGraphDB")
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }
}

// MARK: - Error View

struct ErrorView: View {
    let message: String
    let onRetry: () -> Void

    var body: some View {
        VStack(spacing: 20) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 50))
                .foregroundColor(.red)
            Text("Error Loading Data")
                .font(.headline)
            Text(message)
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)
            Button("Retry", action: onRetry)
                .buttonStyle(.borderedProminent)
        }
    }
}

// MARK: - Color Extension

extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 3:
            (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6:
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8:
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (255, 0, 0, 0)
        }
        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue: Double(b) / 255,
            opacity: Double(a) / 255
        )
    }
}

#Preview {
    MainContentView()
}
