//
// ContentView.swift
// ComplianceChecker - Financial Compliance Verification App
//
// Powered by Gonnect NanoGraphDB (Rust)
// Uses RDF/SPARQL for intelligent compliance reasoning
//

import SwiftUI

struct ContentView: View {
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
            .navigationTitle("Compliance Checker")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    if let db = graphDB {
                        HStack(spacing: 4) {
                            Image(systemName: "cylinder.fill")
                                .foregroundColor(Color(hex: "#5856D6"))
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

                // Compliance Status Breakdown
                if let db = graphDB {
                    ComplianceStatusCard(graphDB: db)
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

                // Load TTL from bundle into named graph
                if let url = Bundle.main.url(forResource: "financial_compliance", withExtension: "ttl") {
                    let content = try String(contentsOf: url, encoding: .utf8)
                    try db.loadTtl(ttlContent: content, graphName: "http://zenya.com/compliance")
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
        case "transactions":
            sparql = """
                SELECT ?transaction ?type ?amount ?date ?status WHERE {
                    ?transaction <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://finregont.com/ontology#Transaction> .
                    ?transaction <http://finregont.com/ontology#transactionType> ?type .
                    ?transaction <http://finregont.com/ontology#amount> ?amount .
                    ?transaction <http://finregont.com/ontology#transactionDate> ?date .
                    ?transaction <http://finregont.com/ontology#status> ?status .
                }
            """
        case "violations":
            sparql = """
                SELECT ?violation ?rule ?severity ?description WHERE {
                    ?violation <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/compliance#Violation> .
                    ?violation <http://example.org/compliance#violatesRule> ?ruleRef .
                    ?ruleRef <http://www.w3.org/2000/01/rdf-schema#label> ?rule .
                    ?violation <http://example.org/compliance#severity> ?severity .
                    ?violation <http://example.org/compliance#description> ?description .
                }
            """
        case "rules":
            sparql = """
                SELECT ?rule ?label ?jurisdiction ?riskLevel WHERE {
                    ?rule <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://finregont.com/ontology#Regulation> .
                    ?rule <http://www.w3.org/2000/01/rdf-schema#label> ?label .
                    ?rule <http://finregont.com/ontology#jurisdiction> ?jurisdiction .
                    ?rule <http://finregont.com/ontology#riskLevel> ?riskLevel .
                }
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
                Image(systemName: "chart.bar.fill")
                    .font(.title2)
                    .foregroundColor(.white)
                Text("Compliance Overview")
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
                    value: "\(graphDB.countByType(typeFilter: "Regulation"))",
                    label: "Rules",
                    icon: "list.bullet.rectangle.fill"
                )
                StatItem(
                    value: "\(graphDB.countByType(typeFilter: "Violation"))",
                    label: "Violations",
                    icon: "exclamationmark.triangle.fill"
                )
            }
        }
        .padding(20)
        .background(
            LinearGradient(
                gradient: Gradient(colors: [Color(hex: "#5856D6"), Color(hex: "#7B68EE")]),
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .cornerRadius(16)
        .shadow(color: Color(hex: "#5856D6").opacity(0.3), radius: 10, y: 5)
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

// MARK: - Compliance Status Card

struct ComplianceStatusCard: View {
    let graphDB: GraphDb

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Image(systemName: "checkmark.shield.fill")
                    .foregroundColor(.green)
                Text("Risk Level Distribution")
                    .font(.headline)
                Spacer()
            }

            VStack(spacing: 12) {
                StatusBar(
                    label: "Critical",
                    count: Int(graphDB.countTriplesFiltered(predicateFilter: "riskLevel", objectFilter: "Critical")),
                    color: .red,
                    icon: "flame.fill"
                )
                StatusBar(
                    label: "High",
                    count: Int(graphDB.countTriplesFiltered(predicateFilter: "riskLevel", objectFilter: "High")),
                    color: .orange,
                    icon: "exclamationmark.circle.fill"
                )
                StatusBar(
                    label: "Medium",
                    count: Int(graphDB.countTriplesFiltered(predicateFilter: "riskLevel", objectFilter: "Medium")),
                    color: .yellow,
                    icon: "info.circle.fill"
                )
            }
        }
        .padding(20)
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 10, y: 5)
    }
}

struct StatusBar: View {
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
                    title: "View Transactions",
                    subtitle: "Financial transactions with compliance status",
                    icon: "creditcard.fill",
                    color: .blue
                ) { onQuery("transactions") }

                QueryButton(
                    title: "Check Violations",
                    subtitle: "Compliance violations and rule breaches",
                    icon: "exclamationmark.triangle.fill",
                    color: .red
                ) { onQuery("violations") }

                QueryButton(
                    title: "List Rules",
                    subtitle: "Regulatory frameworks and requirements",
                    icon: "list.bullet.rectangle.fill",
                    color: Color(hex: "#5856D6")
                ) { onQuery("rules") }
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
                        .foregroundColor(Color(hex: "#5856D6"))
                    Text("How Compliance Reasoning Works")
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
                        title: "Regulatory Knowledge",
                        description: "Rules encoded as RDF triples",
                        example: "<SEC_10b5> rdf:type Regulation",
                        color: .blue
                    )

                    ReasoningStep(
                        number: 2,
                        title: "Transaction Analysis",
                        description: "Financial data linked to regulations",
                        example: "<TXN001> comp:subject SEC_10b5",
                        color: .green
                    )

                    ReasoningStep(
                        number: 3,
                        title: "Violation Detection",
                        description: "SPARQL patterns identify breaches",
                        example: "?txn comp:violates ?rule",
                        color: .orange
                    )

                    ReasoningStep(
                        number: 4,
                        title: "Risk Assessment",
                        description: "Severity computed via graph traversal",
                        example: "Rule -> Penalty -> Fine",
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
                                .foregroundColor(Color(hex: "#5856D6"))
                        }
                        .padding(12)
                        .background(Color(hex: "#5856D6").opacity(0.1))
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
            Text("Loading Compliance Data...")
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
    ContentView()
}
