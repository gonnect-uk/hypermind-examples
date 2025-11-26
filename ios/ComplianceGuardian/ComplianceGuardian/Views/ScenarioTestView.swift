//
// ScenarioTestView.swift
// ComplianceGuardian
//
// "What-if" scenario testing for compliance analysis
//

import SwiftUI

struct ScenarioTestView: View {
    @EnvironmentObject var complianceService: ComplianceService
    @EnvironmentObject var alertService: AlertService

    @State private var selectedScenario: Scenario?
    @State private var isRunning = false
    @State private var results: String?

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Header Card
                    ScenarioHeaderCard()

                    // Predefined Scenarios
                    ScenarioGrid(selectedScenario: $selectedScenario)

                    // Run Controls
                    if let scenario = selectedScenario {
                        ScenarioControlsCard(
                            scenario: scenario,
                            isRunning: $isRunning,
                            results: $results
                        )
                    }

                    // Results Display
                    if let results = results {
                        ScenarioResultsCard(results: results)
                    }
                }
                .padding()
            }
            .navigationTitle("Scenario Testing")
            .background(Color(.systemGroupedBackground))
        }
    }
}

// MARK: - Models

struct Scenario: Identifiable, Hashable {
    let id: String
    let name: String
    let description: String
    let regulation: String
    let icon: String
    let color: Color
    let riskLevel: RiskLevel
    let testData: String

    var sparqlUpdate: String {
        """
        PREFIX fibo: <https://spec.edmcouncil.org/fibo/ontology/>
        PREFIX comp: <http://example.org/compliance#>
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

        INSERT DATA {
          \(testData)
        }
        """
    }
}

// MARK: - Header Card

struct ScenarioHeaderCard: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "testtube.2")
                    .font(.title2)
                    .foregroundColor(.purple)

                Text("Scenario Testing")
                    .font(.headline)
            }

            Text("Test 'what-if' compliance scenarios to assess risk and validate monitoring systems. All scenarios run against the live knowledge graph using SPARQL updates.")
                .font(.subheadline)
                .foregroundColor(.secondary)

            HStack(spacing: 12) {
                InfoChip(icon: "checkmark.shield", text: "Safe Testing", color: .green)
                InfoChip(icon: "arrow.uturn.backward", text: "Reversible", color: .blue)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct InfoChip: View {
    let icon: String
    let text: String
    let color: Color

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: icon)
                .font(.caption)
            Text(text)
                .font(.caption)
                .fontWeight(.medium)
        }
        .foregroundColor(color)
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(color.opacity(0.1))
        .cornerRadius(8)
    }
}

// MARK: - Scenario Grid

struct ScenarioGrid: View {
    @Binding var selectedScenario: Scenario?

    let scenarios: [Scenario] = [
        Scenario(
            id: "gdpr-breach",
            name: "GDPR Data Breach",
            description: "Simulate unauthorized database access affecting 15,000 EU residents. Tests 72-hour notification deadline.",
            regulation: "GDPR Article 33",
            icon: "shield.slash.fill",
            color: .red,
            riskLevel: .critical,
            testData: """
            <http://example.org/breach/test001>
                a gdpr:DataBreach ;
                gdpr:affectedRecords "15000"^^xsd:integer ;
                gdpr:detectedAt "\(ISO8601DateFormatter().string(from: Date()))"^^xsd:dateTime ;
                comp:requiresNotification "true"^^xsd:boolean ;
                comp:deadline "PT72H"^^xsd:duration .
            """
        ),
        Scenario(
            id: "insider-trading",
            name: "Insider Trading",
            description: "Executive purchases company stock 48 hours before merger announcement. Tests SEC Rule 10b-5 monitoring.",
            regulation: "SEC Rule 10b-5",
            icon: "figure.walk.departure",
            color: .red,
            riskLevel: .critical,
            testData: """
            <http://example.org/transaction/test_txn>
                a fibo:Trade ;
                fibo:tradeDate "\(ISO8601DateFormatter().string(from: Date()))"^^xsd:dateTime ;
                fibo:security "ACME" ;
                fibo:quantity "10000"^^xsd:integer ;
                fibo:trader <http://example.org/employee/ceo> ;
                comp:flagged "true"^^xsd:boolean ;
                comp:flagReason "Executive traded before material event" ;
                comp:violatesRule <http://example.org/regulation#SEC_Rule_10b5> .
            """
        ),
        Scenario(
            id: "capital-breach",
            name: "Capital Adequacy Breach",
            description: "CET1 ratio drops to 3.8% due to loan default. Tests Basel III capital requirements (minimum 4.5%).",
            regulation: "Basel III",
            icon: "chart.line.downtrend.xyaxis",
            color: .orange,
            riskLevel: .critical,
            testData: """
            <http://example.org/bank/test_status>
                a comp:CapitalStatus ;
                comp:cet1Ratio "3.8"^^xsd:decimal ;
                comp:calculatedAt "\(ISO8601DateFormatter().string(from: Date()))"^^xsd:dateTime ;
                comp:meetsRequirement "false"^^xsd:boolean ;
                comp:violatesRule <http://example.org/regulation#Basel_III> .
            """
        ),
        Scenario(
            id: "mifid-reporting",
            name: "MiFID II Reporting Delay",
            description: "847 trades not reported within T+1 deadline. Tests MiFID II transaction reporting requirements.",
            regulation: "MiFID II",
            icon: "clock.badge.exclamationmark",
            color: .orange,
            riskLevel: .high,
            testData: """
            <http://example.org/reporting/test_batch>
                a comp:ReportingBatch ;
                comp:tradeCount "847"^^xsd:integer ;
                comp:reportingDeadline "\(ISO8601DateFormatter().string(from: Date().addingTimeInterval(-3600)))"^^xsd:dateTime ;
                comp:status "Overdue" ;
                comp:violatesRule <http://example.org/regulation#MiFID_II> .
            """
        ),
        Scenario(
            id: "data-deletion",
            name: "GDPR Deletion Request",
            description: "Customer requests data erasure under GDPR Article 17. Tests 30-day response requirement.",
            regulation: "GDPR Article 17",
            icon: "trash.circle",
            color: .blue,
            riskLevel: .high,
            testData: """
            <http://example.org/request/test_deletion>
                a gdpr:ErasureRequest ;
                gdpr:requestedAt "\(ISO8601DateFormatter().string(from: Date()))"^^xsd:dateTime ;
                gdpr:dataSubject <http://example.org/customer/test001> ;
                comp:deadline "P30D"^^xsd:duration ;
                comp:status "Pending" .
            """
        ),
        Scenario(
            id: "stress-test",
            name: "Annual Stress Test",
            description: "Simulate adverse economic scenario for Dodd-Frank stress testing. Tests capital adequacy under stress.",
            regulation: "Dodd-Frank",
            icon: "waveform.path.ecg",
            color: .purple,
            riskLevel: .high,
            testData: """
            <http://example.org/stresstest/test2025>
                a comp:StressTest ;
                comp:scenario "Severe Recession" ;
                comp:projectedLoss "125000000"^^xsd:integer ;
                comp:postStressCET1 "4.2"^^xsd:decimal ;
                comp:passesTest "false"^^xsd:boolean .
            """
        )
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Available Scenarios")
                .font(.headline)
                .padding(.horizontal)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                ForEach(scenarios) { scenario in
                    ScenarioCard(
                        scenario: scenario,
                        isSelected: selectedScenario?.id == scenario.id
                    ) {
                        withAnimation(.spring()) {
                            selectedScenario = scenario
                        }
                    }
                }
            }
        }
    }
}

struct ScenarioCard: View {
    let scenario: Scenario
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Image(systemName: scenario.icon)
                        .font(.title2)
                        .foregroundColor(scenario.color)

                    Spacer()

                    if isSelected {
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundColor(.blue)
                    }
                }

                Text(scenario.name)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundColor(.primary)
                    .lineLimit(2)
                    .fixedSize(horizontal: false, vertical: true)

                HStack(spacing: 4) {
                    Image(systemName: scenario.riskLevel.icon)
                        .font(.caption2)
                    Text(scenario.riskLevel.rawValue)
                        .font(.caption2)
                        .fontWeight(.bold)
                }
                .foregroundColor(Color(hex: scenario.riskLevel.color))
            }
            .padding()
            .frame(maxWidth: .infinity, minHeight: 140, alignment: .topLeading)
            .background(isSelected ? scenario.color.opacity(0.1) : Color(.systemBackground))
            .cornerRadius(12)
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(isSelected ? scenario.color : Color.gray.opacity(0.2), lineWidth: isSelected ? 2 : 1)
            )
            .shadow(color: .black.opacity(isSelected ? 0.1 : 0.05), radius: 8, x: 0, y: 2)
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Controls Card

struct ScenarioControlsCard: View {
    @EnvironmentObject var complianceService: ComplianceService
    let scenario: Scenario
    @Binding var isRunning: Bool
    @Binding var results: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Image(systemName: scenario.icon)
                        .foregroundColor(scenario.color)

                    Text(scenario.name)
                        .font(.headline)
                }

                Text(scenario.description)
                    .font(.subheadline)
                    .foregroundColor(.secondary)

                HStack {
                    Label(scenario.regulation, systemImage: "doc.text")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    Spacer()

                    Text(scenario.riskLevel.rawValue)
                        .font(.caption)
                        .fontWeight(.bold)
                        .foregroundColor(Color(hex: scenario.riskLevel.color))
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(Color(hex: scenario.riskLevel.color).opacity(0.15))
                        .cornerRadius(6)
                }
            }

            Divider()

            // SPARQL Preview
            VStack(alignment: .leading, spacing: 8) {
                Text("SPARQL Update")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fontWeight(.bold)

                ScrollView {
                    Text(scenario.sparqlUpdate)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundColor(.secondary)
                        .padding(8)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color(.secondarySystemGroupedBackground))
                        .cornerRadius(8)
                }
                .frame(height: 100)
            }

            Divider()

            // Run Button
            Button {
                runScenario()
            } label: {
                HStack {
                    if isRunning {
                        ProgressView()
                            .tint(.white)
                    } else {
                        Image(systemName: "play.fill")
                    }
                    Text(isRunning ? "Running Scenario..." : "Run Scenario")
                }
                .font(.headline)
                .foregroundColor(.white)
                .frame(maxWidth: .infinity)
                .padding()
                .background(isRunning ? Color.gray : scenario.color)
                .cornerRadius(12)
            }
            .disabled(isRunning)
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }

    private func runScenario() {
        isRunning = true
        results = nil

        Task {
            let result = await complianceService.runScenario(
                scenarioName: scenario.name,
                testData: scenario.testData
            )

            await MainActor.run {
                results = result
                isRunning = false
            }
        }
    }
}

// MARK: - Results Card

struct ScenarioResultsCard: View {
    let results: String

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "chart.bar.doc.horizontal")
                    .foregroundColor(.green)

                Text("Scenario Results")
                    .font(.headline)
            }

            Text(results)
                .font(.subheadline)
                .foregroundColor(.secondary)
                .padding()
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.green.opacity(0.05))
                .cornerRadius(12)

            HStack(spacing: 12) {
                InfoChip(icon: "checkmark.circle.fill", text: "Executed Successfully", color: .green)
                InfoChip(icon: "arrow.triangle.branch", text: "Knowledge Graph Updated", color: .blue)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

#Preview {
    ScenarioTestView()
        .environmentObject(ComplianceService())
        .environmentObject(AlertService())
}
