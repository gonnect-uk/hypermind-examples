//
// ContentView.swift
// ProductFinder - Product Search & Inventory Management
//
// Uses Gonnect NanoGraphDB (Rust FFI) for RDF data
//

import SwiftUI

struct ContentView: View {
    @State private var graphDB: GraphDb?
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var stats: DatabaseStats?
    @State private var queryResults: [QueryResult] = []
    @State private var selectedQuery = "products"

    // Computed statistics
    @State private var productCount: UInt64 = 0
    @State private var alertCount: UInt64 = 0
    @State private var categoryCount: UInt64 = 0
    @State private var brandCount: UInt64 = 0

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 20) {
                    // Hero Stats
                    heroStatsCard

                    // Categories Grid
                    categoryStatsCard

                    // Query Actions
                    queryActionsCard

                    // Results
                    if !queryResults.isEmpty {
                        queryResultsCard
                    }

                    // Performance Info
                    performanceCard
                }
                .padding()
            }
            .background(Color(.systemGroupedBackground))
            .navigationTitle("Product Finder")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    HStack(spacing: 4) {
                        Image(systemName: "cylinder.fill")
                            .foregroundColor(.green)
                        Text("\(stats?.totalTriples ?? 0)")
                            .font(.caption.bold())
                    }
                }
            }
        }
        .onAppear {
            loadDatabase()
        }
    }

    // MARK: - Hero Stats Card

    private var heroStatsCard: some View {
        VStack(spacing: 16) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Inventory Dashboard")
                        .font(.headline)
                        .foregroundColor(.white)
                    Text("Real-time product analytics")
                        .font(.caption)
                        .foregroundColor(.white.opacity(0.8))
                }
                Spacer()
                Image(systemName: "bag.fill")
                    .font(.largeTitle)
                    .foregroundColor(.white.opacity(0.3))
            }

            HStack(spacing: 20) {
                StatItem(value: "\(productCount)", label: "Products", icon: "bag")
                Divider().frame(height: 40).background(Color.white.opacity(0.3))
                StatItem(value: "\(alertCount)", label: "Alerts", icon: "bell.badge")
                Divider().frame(height: 40).background(Color.white.opacity(0.3))
                StatItem(value: "\(stats?.totalEntities ?? 0)", label: "Entities", icon: "cube")
            }
        }
        .padding()
        .background(
            LinearGradient(
                colors: [Color.green, Color.green.opacity(0.7)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .cornerRadius(16)
        .shadow(color: .green.opacity(0.3), radius: 10, y: 5)
    }

    // MARK: - Category Stats Card

    private var categoryStatsCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Categories & Brands")
                .font(.headline)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                CategoryStatCard(
                    title: "Categories",
                    value: "\(categoryCount)",
                    icon: "folder.fill",
                    color: .blue
                )
                CategoryStatCard(
                    title: "Brands",
                    value: "\(brandCount)",
                    icon: "star.fill",
                    color: .purple
                )
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Query Actions Card

    private var queryActionsCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("SPARQL Queries")
                .font(.headline)

            HStack(spacing: 12) {
                QueryButton(
                    title: "Products",
                    icon: "bag",
                    color: .green,
                    isSelected: selectedQuery == "products"
                ) {
                    selectedQuery = "products"
                    executeQuery("products")
                }

                QueryButton(
                    title: "Alerts",
                    icon: "bell.badge",
                    color: .orange,
                    isSelected: selectedQuery == "alerts"
                ) {
                    selectedQuery = "alerts"
                    executeQuery("alerts")
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Query Results Card

    private var queryResultsCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Query Results")
                    .font(.headline)
                Spacer()
                Text("\(queryResults.count) items")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            ForEach(Array(queryResults.prefix(10).enumerated()), id: \.offset) { index, result in
                ProductResultRow(bindings: result.bindings, index: index, queryType: selectedQuery)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Performance Card

    private var performanceCard: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "bolt.fill")
                    .foregroundColor(.yellow)
                Text("Gonnect NanoGraphDB")
                    .font(.caption.bold())
            }

            Text("Rust-powered • 35-180x faster than RDFox • 24 bytes/triple")
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding()
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(.systemBackground))
        .cornerRadius(8)
    }

    // MARK: - Database Operations

    private func loadDatabase() {
        DispatchQueue.global(qos: .userInitiated).async {
            do {
                let db = GraphDb()

                // Load product catalog TTL into named graph
                if let url = Bundle.main.url(forResource: "product-catalog", withExtension: "ttl") {
                    let content = try String(contentsOf: url, encoding: .utf8)
                    try db.loadTtl(ttlContent: content, graphName: "http://zenya.com/products")
                }

                let dbStats = db.getStats()

                // Count by type
                let products = db.countByType(typeFilter: "Product")
                let alerts = db.countByType(typeFilter: "Alert")
                let categories = db.countByType(typeFilter: "Category")
                let brands = db.countByType(typeFilter: "Brand")

                DispatchQueue.main.async {
                    self.graphDB = db
                    self.stats = dbStats
                    self.productCount = products
                    self.alertCount = alerts
                    self.categoryCount = categories
                    self.brandCount = brands
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

        DispatchQueue.global(qos: .userInitiated).async {
            do {
                let sparql: String

                switch queryType {
                case "products":
                    sparql = """
                        SELECT ?product ?name ?price ?stock ?rating WHERE {
                            ?product <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Product> .
                            ?product <http://schema.org/name> ?name .
                            OPTIONAL { ?product <http://schema.org/price> ?price }
                            OPTIONAL { ?product <http://zenya.com/domain/products/stockQuantity> ?stock }
                            OPTIONAL { ?product <http://zenya.com/domain/products/rating> ?rating }
                        } LIMIT 20
                        """
                case "alerts":
                    sparql = """
                        SELECT ?alert ?type ?severity ?message WHERE {
                            ?alert <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/domain/products/InventoryAlert> .
                            OPTIONAL { ?alert <http://zenya.com/domain/products/alertType> ?type }
                            OPTIONAL { ?alert <http://zenya.com/domain/products/severity> ?severity }
                            OPTIONAL { ?alert <http://zenya.com/domain/products/message> ?message }
                        } LIMIT 20
                        """
                default:
                    sparql = "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
                }

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

// MARK: - Supporting Views

struct StatItem: View {
    let value: String
    let label: String
    let icon: String

    var body: some View {
        VStack(spacing: 4) {
            Image(systemName: icon)
                .font(.caption)
                .foregroundColor(.white.opacity(0.8))
            Text(value)
                .font(.title2.bold())
                .foregroundColor(.white)
            Text(label)
                .font(.caption2)
                .foregroundColor(.white.opacity(0.8))
        }
    }
}

struct CategoryStatCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color

    var body: some View {
        HStack {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(color)

            VStack(alignment: .leading) {
                Text(value)
                    .font(.title2.bold())
                Text(title)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
        }
        .padding()
        .background(color.opacity(0.1))
        .cornerRadius(8)
    }
}

struct QueryButton: View {
    let title: String
    let icon: String
    let color: Color
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack {
                Image(systemName: icon)
                Text(title)
            }
            .font(.subheadline.bold())
            .foregroundColor(isSelected ? .white : color)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 12)
            .background(isSelected ? color : color.opacity(0.1))
            .cornerRadius(8)
        }
    }
}

struct ProductResultRow: View {
    let bindings: [String: String]
    let index: Int
    let queryType: String

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if queryType == "products" {
                HStack {
                    Text(bindings["name"] ?? "Unknown Product")
                        .font(.subheadline.bold())
                    Spacer()
                    if let price = bindings["price"] {
                        Text("$\(price)")
                            .font(.subheadline.bold())
                            .foregroundColor(.green)
                    }
                }

                HStack {
                    if let stock = bindings["stock"] {
                        Label(stock, systemImage: "shippingbox")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    if let rating = bindings["rating"] {
                        Label(rating, systemImage: "star.fill")
                            .font(.caption)
                            .foregroundColor(.orange)
                    }
                }
            } else if queryType == "alerts" {
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundColor(.orange)
                    Text(bindings["type"] ?? "Alert")
                        .font(.subheadline.bold())
                    Spacer()
                    if let severity = bindings["severity"] {
                        Text(severity)
                            .font(.caption)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 2)
                            .background(severityColor(severity).opacity(0.2))
                            .foregroundColor(severityColor(severity))
                            .cornerRadius(4)
                    }
                }

                if let message = bindings["message"] {
                    Text(message)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(8)
    }

    private func severityColor(_ severity: String) -> Color {
        switch severity.lowercased() {
        case "high", "critical": return .red
        case "medium", "warning": return .orange
        default: return .yellow
        }
    }
}

#Preview {
    ContentView()
}
