//
// AuditTrailView.swift
// ComplianceGuardian
//
// W3C PROV provenance timeline for compliance auditing
//

import SwiftUI

struct AuditTrailView: View {
    @EnvironmentObject var alertService: AlertService
    @State private var selectedDate = Date()
    @State private var showingFilter = false

    var auditEvents: [AuditEvent] {
        // Combine active and historical alerts
        let activeEvents = alertService.activeAlerts.map { AuditEvent(from: $0, status: .active) }
        let historyEvents = alertService.alertHistory.map { AuditEvent(from: $0, status: .resolved) }

        return (activeEvents + historyEvents + sampleEvents)
            .sorted { $0.timestamp > $1.timestamp }
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Timeline Stats
                    AuditStatsCard(events: auditEvents)

                    // Timeline View
                    AuditTimeline(events: auditEvents)
                }
                .padding()
            }
            .navigationTitle("Audit Trail")
            .background(Color(.systemGroupedBackground))
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        showingFilter = true
                    } label: {
                        Image(systemName: "line.3.horizontal.decrease.circle")
                    }
                }
            }
            .sheet(isPresented: $showingFilter) {
                FilterSheet()
            }
        }
    }
}

// MARK: - Models

struct AuditEvent: Identifiable {
    let id: UUID
    let timestamp: Date
    let eventType: AuditEventType
    let title: String
    let description: String
    let actor: String
    let entityId: String
    let regulationId: String?
    let status: EventStatus
    let provenanceData: ProvenanceData

    // Memberwise initializer (needed because we have a custom init below)
    init(id: UUID, timestamp: Date, eventType: AuditEventType, title: String, description: String,
         actor: String, entityId: String, regulationId: String?, status: EventStatus,
         provenanceData: ProvenanceData) {
        self.id = id
        self.timestamp = timestamp
        self.eventType = eventType
        self.title = title
        self.description = description
        self.actor = actor
        self.entityId = entityId
        self.regulationId = regulationId
        self.status = status
        self.provenanceData = provenanceData
    }

    init(from alert: ComplianceAlert, status: EventStatus) {
        self.id = alert.id
        self.timestamp = alert.detectedAt
        self.eventType = .breachDetected
        self.title = alert.title
        self.description = alert.message
        self.actor = "Monitoring System"
        self.entityId = alert.transactionId ?? "N/A"
        self.regulationId = alert.regulationId
        self.status = status
        self.provenanceData = ProvenanceData(
            wasGeneratedBy: "Automated Monitoring",
            wasAttributedTo: "ComplianceGuardian AI",
            wasInformedBy: alert.regulationLabel
        )
    }
}

enum AuditEventType: String {
    case breachDetected = "Breach Detected"
    case complianceCheck = "Compliance Check"
    case regulationUpdated = "Regulation Updated"
    case auditPerformed = "Audit Performed"
    case remediation = "Remediation Action"
    case notificationSent = "Notification Sent"

    var icon: String {
        switch self {
        case .breachDetected: return "exclamationmark.shield.fill"
        case .complianceCheck: return "checkmark.shield.fill"
        case .regulationUpdated: return "arrow.triangle.2.circlepath"
        case .auditPerformed: return "magnifyingglass.circle.fill"
        case .remediation: return "wrench.and.screwdriver.fill"
        case .notificationSent: return "bell.badge.fill"
        }
    }

    var color: Color {
        switch self {
        case .breachDetected: return .red
        case .complianceCheck: return .green
        case .regulationUpdated: return .orange
        case .auditPerformed: return .blue
        case .remediation: return .purple
        case .notificationSent: return .cyan
        }
    }
}

enum EventStatus: String {
    case active = "Active"
    case inProgress = "In Progress"
    case resolved = "Resolved"
    case escalated = "Escalated"
}

struct ProvenanceData {
    let wasGeneratedBy: String  // W3C PROV: Activity
    let wasAttributedTo: String // W3C PROV: Agent
    let wasInformedBy: String   // W3C PROV: Entity
}

// MARK: - Stats Card

struct AuditStatsCard: View {
    let events: [AuditEvent]

    var totalEvents: Int { events.count }
    var activeEvents: Int { events.filter { $0.status == .active }.count }
    var resolvedEvents: Int { events.filter { $0.status == .resolved }.count }

    var body: some View {
        VStack(spacing: 16) {
            HStack {
                Text("Audit Statistics")
                    .font(.headline)
                Spacer()
            }

            HStack(spacing: 16) {
                StatBox(label: "Total", value: "\(totalEvents)", color: .blue)
                StatBox(label: "Active", value: "\(activeEvents)", color: .red)
                StatBox(label: "Resolved", value: "\(resolvedEvents)", color: .green)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct StatBox: View {
    let label: String
    let value: String
    let color: Color

    var body: some View {
        VStack(spacing: 8) {
            Text(value)
                .font(.title)
                .fontWeight(.bold)
                .foregroundColor(color)

            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding()
        .background(color.opacity(0.1))
        .cornerRadius(12)
    }
}

// MARK: - Timeline

struct AuditTimeline: View {
    let events: [AuditEvent]

    var groupedEvents: [(String, [AuditEvent])] {
        let grouped = Dictionary(grouping: events) { event in
            dateString(from: event.timestamp)
        }
        return grouped.sorted { $0.key > $1.key }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            HStack {
                Text("Timeline")
                    .font(.headline)
                Spacer()
            }

            ForEach(groupedEvents, id: \.0) { date, events in
                VStack(alignment: .leading, spacing: 16) {
                    Text(date)
                        .font(.subheadline)
                        .fontWeight(.bold)
                        .foregroundColor(.secondary)
                        .padding(.horizontal)

                    ForEach(events) { event in
                        TimelineEventRow(event: event)
                    }
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }

    private func dateString(from date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        return formatter.string(from: date)
    }
}

struct TimelineEventRow: View {
    let event: AuditEvent
    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                withAnimation(.spring()) {
                    isExpanded.toggle()
                }
            } label: {
                HStack(spacing: 12) {
                    // Timeline dot and line
                    VStack(spacing: 0) {
                        Circle()
                            .fill(event.eventType.color)
                            .frame(width: 12, height: 12)

                        Rectangle()
                            .fill(Color.gray.opacity(0.3))
                            .frame(width: 2)
                    }

                    // Event content
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Image(systemName: event.eventType.icon)
                                .foregroundColor(event.eventType.color)

                            Text(event.eventType.rawValue)
                                .font(.caption)
                                .fontWeight(.bold)
                                .foregroundColor(event.eventType.color)

                            Spacer()

                            Text(timeString(from: event.timestamp))
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }

                        Text(event.title)
                            .font(.subheadline)
                            .fontWeight(.semibold)
                            .lineLimit(isExpanded ? nil : 2)

                        HStack {
                            StatusBadge(status: event.status)

                            Spacer()

                            Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }

                        if isExpanded {
                            expandedContent
                        }
                    }
                    .padding(.vertical, 8)
                }
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal)
    }

    private var expandedContent: some View {
        VStack(alignment: .leading, spacing: 12) {
            Divider()

            // Description
            Text(event.description)
                .font(.caption)
                .foregroundColor(.secondary)

            // Provenance (W3C PROV)
            VStack(alignment: .leading, spacing: 8) {
                Text("Provenance (W3C PROV)")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)

                ProvenanceRow(label: "wasGeneratedBy", value: event.provenanceData.wasGeneratedBy)
                ProvenanceRow(label: "wasAttributedTo", value: event.provenanceData.wasAttributedTo)
                ProvenanceRow(label: "wasInformedBy", value: event.provenanceData.wasInformedBy)
            }
            .padding(12)
            .background(Color(.secondarySystemGroupedBackground))
            .cornerRadius(8)

            // Metadata
            VStack(alignment: .leading, spacing: 6) {
                MetadataRow(label: "Actor", value: event.actor)
                MetadataRow(label: "Entity ID", value: event.entityId)
                if let regulationId = event.regulationId {
                    MetadataRow(label: "Regulation", value: regulationId.components(separatedBy: "#").last ?? regulationId)
                }
            }
        }
    }

    private func timeString(from date: Date) -> String {
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

struct StatusBadge: View {
    let status: EventStatus

    var color: Color {
        switch status {
        case .active: return .red
        case .inProgress: return .orange
        case .resolved: return .green
        case .escalated: return .purple
        }
    }

    var body: some View {
        Text(status.rawValue)
            .font(.caption2)
            .fontWeight(.bold)
            .foregroundColor(color)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(color.opacity(0.15))
            .cornerRadius(6)
    }
}

struct ProvenanceRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text("\(label):")
                .font(.caption2)
                .foregroundColor(.secondary)
                .fontWeight(.medium)
                .frame(width: 120, alignment: .leading)

            Text(value)
                .font(.caption2)
                .foregroundColor(.primary)
        }
    }
}

struct MetadataRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
            Spacer()
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
        }
    }
}

// MARK: - Filter Sheet

struct FilterSheet: View {
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section("Event Type") {
                    ForEach([AuditEventType.breachDetected, .complianceCheck, .regulationUpdated], id: \.self) { type in
                        Toggle(type.rawValue, isOn: .constant(true))
                    }
                }

                Section("Status") {
                    ForEach([EventStatus.active, .inProgress, .resolved], id: \.self) { status in
                        Toggle(status.rawValue, isOn: .constant(true))
                    }
                }

                Section("Date Range") {
                    DatePicker("From", selection: .constant(Date().addingTimeInterval(-7*86400)), displayedComponents: .date)
                    DatePicker("To", selection: .constant(Date()), displayedComponents: .date)
                }
            }
            .navigationTitle("Filter Events")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

// MARK: - Sample Data

let sampleEvents: [AuditEvent] = [
    AuditEvent(
        id: UUID(),
        timestamp: Date().addingTimeInterval(-3600),
        eventType: .complianceCheck,
        title: "Automated Compliance Scan Completed",
        description: "Daily compliance check performed on all active transactions. 1,245 transactions scanned, 2 flagged for review.",
        actor: "System Scheduler",
        entityId: "scan-2025-11-18",
        regulationId: nil,
        status: .resolved,
        provenanceData: ProvenanceData(
            wasGeneratedBy: "Scheduled Compliance Check",
            wasAttributedTo: "ComplianceGuardian Automated Scanner",
            wasInformedBy: "Daily Compliance Policy"
        )
    ),
    AuditEvent(
        id: UUID(),
        timestamp: Date().addingTimeInterval(-7200),
        eventType: .notificationSent,
        title: "EDPB Breach Notification Sent",
        description: "Formal breach notification submitted to European Data Protection Board regarding database access incident.",
        actor: "Compliance Officer (Jane Smith)",
        entityId: "notification-edpb-001",
        regulationId: "http://example.org/gdpr#Article_33_Breach_Notification",
        status: .resolved,
        provenanceData: ProvenanceData(
            wasGeneratedBy: "GDPR Breach Notification Process",
            wasAttributedTo: "Jane Smith (Chief Compliance Officer)",
            wasInformedBy: "GDPR Article 33 Requirements"
        )
    ),
    AuditEvent(
        id: UUID(),
        timestamp: Date().addingTimeInterval(-10800),
        eventType: .remediation,
        title: "Capital Reserves Increased",
        description: "Emergency capital injection of $50M completed to restore CET1 ratio to 5.2% (above minimum 4.5% threshold).",
        actor: "Finance Team",
        entityId: "capital-injection-001",
        regulationId: "http://example.org/regulation#Basel_III",
        status: .resolved,
        provenanceData: ProvenanceData(
            wasGeneratedBy: "Capital Adequacy Remediation",
            wasAttributedTo: "CFO & Board of Directors",
            wasInformedBy: "Basel III Capital Requirements"
        )
    )
]

#Preview {
    AuditTrailView()
        .environmentObject(AlertService())
}
