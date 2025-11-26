//
// ComplianceService.swift
// ComplianceGuardian
//
// Core service using REAL SPARQL queries against rust-kgdb
//

import Foundation
// Note: gonnect types are available directly (no import needed) because
// gonnect.swift is compiled as part of this target

@MainActor
class ComplianceService: ObservableObject {
    @Published var regulations: [Regulation] = []
    @Published var requirements: [ComplianceRequirement] = []
    @Published var isLoading = false
    @Published var errorMessage: String?
    @Published var complianceScore: Double = 0.0
    @Published var violationCount: Int = 0

    private let graphDB = GraphDb() // Note: Generated from Rust (GraphDb not GraphDB)
    private let graphName = "financial_compliance"

    // MARK: - Initialization

    func initialize() async {
        isLoading = true
        defer { isLoading = false }

        do {
            // Load TTL dataset from bundle root (added via project.yml resources)
            guard let ttlURL = Bundle.main.url(forResource: "financial_compliance", withExtension: "ttl"),
                  let ttlContent = try? String(contentsOf: ttlURL) else {
                errorMessage = "Failed to load financial_compliance.ttl from app bundle"
                return
            }

            try graphDB.loadTtl(ttlContent: ttlContent, graphName: graphName)

            // Fetch regulations and requirements
            await fetchRegulations()
            await fetchRequirements()
            await updateComplianceMetrics()

        } catch {
            errorMessage = "Initialization error: \(error.localizedDescription)"
        }
    }

    // MARK: - SPARQL Queries

    /// Query critical regulations with penalties > $5M
    func fetchRegulations() async {
        let sparql = """
        PREFIX fro: <http://finregont.com/ontology#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
        PREFIX reg: <http://example.org/regulation#>
        PREFIX gdpr: <http://example.org/gdpr#>

        SELECT ?reg ?label ?jurisdiction ?body ?enactDate ?amended ?risk ?desc
               ?maxFine ?maxPrison ?currency ?percentRev ?responseTime
        WHERE {
          ?reg rdfs:label ?label ;
               fro:jurisdiction ?jurisdiction ;
               fro:regulatoryBody ?body ;
               fro:enactmentDate ?enactDate ;
               fro:riskLevel ?risk ;
               rdfs:comment ?desc .

          OPTIONAL { ?reg fro:lastAmended ?amended }
          OPTIONAL { ?reg fro:responseTime ?responseTime }

          ?reg fro:penalty ?penalty .
          ?penalty fro:maxFine ?maxFine ;
                   fro:currency ?currency .

          OPTIONAL { ?penalty fro:maxPrison ?maxPrison }
          OPTIONAL { ?penalty fro:percentageRevenue ?percentRev }

          FILTER(?maxFine > 5000000)
        }
        ORDER BY DESC(?maxFine)
        """

        do {
            let results = try graphDB.querySelect(sparql: sparql)
            var fetchedRegulations: [Regulation] = []

            for result in results {
                guard let id = result.bindings["reg"],
                      let label = result.bindings["label"],
                      let jurisdiction = result.bindings["jurisdiction"],
                      let riskStr = result.bindings["risk"],
                      let description = result.bindings["desc"],
                      let fineStr = result.bindings["maxFine"],
                      let currency = result.bindings["currency"] else {
                    continue
                }

                let riskLevel = RiskLevel(rawValue: cleanLiteral(riskStr)) ?? .medium
                let maxFine = Int(cleanLiteral(fineStr)) ?? 0
                let maxPrison = result.bindings["maxPrison"].flatMap { Int(cleanLiteral($0)) }
                let percentRev = result.bindings["percentRev"].flatMap { Int(cleanLiteral($0)) }

                // Parse response time (ISO 8601 duration like PT72H)
                let responseTime = result.bindings["responseTime"]
                    .flatMap { parseISO8601Duration(cleanLiteral($0)) }

                let regulation = Regulation(
                    id: cleanURI(id),
                    label: cleanLiteral(label),
                    jurisdiction: cleanLiteral(jurisdiction),
                    regulatoryBody: extractBodyName(result.bindings["body"] ?? ""),
                    enactmentDate: parseDate(cleanLiteral(result.bindings["enactDate"] ?? "")) ?? Date(),
                    lastAmended: result.bindings["amended"].flatMap { parseDate(cleanLiteral($0)) },
                    riskLevel: riskLevel,
                    description: cleanLiteral(description),
                    maxFine: maxFine,
                    maxPrison: maxPrison,
                    currency: cleanLiteral(currency),
                    percentageRevenue: percentRev,
                    responseTime: responseTime,
                    requirements: [],
                    additionalSanctions: nil
                )

                fetchedRegulations.append(regulation)
            }

            regulations = fetchedRegulations
        } catch {
            errorMessage = "Failed to fetch regulations: \(error.localizedDescription)"
        }
    }

    /// Query compliance requirements
    func fetchRequirements() async {
        let sparql = """
        PREFIX comp: <http://example.org/compliance#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

        SELECT ?req ?label ?desc ?freq ?deadline ?auto ?cost ?threshold ?critical
        WHERE {
          ?req a comp:ComplianceRequirement ;
               rdfs:label ?label ;
               comp:description ?desc .

          OPTIONAL { ?req comp:monitoringFrequency ?freq }
          OPTIONAL { ?req comp:deadline ?deadline }
          OPTIONAL { ?req comp:automatable ?auto }
          OPTIONAL { ?req comp:implementationCost ?cost }
          OPTIONAL { ?req comp:threshold ?threshold }
          OPTIONAL { ?req comp:critical ?critical }
        }
        """

        do {
            let results = try graphDB.querySelect(sparql: sparql)
            var fetchedRequirements: [ComplianceRequirement] = []

            for result in results {
                guard let id = result.bindings["req"],
                      let label = result.bindings["label"],
                      let description = result.bindings["desc"] else {
                    continue
                }

                let deadline = result.bindings["deadline"]
                    .flatMap { parseISO8601Duration(cleanLiteral($0)) }

                let automatable = result.bindings["auto"]
                    .map { cleanLiteral($0).lowercased() == "true" } ?? false

                let critical = result.bindings["critical"]
                    .map { cleanLiteral($0).lowercased() == "true" } ?? false

                let threshold = result.bindings["threshold"]
                    .flatMap { Double(cleanLiteral($0)) }

                let requirement = ComplianceRequirement(
                    id: cleanURI(id),
                    label: cleanLiteral(label),
                    description: cleanLiteral(description),
                    monitoringFrequency: result.bindings["freq"].map { cleanLiteral($0) },
                    deadline: deadline,
                    automatable: automatable,
                    implementationCost: result.bindings["cost"].map { cleanLiteral($0) },
                    threshold: threshold,
                    critical: critical
                )

                fetchedRequirements.append(requirement)
            }

            requirements = fetchedRequirements
        } catch {
            errorMessage = "Failed to fetch requirements: \(error.localizedDescription)"
        }
    }

    /// Query for flagged transactions (violations)
    func fetchViolations() async -> [String] {
        let sparql = """
        PREFIX comp: <http://example.org/compliance#>
        PREFIX fibo: <https://spec.edmcouncil.org/fibo/ontology/>

        SELECT ?txn ?reason ?rule
        WHERE {
          ?txn a fibo:Trade ;
               comp:flagged "true"^^<http://www.w3.org/2001/XMLSchema#boolean> ;
               comp:flagReason ?reason ;
               comp:violatesRule ?rule .
        }
        """

        do {
            let results = try graphDB.querySelect(sparql: sparql)
            return results.compactMap { $0.bindings["txn"] }
        } catch {
            return []
        }
    }

    /// Update overall compliance metrics
    func updateComplianceMetrics() async {
        let violations = await fetchViolations()
        violationCount = violations.count

        // Calculate compliance score (0-100)
        let totalRequirements = max(requirements.count, 1)
        let criticalCount = requirements.filter { $0.critical }.count
        let violationPenalty = min(violations.count * 10, 30) // Max 30% penalty

        complianceScore = max(0, 100 - Double(violationPenalty) - Double(criticalCount) * 5)
    }

    // MARK: - Scenario Testing

    /// Execute "what-if" scenario by inserting test data
    func runScenario(scenarioName: String, testData: String) async -> String {
        let updateSparql = """
        PREFIX fibo: <https://spec.edmcouncil.org/fibo/ontology/>
        PREFIX comp: <http://example.org/compliance#>

        INSERT DATA {
          \(testData)
        }
        """

        do {
            // TODO: SPARQL UPDATE not yet implemented in FFI
            // try graphDB.update(sparql: updateSparql)
            await updateComplianceMetrics()
            return "Scenario '\(scenarioName)' simulated (UPDATE pending FFI support). Current violation count: \(violationCount)"
        } catch {
            return "Scenario failed: \(error.localizedDescription)"
        }
    }

    // MARK: - Utility Functions

    private func cleanURI(_ uri: String) -> String {
        uri.trimmingCharacters(in: CharacterSet(charactersIn: "<>"))
    }

    private func cleanLiteral(_ literal: String) -> String {
        // Remove quotes and datatype annotations
        var cleaned = literal
        if cleaned.hasPrefix("\"") && cleaned.contains("\"^^") {
            if let endQuote = cleaned.lastIndex(of: "\"") {
                cleaned = String(cleaned[cleaned.index(after: cleaned.startIndex)..<endQuote])
            }
        } else if cleaned.hasPrefix("\"") {
            cleaned = cleaned.trimmingCharacters(in: CharacterSet(charactersIn: "\""))
        }
        return cleaned
    }

    private func extractBodyName(_ bodyURI: String) -> String {
        let uri = cleanURI(bodyURI)
        return uri.components(separatedBy: "#").last ?? uri.components(separatedBy: "/").last ?? uri
    }

    private func parseDate(_ dateStr: String) -> Date? {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withFullDate]
        return formatter.date(from: dateStr)
    }

    /// Parse ISO 8601 duration (PT72H, P30D, etc.) to seconds
    private func parseISO8601Duration(_ duration: String) -> TimeInterval? {
        var seconds: TimeInterval = 0

        if duration.hasPrefix("PT") {
            let timeStr = String(duration.dropFirst(2))
            if let hIndex = timeStr.firstIndex(of: "H") {
                let hours = Double(timeStr[..<hIndex]) ?? 0
                seconds += hours * 3600
            }
            if let mIndex = timeStr.firstIndex(of: "M") {
                let minutes = Double(timeStr[..<mIndex]) ?? 0
                seconds += minutes * 60
            }
        } else if duration.hasPrefix("P") {
            let dayStr = String(duration.dropFirst())
            if let dIndex = dayStr.firstIndex(of: "D") {
                let days = Double(dayStr[..<dIndex]) ?? 0
                seconds += days * 86400
            }
        }

        return seconds > 0 ? seconds : nil
    }
}
