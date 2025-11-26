//
// DashboardView.swift
// ComplianceGuardian
//
// Real-time compliance monitoring dashboard with risk heatmap
//

import SwiftUI
import Charts

struct DashboardView: View {
    @EnvironmentObject var complianceService: ComplianceService
    @EnvironmentObject var alertService: AlertService

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Compliance Score Card
                    ComplianceScoreCard(score: complianceService.complianceScore)

                    // Active Alerts Section
                    ActiveAlertsSection(alerts: alertService.activeAlerts)

                    // Risk Heatmap
                    RiskHeatmapSection(regulations: complianceService.regulations)

                    // Critical Deadlines
                    CriticalDeadlinesSection(alerts: alertService.activeAlerts)

                    // Violation Summary
                    ViolationSummarySection(count: complianceService.violationCount)
                }
                .padding()
            }
            .navigationTitle("Compliance Dashboard")
            .background(Color(.systemGroupedBackground))
            .refreshable {
                await complianceService.updateComplianceMetrics()
            }
        }
    }
}

// MARK: - Compliance Score Card

struct ComplianceScoreCard: View {
    let score: Double

    var scoreColor: Color {
        switch score {
        case 90...100:
            return .green
        case 70..<90:
            return .orange
        default:
            return .red
        }
    }

    var scoreStatus: String {
        switch score {
        case 90...100:
            return "Excellent"
        case 70..<90:
            return "At Risk"
        default:
            return "Critical"
        }
    }

    var body: some View {
        VStack(spacing: 16) {
            HStack {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Compliance Score")
                        .font(.headline)
                        .foregroundColor(.secondary)

                    HStack(alignment: .firstTextBaseline, spacing: 4) {
                        Text("\(Int(score))")
                            .font(.system(size: 56, weight: .bold, design: .rounded))
                            .foregroundColor(scoreColor)

                        Text("/ 100")
                            .font(.title2)
                            .foregroundColor(.secondary)
                    }

                    Text(scoreStatus)
                        .font(.subheadline)
                        .fontWeight(.semibold)
                        .foregroundColor(scoreColor)
                }

                Spacer()

                ZStack {
                    Circle()
                        .stroke(Color.gray.opacity(0.2), lineWidth: 12)
                        .frame(width: 100, height: 100)

                    Circle()
                        .trim(from: 0, to: score / 100)
                        .stroke(scoreColor, style: StrokeStyle(lineWidth: 12, lineCap: .round))
                        .frame(width: 100, height: 100)
                        .rotationEffect(.degrees(-90))
                        .animation(.easeInOut, value: score)

                    Image(systemName: score >= 90 ? "checkmark.circle.fill" : "exclamationmark.triangle.fill")
                        .font(.system(size: 32))
                        .foregroundColor(scoreColor)
                }
            }

            Divider()

            HStack {
                ScoreMetric(label: "Regulations", value: "7", icon: "doc.text.fill")
                Spacer()
                ScoreMetric(label: "Requirements", value: "12", icon: "checklist")
                Spacer()
                ScoreMetric(label: "Violations", value: "2", icon: "xmark.circle.fill", color: .red)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct ScoreMetric: View {
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

// MARK: - Active Alerts Section

struct ActiveAlertsSection: View {
    let alerts: [ComplianceAlert]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "bell.badge.fill")
                    .foregroundColor(.red)
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
                Text("No active alerts")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()
            } else {
                ForEach(alerts.prefix(3)) { alert in
                    AlertCardCompact(alert: alert)
                }

                if alerts.count > 3 {
                    NavigationLink {
                        MonitoringView()
                    } label: {
                        Text("View all \(alerts.count) alerts →")
                            .font(.subheadline)
                            .fontWeight(.semibold)
                            .foregroundColor(.blue)
                    }
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct AlertCardCompact: View {
    let alert: ComplianceAlert

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: alert.severity.icon)
                .font(.title3)
                .foregroundColor(Color(hex: alert.severity.color))

            VStack(alignment: .leading, spacing: 4) {
                Text(alert.title)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .lineLimit(1)

                if let timeRemaining = alert.formattedTimeRemaining {
                    Text("⏱ \(timeRemaining) remaining")
                        .font(.caption)
                        .foregroundColor(Color(hex: alert.urgencyColor))
                        .fontWeight(.bold)
                }
            }

            Spacer()

            Image(systemName: "chevron.right")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(12)
        .background(Color(.secondarySystemGroupedBackground))
        .cornerRadius(12)
    }
}

// MARK: - Risk Heatmap

struct RiskHeatmapSection: View {
    let regulations: [Regulation]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "chart.bar.xaxis")
                    .foregroundColor(.orange)
                Text("Risk Heatmap")
                    .font(.headline)
                Spacer()
            }

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                ForEach(regulations.prefix(6)) { regulation in
                    RiskCell(regulation: regulation)
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct RiskCell: View {
    let regulation: Regulation

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: regulation.riskLevel.icon)
                    .font(.caption)
                    .foregroundColor(Color(hex: regulation.riskLevel.color))

                Spacer()

                Text(regulation.riskLevel.rawValue)
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(Color(hex: regulation.riskLevel.color))
            }

            Text(regulation.label)
                .font(.caption)
                .fontWeight(.semibold)
                .lineLimit(2)
                .fixedSize(horizontal: false, vertical: true)

            Text(regulation.formattedFine)
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding(12)
        .frame(maxWidth: .infinity, minHeight: 100, alignment: .topLeading)
        .background(Color(hex: regulation.riskLevel.color).opacity(0.1))
        .cornerRadius(12)
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color(hex: regulation.riskLevel.color), lineWidth: 2)
        )
    }
}

// MARK: - Critical Deadlines

struct CriticalDeadlinesSection: View {
    let alerts: [ComplianceAlert]

    var sortedAlerts: [ComplianceAlert] {
        alerts.filter { $0.deadline != nil }
            .sorted { ($0.timeRemaining ?? 0) < ($1.timeRemaining ?? 0) }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "timer")
                    .foregroundColor(.red)
                Text("Critical Deadlines")
                    .font(.headline)
                Spacer()
            }

            if sortedAlerts.isEmpty {
                Text("No pending deadlines")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()
            } else {
                ForEach(sortedAlerts.prefix(3)) { alert in
                    DeadlineRow(alert: alert)
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct DeadlineRow: View {
    let alert: ComplianceAlert

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(alert.regulationLabel)
                    .font(.subheadline)
                    .fontWeight(.semibold)

                Text(alert.title)
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            if let timeRemaining = alert.formattedTimeRemaining {
                VStack(alignment: .trailing, spacing: 4) {
                    Text(timeRemaining)
                        .font(.subheadline)
                        .fontWeight(.bold)
                        .foregroundColor(Color(hex: alert.urgencyColor))

                    Text("remaining")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(12)
        .background(Color(.secondarySystemGroupedBackground))
        .cornerRadius(12)
    }
}

// MARK: - Violation Summary

struct ViolationSummarySection: View {
    let count: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "xmark.shield.fill")
                    .foregroundColor(count > 0 ? .red : .green)
                Text("Violations")
                    .font(.headline)
                Spacer()
                Text("\(count)")
                    .font(.title2)
                    .fontWeight(.bold)
                    .foregroundColor(count > 0 ? .red : .green)
            }

            if count > 0 {
                Text("Active compliance violations detected. Immediate action required.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            } else {
                Text("No violations detected. All systems compliant.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

// MARK: - Color Extension

extension Color {
    init(hex: String) {
        let scanner = Scanner(string: hex)
        var rgbValue: UInt64 = 0
        scanner.scanHexInt64(&rgbValue)

        let r = Double((rgbValue & 0xFF0000) >> 16) / 255.0
        let g = Double((rgbValue & 0x00FF00) >> 8) / 255.0
        let b = Double(rgbValue & 0x0000FF) / 255.0

        self.init(red: r, green: g, blue: b)
    }
}

#Preview {
    DashboardView()
        .environmentObject(ComplianceService())
        .environmentObject(AlertService())
}
