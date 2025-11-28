//
// ContentView.swift
// GraphDBAdmin - Generic RDF Triple Store Administration
//
// Uses Gonnect NanoGraphDB (Rust FFI)
// Super generic - NO HARDCODING
//

import SwiftUI

// MARK: - Reasoner Types (W3C Standards)

enum ReasonerType: String, CaseIterable, Identifiable {
    case rdfs = "RDFS"
    case owlRL = "OWL 2 RL"
    case owlEL = "OWL 2 EL"
    case owlQL = "OWL 2 QL"
    case shacl = "SHACL"
    case datalog = "Datalog"

    var id: String { rawValue }

    var description: String {
        switch self {
        case .rdfs: return "RDF Schema entailment rules"
        case .owlRL: return "OWL 2 Rule Language profile"
        case .owlEL: return "OWL 2 Existential Language profile"
        case .owlQL: return "OWL 2 Query Language profile"
        case .shacl: return "Shapes Constraint Language validation"
        case .datalog: return "Datalog recursive rules"
        }
    }

    var icon: String {
        switch self {
        case .rdfs: return "arrow.triangle.branch"
        case .owlRL: return "brain"
        case .owlEL: return "function"
        case .owlQL: return "magnifyingglass.circle"
        case .shacl: return "checkmark.shield"
        case .datalog: return "repeat"
        }
    }

    var color: Color {
        switch self {
        case .rdfs: return .blue
        case .owlRL: return .purple
        case .owlEL: return .orange
        case .owlQL: return .green
        case .shacl: return .red
        case .datalog: return .indigo
        }
    }

    // Description of what this query template returns
    var templateDescription: String {
        switch self {
        case .rdfs:
            return "Classes & Labels: Entities with rdf:type and rdfs:label"
        case .owlRL:
            return "All Triples: Complete subject-predicate-object"
        case .owlEL:
            return "Predicates Only: List of all unique predicates"
        case .owlQL:
            return "Type Statistics: Count entities by rdf:type"
        case .shacl:
            return "Label Search: Entities with rdfs:label values"
        case .datalog:
            return "Type Hierarchy: Subjects with their types"
        }
    }

    // Generate SPARQL template for this query type
    // NOTE: These are query templates, not actual reasoners - they demonstrate different SPARQL patterns
    var sparqlTemplate: String {
        switch self {
        case .rdfs:
            // Find entities with both type and label (class instances)
            return "SELECT ?entity ?type ?label WHERE { ?entity <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type . ?entity <http://www.w3.org/2000/01/rdf-schema#label> ?label } LIMIT 50"
        case .owlRL:
            // Return all triples (most general query)
            return "SELECT ?subject ?predicate ?object WHERE { ?subject ?predicate ?object } LIMIT 50"
        case .owlEL:
            // Return only unique predicates
            return "SELECT DISTINCT ?predicate WHERE { ?s ?predicate ?o } LIMIT 50"
        case .owlQL:
            // Aggregate query: count by type
            return "SELECT ?type (COUNT(?entity) AS ?count) WHERE { ?entity <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type } GROUP BY ?type LIMIT 20"
        case .shacl:
            // Find labeled entities
            return "SELECT ?entity ?label WHERE { ?entity <http://www.w3.org/2000/01/rdf-schema#label> ?label } LIMIT 50"
        case .datalog:
            // Find typed subjects (distinct)
            return "SELECT DISTINCT ?subject ?type WHERE { ?subject <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type } LIMIT 50"
        }
    }
}

// MARK: - Main View

struct ContentView: View {
    @State private var graphDB: GraphDb?
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var stats: DatabaseStats?
    @State private var queryResults: [QueryResult] = []
    @State private var selectedReasoner: ReasonerType = .rdfs
    @State private var customSparql = ""
    @State private var showReasonerPicker = false
    @State private var selectedTab = 0
    @State private var allTriples: [TripleResult] = []
    @State private var allGraphs: [String] = []
    @State private var allPredicates: [String] = []

    // Pagination state
    @State private var tripleCurrentPage = 0
    @State private var graphCurrentPage = 0
    @State private var predicateCurrentPage = 0
    private let pageSize = 20

    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Tab picker
                Picker("View", selection: $selectedTab) {
                    Text("Overview").tag(0)
                    Text("SPARQL").tag(1)
                    Text("Browser").tag(2)
                }
                .pickerStyle(.segmented)
                .padding()

                ScrollView {
                    VStack(spacing: 20) {
                        switch selectedTab {
                        case 0:
                            // Overview tab
                            databaseHealthCard
                            statsGridCard
                            graphsListCard
                            reasonerSelectionCard
                            performanceCard
                        case 1:
                            // SPARQL Query tab
                            customSparqlCard
                            if !queryResults.isEmpty {
                                queryResultsCard
                            }
                        case 2:
                            // Browser tab
                            tripleBrowserCard
                            // Show ALL named graphs in the database
                            graphBrowserCard
                            predicateBrowserCard
                        default:
                            EmptyView()
                        }
                    }
                    .padding()
                }
            }
            .background(Color(.systemGroupedBackground))
            .navigationTitle("GraphDB Admin")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    HStack(spacing: 4) {
                        Circle()
                            .fill(graphDB != nil ? Color.green : Color.red)
                            .frame(width: 8, height: 8)
                        Text(graphDB != nil ? "Connected" : "Offline")
                            .font(.caption)
                    }
                }
            }
        }
        .onAppear {
            loadDatabase()
        }
    }

    // MARK: - Database Health Card

    private var databaseHealthCard: some View {
        VStack(spacing: 16) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Database Health")
                        .font(.headline)
                        .foregroundColor(.white)
                    Text("Gonnect NanoGraphDB")
                        .font(.caption)
                        .foregroundColor(.white.opacity(0.8))
                }
                Spacer()
                Image(systemName: "server.rack")
                    .font(.largeTitle)
                    .foregroundColor(.white.opacity(0.3))
            }

            HStack(spacing: 20) {
                HealthMetric(label: "Status", value: graphDB != nil ? "Online" : "Offline", icon: "power")
                Divider().frame(height: 40).background(Color.white.opacity(0.3))
                HealthMetric(label: "Version", value: getVersion(), icon: "tag")
                Divider().frame(height: 40).background(Color.white.opacity(0.3))
                HealthMetric(label: "Backend", value: stats?.storageBackend ?? "Memory", icon: "internaldrive")
            }
        }
        .padding()
        .background(
            LinearGradient(
                colors: [Color.purple, Color.purple.opacity(0.7)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .cornerRadius(16)
        .shadow(color: .purple.opacity(0.3), radius: 10, y: 5)
    }

    // MARK: - Stats Grid Card

    private var statsGridCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Database Statistics")
                .font(.headline)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                StatCard(
                    title: "Total Triples",
                    value: "\(stats?.totalTriples ?? 0)",
                    icon: "cylinder.fill",
                    color: .blue
                )
                StatCard(
                    title: "Entities",
                    value: "\(stats?.totalEntities ?? 0)",
                    icon: "cube.fill",
                    color: .green
                )
                StatCard(
                    title: "Named Graphs",
                    value: "\(allGraphs.count)",
                    icon: "chart.bar.doc.horizontal.fill",
                    color: .orange
                )
                StatCard(
                    title: "Dictionary",
                    value: "\(stats?.dictionarySize ?? 0)",
                    icon: "text.book.closed.fill",
                    color: .purple
                )
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Graphs List Card

    private var graphsListCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "folder.badge.gearshape")
                    .foregroundColor(.orange)
                Text("Named Graphs")
                    .font(.headline)
                Spacer()
                Text("\(allGraphs.count) total")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            if allGraphs.isEmpty {
                HStack {
                    Image(systemName: "questionmark.folder")
                        .foregroundColor(.secondary)
                    Text("No named graphs found")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding(.vertical, 8)
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(allGraphs.prefix(5), id: \.self) { graph in
                        HStack {
                            Image(systemName: "folder.fill")
                                .font(.caption)
                                .foregroundColor(.orange)
                            Text(formatURI(graph))
                                .font(.caption)
                                .lineLimit(1)
                            Spacer()
                            // Show metadata: which TTL file
                            if graph.contains("catalog/database") {
                                Text("database-catalog.ttl")
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                            }
                        }
                        .padding(.vertical, 4)
                    }
                    if allGraphs.count > 5 {
                        Text("+ \(allGraphs.count - 5) more in Browser tab")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .padding(.top, 4)
                    }
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Reasoner Selection Card

    private var reasonerSelectionCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Query Templates")
                    .font(.headline)
                Spacer()
                Text("SPARQL Patterns")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 8) {
                ForEach(ReasonerType.allCases) { reasoner in
                    ReasonerButton(
                        reasoner: reasoner,
                        isSelected: selectedReasoner == reasoner
                    ) {
                        selectedReasoner = reasoner
                        customSparql = reasoner.sparqlTemplate
                        // Auto-execute query and switch to SPARQL tab
                        selectedTab = 1
                        executeQuery()
                    }
                }
            }

            // Show description of selected template
            Text(selectedReasoner.templateDescription)
                .font(.caption)
                .foregroundColor(.secondary)
                .padding(.top, 4)
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Custom SPARQL Query Card

    private var customSparqlCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("SPARQL Query")
                    .font(.headline)
                Spacer()
                // Show selected reasoner template
                HStack(spacing: 4) {
                    Image(systemName: selectedReasoner.icon)
                        .foregroundColor(selectedReasoner.color)
                        .font(.caption)
                    Text(selectedReasoner.rawValue)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(selectedReasoner.color.opacity(0.1))
                .cornerRadius(6)

                Button("Clear") {
                    customSparql = ""
                }
                .font(.caption)
                Button("Execute") {
                    executeQuery()
                }
                .buttonStyle(.borderedProminent)
            }

            // Editable text area for custom SPARQL
            TextEditor(text: $customSparql)
                .font(.system(.caption, design: .monospaced))
                .frame(minHeight: 200)
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .stroke(Color(.systemGray4), lineWidth: 1)
                )

            // Display error message if query failed
            if let errorMessage = errorMessage {
                HStack(spacing: 8) {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundColor(.red)
                    Text(errorMessage)
                        .font(.caption)
                        .foregroundColor(.red)
                    Spacer()
                    Button("Dismiss") {
                        self.errorMessage = nil
                    }
                    .font(.caption)
                }
                .padding(8)
                .background(Color.red.opacity(0.1))
                .cornerRadius(6)
            }

            // Quick templates - auto-execute on click
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(["SELECT ALL", "COUNT", "TYPES", "PREDICATES"], id: \.self) { template in
                        Button(template) {
                            let query = getQueryTemplate(template)
                            customSparql = query
                            executeQuery(query)
                        }
                        .font(.caption)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(Color(.systemGray5))
                        .cornerRadius(6)
                    }
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Triple Browser Card

    private var tripleBrowserCard: some View {
        let totalPages = (allTriples.count + pageSize - 1) / pageSize
        let startIndex = tripleCurrentPage * pageSize
        let endIndex = min(startIndex + pageSize, allTriples.count)
        let currentTriples = allTriples.isEmpty ? [] : Array(allTriples[startIndex..<endIndex])

        return VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "tablecells")
                    .foregroundColor(.blue)
                Text("Triple Browser")
                    .font(.headline)
                Spacer()
                Text("\(allTriples.count) total")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Button("Load") {
                    tripleCurrentPage = 0
                    loadTriples()
                }
                .font(.caption)
            }

            if allTriples.isEmpty {
                Text("Tap Load to fetch triples")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()
            } else {
                ForEach(Array(currentTriples.enumerated()), id: \.offset) { index, triple in
                    TripleRow(triple: triple)
                }

                // Pagination controls
                if totalPages > 1 {
                    HStack {
                        Button(action: { tripleCurrentPage = max(0, tripleCurrentPage - 1) }) {
                            Image(systemName: "chevron.left")
                        }
                        .disabled(tripleCurrentPage == 0)

                        Spacer()
                        Text("Page \(tripleCurrentPage + 1) of \(totalPages)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Spacer()

                        Button(action: { tripleCurrentPage = min(totalPages - 1, tripleCurrentPage + 1) }) {
                            Image(systemName: "chevron.right")
                        }
                        .disabled(tripleCurrentPage >= totalPages - 1)
                    }
                    .padding(.top, 8)
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Graph Browser Card

    private var graphBrowserCard: some View {
        // Add default graph to display list
        let displayGraphs = ["(default graph)"] + allGraphs
        let totalPages = (displayGraphs.count + pageSize - 1) / pageSize
        let startIndex = graphCurrentPage * pageSize
        let endIndex = min(startIndex + pageSize, displayGraphs.count)
        let currentGraphs = Array(displayGraphs[startIndex..<endIndex])

        return VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "chart.bar.doc.horizontal")
                    .foregroundColor(.orange)
                Text("Graphs")
                    .font(.headline)
                Spacer()
                Text("\(displayGraphs.count) total")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Button("Load") {
                    graphCurrentPage = 0
                    loadGraphs()
                }
                .font(.caption)
            }

            ForEach(currentGraphs, id: \.self) { graph in
                HStack {
                    if graph == "(default graph)" {
                        Image(systemName: "house")
                            .foregroundColor(.blue)
                        Text("Default Graph")
                            .font(.caption)
                            .fontWeight(.medium)
                    } else {
                        Image(systemName: "folder")
                            .foregroundColor(.orange)
                        Text(formatURI(graph))
                            .font(.caption)
                            .lineLimit(1)
                    }
                    Spacer()
                }
                .padding(.vertical, 4)
            }

            // Pagination controls
            if totalPages > 1 {
                HStack {
                    Button(action: { graphCurrentPage = max(0, graphCurrentPage - 1) }) {
                        Image(systemName: "chevron.left")
                    }
                    .disabled(graphCurrentPage == 0)

                    Spacer()
                    Text("Page \(graphCurrentPage + 1) of \(totalPages)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()

                    Button(action: { graphCurrentPage = min(totalPages - 1, graphCurrentPage + 1) }) {
                        Image(systemName: "chevron.right")
                    }
                    .disabled(graphCurrentPage >= totalPages - 1)
                }
                .padding(.top, 8)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 5, y: 2)
    }

    // MARK: - Predicate Browser Card

    private var predicateBrowserCard: some View {
        let totalPages = (allPredicates.count + pageSize - 1) / pageSize
        let startIndex = predicateCurrentPage * pageSize
        let endIndex = min(startIndex + pageSize, allPredicates.count)
        let currentPredicates = allPredicates.isEmpty ? [] : Array(allPredicates[startIndex..<endIndex])

        return VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "arrow.right")
                    .foregroundColor(.green)
                Text("Predicates")
                    .font(.headline)
                Spacer()
                Text("\(allPredicates.count) total")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Button("Load") {
                    predicateCurrentPage = 0
                    loadPredicates()
                }
                .font(.caption)
            }

            if allPredicates.isEmpty {
                Text("Tap Load to fetch predicates")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()
            } else {
                ForEach(currentPredicates, id: \.self) { predicate in
                    HStack {
                        Image(systemName: "arrow.right.circle")
                            .foregroundColor(.green)
                        Text(formatURI(predicate))
                            .font(.caption)
                            .lineLimit(1)
                        Spacer()
                    }
                    .padding(.vertical, 4)
                }

                // Pagination controls
                if totalPages > 1 {
                    HStack {
                        Button(action: { predicateCurrentPage = max(0, predicateCurrentPage - 1) }) {
                            Image(systemName: "chevron.left")
                        }
                        .disabled(predicateCurrentPage == 0)

                        Spacer()
                        Text("Page \(predicateCurrentPage + 1) of \(totalPages)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Spacer()

                        Button(action: { predicateCurrentPage = min(totalPages - 1, predicateCurrentPage + 1) }) {
                            Image(systemName: "chevron.right")
                        }
                        .disabled(predicateCurrentPage >= totalPages - 1)
                    }
                    .padding(.top, 8)
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
                Text("\(queryResults.count) bindings")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            ForEach(Array(queryResults.prefix(10).enumerated()), id: \.offset) { index, result in
                ResultRow(bindings: result.bindings, index: index)
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
                Text("Rust-Powered Performance")
                    .font(.caption.bold())
            }

            HStack(spacing: 16) {
                PerformanceMetric(label: "Lookup", value: "882 ns")
                PerformanceMetric(label: "Insert", value: "391K/s")
                PerformanceMetric(label: "Memory", value: "24 B/triple")
            }
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
                // Create GraphDB instance with admin-specific graph URI
                let db = GraphDb(appGraphUri: "http://zenya.com/admin")

                // Load GraphDBAdmin catalog into default graph (http://zenya.com/admin)
                // Per-app architecture: 1 TTL = 1 named graph (graph name = app name unless specified)
                // TTL files are in bundle root (not in subdirectory)
                if let url = Bundle.main.url(forResource: "database-catalog", withExtension: "ttl") {
                    let content = try String(contentsOf: url, encoding: .utf8)
                    // Load into default graph (graphName: nil means use app's default graph URI)
                    try db.loadTtl(ttlContent: content, graphName: nil)
                }

                let dbStats = db.getStats()

                DispatchQueue.main.async {
                    self.graphDB = db
                    self.stats = dbStats
                    self.customSparql = selectedReasoner.sparqlTemplate
                    self.isLoading = false
                    // Load all named graphs dynamically
                    self.loadGraphs()
                }
            } catch {
                DispatchQueue.main.async {
                    self.errorMessage = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }

    private func executeQuery(_ overrideQuery: String? = nil) {
        guard let db = graphDB else { return }

        let sparql = overrideQuery ?? (customSparql.isEmpty ? selectedReasoner.sparqlTemplate : customSparql)

        DispatchQueue.global(qos: .userInitiated).async {
            do {
                // Convert template to executable query
                // Remove comments and execute
                let executableSparql = sparql
                    .components(separatedBy: "\n")
                    .filter { !$0.trimmingCharacters(in: .whitespaces).hasPrefix("#") }
                    .joined(separator: "\n")

                let results = try db.querySelect(sparql: executableSparql)

                DispatchQueue.main.async {
                    self.queryResults = results
                    self.errorMessage = nil // Clear error on success
                }
            } catch {
                DispatchQueue.main.async {
                    self.errorMessage = error.localizedDescription
                }
            }
        }
    }

    private func loadTriples() {
        guard let db = graphDB else { return }
        DispatchQueue.global(qos: .userInitiated).async {
            let triples = db.getAllTriples(limit: 100)
            DispatchQueue.main.async {
                self.allTriples = triples
            }
        }
    }

    private func loadGraphs() {
        guard let db = graphDB else { return }
        DispatchQueue.global(qos: .userInitiated).async {
            // List ALL application graphs in the system (GraphDBAdmin-specific)
            // This shows all known app graphs: RiskAnalyzer, ProductFinder, ComplianceChecker, Admin
            let graphs = db.listAllAppGraphs()
            DispatchQueue.main.async {
                self.allGraphs = graphs
            }
        }
    }

    private func loadPredicates() {
        guard let db = graphDB else { return }
        DispatchQueue.global(qos: .userInitiated).async {
            let predicates = db.getAllPredicates(limit: 50)
            DispatchQueue.main.async {
                self.allPredicates = predicates
            }
        }
    }

    private func getQueryTemplate(_ name: String) -> String {
        switch name {
        case "SELECT ALL":
            return "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 100"
        case "COUNT":
            return "SELECT (COUNT(*) AS ?count) WHERE { ?s ?p ?o }"
        case "TYPES":
            return "SELECT DISTINCT ?type WHERE { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type }"
        case "PREDICATES":
            return "SELECT DISTINCT ?p WHERE { ?s ?p ?o }"
        default:
            return ""
        }
    }

    private func formatURI(_ uri: String) -> String {
        // Extract local name from URI
        if let lastSlash = uri.lastIndex(of: "/") {
            return String(uri[uri.index(after: lastSlash)...])
        }
        if let lastHash = uri.lastIndex(of: "#") {
            return String(uri[uri.index(after: lastHash)...])
        }
        return uri
    }
}

// MARK: - Supporting Views

struct HealthMetric: View {
    let label: String
    let value: String
    let icon: String

    var body: some View {
        VStack(spacing: 4) {
            Image(systemName: icon)
                .font(.caption)
                .foregroundColor(.white.opacity(0.8))
            Text(value)
                .font(.subheadline.bold())
                .foregroundColor(.white)
            Text(label)
                .font(.caption2)
                .foregroundColor(.white.opacity(0.8))
        }
    }
}

struct StatCard: View {
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
                    .font(.title3.bold())
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

struct ReasonerButton: View {
    let reasoner: ReasonerType
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 4) {
                Image(systemName: reasoner.icon)
                    .font(.title3)
                Text(reasoner.rawValue)
                    .font(.caption2)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 8)
            .foregroundColor(isSelected ? .white : reasoner.color)
            .background(isSelected ? reasoner.color : reasoner.color.opacity(0.1))
            .cornerRadius(8)
        }
    }
}

struct PerformanceMetric: View {
    let label: String
    let value: String

    var body: some View {
        VStack(spacing: 2) {
            Text(value)
                .font(.caption.bold())
            Text(label)
                .font(.caption2)
                .foregroundColor(.secondary)
        }
    }
}

struct ResultRow: View {
    let bindings: [String: String]
    let index: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            ForEach(Array(bindings.keys.sorted()), id: \.self) { key in
                HStack {
                    Text("?\(key)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                    Text(bindings[key] ?? "")
                        .font(.caption)
                        .lineLimit(1)
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(8)
    }
}

struct TripleRow: View {
    let triple: TripleResult

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("S:")
                    .font(.caption2.bold())
                    .foregroundColor(.blue)
                Text(formatURI(triple.subject))
                    .font(.caption)
                    .lineLimit(1)
            }
            HStack {
                Text("P:")
                    .font(.caption2.bold())
                    .foregroundColor(.green)
                Text(formatURI(triple.predicate))
                    .font(.caption)
                    .lineLimit(1)
            }
            HStack {
                Text("O:")
                    .font(.caption2.bold())
                    .foregroundColor(.orange)
                Text(formatURI(triple.object))
                    .font(.caption)
                    .lineLimit(1)
            }
        }
        .padding(8)
        .background(Color(.systemGray6))
        .cornerRadius(6)
    }

    private func formatURI(_ uri: String) -> String {
        if let lastSlash = uri.lastIndex(of: "/") {
            return String(uri[uri.index(after: lastSlash)...])
        }
        if let lastHash = uri.lastIndex(of: "#") {
            return String(uri[uri.index(after: lastHash)...])
        }
        return uri
    }
}

#Preview {
    ContentView()
}
