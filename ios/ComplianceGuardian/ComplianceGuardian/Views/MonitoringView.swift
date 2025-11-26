//
// MonitoringView.swift
// ComplianceGuardian
//
// Real-time transaction monitoring and breach detection
//

import SwiftUI

struct MonitoringView: View {
    @EnvironmentObject var alertService: AlertService
    @State private var selectedAlert: ComplianceAlert?
    @State private var showingBreachSimulator = false

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Real-time Status
                    MonitoringStatusCard()

                    // Active Alerts List
                    ActiveAlertsList(
                        alerts: alertService.activeAlerts,
                        selectedAlert: $selectedAlert
                    )

                    // Breach Simulator (for demo)
                    BreachSimulatorCard(showingSheet: $showingBreachSimulator)
                }
                .padding()
            }
            .navigationTitle("Monitoring")
            .background(Color(.systemGroupedBackground))
            .sheet(item: $selectedAlert) { alert in
                AlertDetailSheet(alert: alert)
                    .environmentObject(alertService)
            }
            .sheet(isPresented: $showingBreachSimulator) {
                BreachSimulatorSheet()
                    .environmentObject(alertService)
            }
        }
    }
}

// MARK: - Monitoring Status Card

struct MonitoringStatusCard: View {
    @State private var lastUpdate = Date()

    var body: some View {
        VStack(spacing: 16) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Real-time Monitoring")
                        .font(.headline)

                    HStack(spacing: 6) {
                        Circle()
                            .fill(Color.green)
                            .frame(width: 8, height: 8)
                            .animation(.easeInOut(duration: 1.0).repeatForever(), value: UUID())

                        Text("Active")
                            .font(.subheadline)
                            .foregroundColor(.green)
                            .fontWeight(.semibold)
                    }
                }

                Spacer()

                VStack(alignment: .trailing, spacing: 4) {
                    Text("Last Update")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    Text(timeAgo(from: lastUpdate))
                        .font(.caption)
                        .fontWeight(.medium)
                }
            }

            Divider()

            HStack {
                MonitoringMetric(label: "Transactions", value: "1,247", icon: "arrow.left.arrow.right")
                Spacer()
                MonitoringMetric(label: "Flagged", value: "2", icon: "flag.fill", color: .red)
                Spacer()
                MonitoringMetric(label: "Cleared", value: "1,245", icon: "checkmark.seal.fill", color: .green)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
        .onAppear {
            Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { _ in
                lastUpdate = Date()
            }
        }
    }

    private func timeAgo(from date: Date) -> String {
        let seconds = Int(Date().timeIntervalSince(date))
        if seconds < 60 {
            return "\(seconds)s ago"
        } else if seconds < 3600 {
            return "\(seconds / 60)m ago"
        } else {
            return "\(seconds / 3600)h ago"
        }
    }
}

struct MonitoringMetric: View {
    let label: String
    let value: String
    let icon: String
    var color: Color = .blue

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.title3)
                .foregroundColor(color)

            Text(value)
                .font(.title2)
                .fontWeight(.bold)

            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }
}

// MARK: - Active Alerts List

struct ActiveAlertsList: View {
    let alerts: [ComplianceAlert]
    @Binding var selectedAlert: ComplianceAlert?

    var sortedAlerts: [ComplianceAlert] {
        alerts.sorted { a, b in
            // Sort by urgency: expired > critical > high > medium > low
            if a.isExpired != b.isExpired {
                return a.isExpired
            }
            if a.severity.priority != b.severity.priority {
                return a.severity.priority < b.severity.priority
            }
            return (a.timeRemaining ?? Double.infinity) < (b.timeRemaining ?? Double.infinity)
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Active Alerts")
                    .font(.headline)
                Spacer()
                Text("\(alerts.count)")
                    .font(.subheadline)
                    .fontWeight(.bold)
                    .foregroundColor(.red)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.red.opacity(0.1))
                    .cornerRadius(8)
            }

            if alerts.isEmpty {
                EmptyAlertsView()
            } else {
                ForEach(sortedAlerts) { alert in
                    Button {
                        selectedAlert = alert
                    } label: {
                        AlertRow(alert: alert)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct AlertRow: View {
    let alert: ComplianceAlert

    var body: some View {
        HStack(spacing: 12) {
            // Severity Icon
            ZStack {
                Circle()
                    .fill(Color(hex: alert.severity.color).opacity(0.15))
                    .frame(width: 44, height: 44)

                Image(systemName: alert.severity.icon)
                    .font(.title3)
                    .foregroundColor(Color(hex: alert.severity.color))
            }

            // Alert Content
            VStack(alignment: .leading, spacing: 6) {
                Text(alert.title)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .lineLimit(1)

                Text(alert.regulationLabel)
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .lineLimit(1)

                if let timeRemaining = alert.formattedTimeRemaining {
                    HStack(spacing: 4) {
                        Image(systemName: "timer")
                            .font(.caption2)
                        Text(timeRemaining)
                            .font(.caption)
                            .fontWeight(.bold)
                        Text("remaining")
                            .font(.caption)
                    }
                    .foregroundColor(Color(hex: alert.urgencyColor))
                }
            }

            Spacer()

            Image(systemName: "chevron.right")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding()
        .background(Color(.secondarySystemGroupedBackground))
        .cornerRadius(12)
    }
}

struct EmptyAlertsView: View {
    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 48))
                .foregroundColor(.green)

            Text("No Active Alerts")
                .font(.subheadline)
                .fontWeight(.semibold)

            Text("All monitored transactions are compliant")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 32)
    }
}

// MARK: - Alert Detail Sheet

struct AlertDetailSheet: View {
    @EnvironmentObject var alertService: AlertService
    @Environment(\.dismiss) var dismiss
    let alert: ComplianceAlert

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    // Header
                    VStack(alignment: .leading, spacing: 12) {
                        HStack {
                            Image(systemName: alert.severity.icon)
                                .font(.largeTitle)
                                .foregroundColor(Color(hex: alert.severity.color))

                            Spacer()

                            Text(alert.severity.rawValue)
                                .font(.subheadline)
                                .fontWeight(.bold)
                                .foregroundColor(Color(hex: alert.severity.color))
                                .padding(.horizontal, 12)
                                .padding(.vertical, 6)
                                .background(Color(hex: alert.severity.color).opacity(0.15))
                                .cornerRadius(8)
                        }

                        Text(alert.title)
                            .font(.title)
                            .fontWeight(.bold)
                    }

                    // Countdown Timer
                    if let timeRemaining = alert.formattedTimeRemaining {
                        VStack(spacing: 8) {
                            Text("TIME REMAINING")
                                .font(.caption)
                                .foregroundColor(.secondary)
                                .fontWeight(.bold)

                            Text(timeRemaining)
                                .font(.system(size: 48, weight: .bold, design: .rounded))
                                .foregroundColor(Color(hex: alert.urgencyColor))
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(Color(hex: alert.urgencyColor).opacity(0.1))
                        .cornerRadius(16)
                    }

                    // Details
                    DetailSection(title: "Regulation", content: alert.regulationLabel)
                    DetailSection(title: "Description", content: alert.message)
                    DetailSection(title: "Required Action", content: alert.actionRequired)

                    if let transactionId = alert.transactionId {
                        DetailSection(title: "Transaction ID", content: transactionId)
                    }

                    // Actions
                    VStack(spacing: 12) {
                        Button {
                            alertService.acknowledgeAlert(alert)
                            dismiss()
                        } label: {
                            HStack {
                                Image(systemName: "checkmark.circle.fill")
                                Text("Acknowledge & Take Action")
                            }
                            .font(.headline)
                            .foregroundColor(.white)
                            .frame(maxWidth: .infinity)
                            .padding()
                            .background(Color.blue)
                            .cornerRadius(12)
                        }

                        Button {
                            alertService.dismissAlert(alert)
                            dismiss()
                        } label: {
                            HStack {
                                Image(systemName: "xmark.circle")
                                Text("Dismiss Alert")
                            }
                            .font(.headline)
                            .foregroundColor(.red)
                            .frame(maxWidth: .infinity)
                            .padding()
                            .background(Color.red.opacity(0.1))
                            .cornerRadius(12)
                        }
                    }
                }
                .padding()
            }
            .navigationTitle("Alert Details")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Close") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct DetailSection: View {
    let title: String
    let content: String

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.caption)
                .foregroundColor(.secondary)
                .fontWeight(.bold)
                .textCase(.uppercase)

            Text(content)
                .font(.subheadline)
        }
        .padding()
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(.secondarySystemGroupedBackground))
        .cornerRadius(12)
    }
}

// MARK: - Breach Simulator

struct BreachSimulatorCard: View {
    @Binding var showingSheet: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "testtube.2")
                    .foregroundColor(.purple)
                Text("Test Scenarios")
                    .font(.headline)
                Spacer()
            }

            Text("Simulate compliance breaches to test monitoring and alerting systems")
                .font(.subheadline)
                .foregroundColor(.secondary)

            Button {
                showingSheet = true
            } label: {
                HStack {
                    Image(systemName: "play.circle.fill")
                    Text("Run Breach Simulation")
                }
                .font(.subheadline)
                .fontWeight(.semibold)
                .foregroundColor(.white)
                .frame(maxWidth: .infinity)
                .padding()
                .background(Color.purple)
                .cornerRadius(12)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct BreachSimulatorSheet: View {
    @EnvironmentObject var alertService: AlertService
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationStack {
            List {
                Section("Data Privacy Breaches") {
                    SimulationButton(
                        title: "GDPR Data Breach",
                        description: "Simulate unauthorized database access requiring 72-hour notification",
                        icon: "shield.slash.fill",
                        color: .red
                    ) {
                        alertService.simulateBreachDetection(breachType: "gdpr")
                        dismiss()
                    }
                }

                Section("Financial Compliance") {
                    SimulationButton(
                        title: "Capital Adequacy Breach",
                        description: "Simulate CET1 ratio dropping below Basel III minimum (4.5%)",
                        icon: "chart.line.downtrend.xyaxis",
                        color: .orange
                    ) {
                        alertService.simulateBreachDetection(breachType: "capital")
                        dismiss()
                    }

                    SimulationButton(
                        title: "Insider Trading Detection",
                        description: "Simulate suspicious trading pattern matching insider trading",
                        icon: "figure.walk.departure",
                        color: .red
                    ) {
                        alertService.simulateBreachDetection(breachType: "insider")
                        dismiss()
                    }
                }
            }
            .navigationTitle("Breach Simulator")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct SimulationButton: View {
    let title: String
    let description: String
    let icon: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 16) {
                ZStack {
                    Circle()
                        .fill(color.opacity(0.15))
                        .frame(width: 44, height: 44)

                    Image(systemName: icon)
                        .foregroundColor(color)
                }

                VStack(alignment: .leading, spacing: 4) {
                    Text(title)
                        .font(.subheadline)
                        .fontWeight(.semibold)
                        .foregroundColor(.primary)

                    Text(description)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(2)
                }

                Spacer()

                Image(systemName: "play.circle.fill")
                    .font(.title2)
                    .foregroundColor(color)
            }
        }
    }
}

#Preview {
    MonitoringView()
        .environmentObject(AlertService())
}
