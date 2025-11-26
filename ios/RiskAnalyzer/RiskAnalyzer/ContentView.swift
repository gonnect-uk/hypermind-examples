//
// ContentView.swift
// RiskAnalyzer - Insurance Risk Analysis App
//
// Powered by Gonnect NanoGraphDB (Rust)
// Uses RDF/SPARQL for intelligent risk reasoning
//

import SwiftUI

struct ContentView: View {
    @State private var graphDB: GraphDb?
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var stats: DatabaseStats?
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
            .navigationTitle("Risk Analyzer")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    if let db = graphDB {
                        HStack(spacing: 4) {
                            Image(systemName: "cylinder.fill")
                                .foregroundColor(.purple)
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

                // Risk Breakdown
                if let db = graphDB {
                    RiskBreakdownCard(graphDB: db)
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
                // Create GraphDB instance with app-specific graph URI
                let db = GraphDb(appGraphUri: "http://zenya.com/insurance")

                // Load TTL from bundle (automatically scoped to app's graph)
                if let url = Bundle.main.url(forResource: "insurance-policies", withExtension: "ttl") {
                    let content = try String(contentsOf: url, encoding: .utf8)
                    try db.loadTtl(ttlContent: content)
                }

                let dbStats = db.getStats()

                DispatchQueue.main.async {
                    self.graphDB = db
                    self.stats = dbStats
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
        NSLog("DEBUG: executeQuery called with type: %@", queryType)
        guard let db = graphDB else {
            NSLog("DEBUG: graphDB is nil!")
            errorMessage = "Database not loaded"
            return
        }
        NSLog("DEBUG: graphDB exists, triple count: %d", db.countTriples())

        let sparql: String
        switch queryType {
        case "policies":
            sparql = "SELECT ?policy ?risk WHERE { GRAPH <http://zenya.com/insurance> { ?policy <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/domain/insurance/Policy> . ?policy <http://zenya.com/domain/insurance/riskLevel> ?risk } } LIMIT 50"
        case "violations":
            sparql = "SELECT ?violation ?type WHERE { GRAPH <http://zenya.com/insurance> { ?violation <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/domain/insurance/PolicyViolation> . ?violation <http://zenya.com/domain/insurance/violationType> ?type } } LIMIT 50"
        case "high-risk":
            sparql = "SELECT ?policy ?score WHERE { GRAPH <http://zenya.com/insurance> { ?policy <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/domain/insurance/Policy> . ?policy <http://zenya.com/domain/insurance/riskScore> ?score } } LIMIT 50"
        default:
            print("DEBUG: Unknown query type: \(queryType)")
            return
        }

        print("DEBUG: Executing query type '\(queryType)'")
        print("DEBUG: SPARQL: \(sparql)")

        DispatchQueue.global(qos: .userInitiated).async {
            do {
                let results = try db.querySelect(sparql: sparql)
                NSLog("DEBUG: Query returned %d results", results.count)
                for (i, result) in results.prefix(3).enumerated() {
                    NSLog("DEBUG: Result %d: %@", i, result.bindings.description)
                }
                DispatchQueue.main.async {
                    self.queryResults = results
                    NSLog("DEBUG: Updated queryResults, count: %d", self.queryResults.count)
                }
            } catch {
                DispatchQueue.main.async {
                    self.errorMessage = "Query failed: \(error.localizedDescription)"
                    NSLog("DEBUG: SPARQL Query Error: %@", error.localizedDescription)
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
                Image(systemName: "chart.bar.fill")
                    .font(.title2)
                    .foregroundColor(.white)
                Text("Knowledge Graph Overview")
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
                    value: "\(graphDB.countByType(typeFilter: "Policy"))",
                    label: "Policies",
                    icon: "doc.text.fill"
                )
                StatItem(
                    value: "\(graphDB.countByType(typeFilter: "PolicyViolation"))",
                    label: "Violations",
                    icon: "exclamationmark.triangle.fill"
                )
            }
        }
        .padding(20)
        .background(
            LinearGradient(
                gradient: Gradient(colors: [Color.purple, Color.blue]),
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .cornerRadius(16)
        .shadow(color: .purple.opacity(0.3), radius: 10, y: 5)
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

// MARK: - Risk Breakdown Card

struct RiskBreakdownCard: View {
    let graphDB: GraphDb

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Image(systemName: "gauge.with.dots.needle.bottom.50percent")
                    .foregroundColor(.orange)
                Text("Risk Distribution")
                    .font(.headline)
                Spacer()
            }

            VStack(spacing: 12) {
                RiskBar(
                    label: "High Risk",
                    count: Int(graphDB.countTriplesFiltered(predicateFilter: "riskLevel", objectFilter: "High")),
                    color: .red,
                    icon: "flame.fill"
                )
                RiskBar(
                    label: "Medium Risk",
                    count: Int(graphDB.countTriplesFiltered(predicateFilter: "riskLevel", objectFilter: "Medium")),
                    color: .orange,
                    icon: "exclamationmark.circle.fill"
                )
                RiskBar(
                    label: "Low Risk",
                    count: Int(graphDB.countTriplesFiltered(predicateFilter: "riskLevel", objectFilter: "Low")),
                    color: .green,
                    icon: "checkmark.circle.fill"
                )
            }
        }
        .padding(20)
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 10, y: 5)
    }
}

struct RiskBar: View {
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
                    title: "List All Policies",
                    subtitle: "Show policy details with risk scores",
                    icon: "doc.text.fill",
                    color: .blue
                ) {
                    NSLog("DEBUG: QueryButton 'policies' tapped")
                    onQuery("policies")
                }

                QueryButton(
                    title: "View Violations",
                    subtitle: "Policy violations detected by rules",
                    icon: "exclamationmark.triangle.fill",
                    color: .orange
                ) {
                    NSLog("DEBUG: QueryButton 'violations' tapped")
                    onQuery("violations")
                }

                QueryButton(
                    title: "High Risk Analysis",
                    subtitle: "Policies requiring immediate attention",
                    icon: "flame.fill",
                    color: .red
                ) {
                    NSLog("DEBUG: QueryButton 'high-risk' tapped")
                    onQuery("high-risk")
                }
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
        // Extract local name from URI
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
                        .foregroundColor(.purple)
                    Text("How Graph Reasoning Works")
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
                        title: "RDF Triple Store",
                        description: "Data stored as Subject-Predicate-Object facts",
                        example: "<Policy001> <riskLevel> \"High\"",
                        color: .blue
                    )

                    ReasoningStep(
                        number: 2,
                        title: "Type Classification",
                        description: "Entities classified via rdf:type predicate",
                        example: "<Policy001> a ins:Policy",
                        color: .green
                    )

                    ReasoningStep(
                        number: 3,
                        title: "Pattern Matching",
                        description: "SPARQL queries find matching patterns",
                        example: "?policy ins:riskLevel \"High\"",
                        color: .orange
                    )

                    ReasoningStep(
                        number: 4,
                        title: "Rule-Based Reasoning",
                        description: "Violations detected by graph traversal",
                        example: "Policy → Customer → Age → Rules",
                        color: .red
                    )

                    if let db = graphDB {
                        let stats = db.getStats()
                        HStack {
                            VStack(alignment: .leading) {
                                Text("Live Statistics")
                                    .font(.caption.bold())
                                Text("\(stats.totalTriples) triples • \(stats.totalEntities) entities")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            Spacer()
                            Text("Sub-ms")
                                .font(.caption.bold())
                                .foregroundColor(.purple)
                        }
                        .padding(12)
                        .background(Color.purple.opacity(0.1))
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
        let stats = getPerformanceStats()

        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Image(systemName: "bolt.fill")
                    .foregroundColor(.yellow)
                Text("NanoGraphDB Performance")
                    .font(.headline)
                Spacer()
            }

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                PerfStat(label: "Lookup", value: stats.lookupSpeed)
                PerfStat(label: "Insert", value: stats.bulkInsertSpeed)
                PerfStat(label: "Memory", value: stats.memoryPerTriple)
                PerfStat(label: "vs RDFox", value: stats.vsRdfoxLookup)
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
            Text("Loading Knowledge Graph...")
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

#Preview {
    ContentView()
}
