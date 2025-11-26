//
// Alert.swift
// ComplianceGuardian
//
// Compliance alert model with countdown timers
//

import Foundation

struct ComplianceAlert: Identifiable, Codable {
    let id: UUID
    let regulationId: String
    let regulationLabel: String
    let severity: RiskLevel
    let title: String
    let message: String
    let detectedAt: Date
    let deadline: Date?
    let actionRequired: String
    let transactionId: String?

    var isExpired: Bool {
        guard let deadline = deadline else { return false }
        return Date() > deadline
    }

    var timeRemaining: TimeInterval? {
        guard let deadline = deadline else { return nil }
        return max(0, deadline.timeIntervalSince(Date()))
    }

    var formattedTimeRemaining: String? {
        guard let remaining = timeRemaining else { return nil }

        let hours = Int(remaining / 3600)
        let minutes = Int((remaining.truncatingRemainder(dividingBy: 3600)) / 60)
        let seconds = Int(remaining.truncatingRemainder(dividingBy: 60))

        if hours > 24 {
            let days = hours / 24
            let remainingHours = hours % 24
            return "\(days)d \(remainingHours)h"
        } else if hours > 0 {
            return "\(hours)h \(minutes)m"
        } else if minutes > 0 {
            return "\(minutes)m \(seconds)s"
        } else {
            return "\(seconds)s"
        }
    }

    var urgencyColor: String {
        guard let remaining = timeRemaining else {
            return severity.color
        }

        if isExpired {
            return "FF3B30" // Red
        } else if remaining < 3600 { // Less than 1 hour
            return "FF3B30" // Red
        } else if remaining < 21600 { // Less than 6 hours
            return "FF9500" // Orange
        } else if remaining < 43200 { // Less than 12 hours
            return "FFCC00" // Yellow
        } else {
            return "34C759" // Green
        }
    }
}

// Sample alerts
extension ComplianceAlert {
    static let sampleGDPRBreach = ComplianceAlert(
        id: UUID(),
        regulationId: "http://example.org/gdpr#Article_33_Breach_Notification",
        regulationLabel: "GDPR Article 33",
        severity: .critical,
        title: "Data Breach Detected",
        message: "Unauthorized access detected in customer database. Breach notification required within 72 hours.",
        detectedAt: Date().addingTimeInterval(-14400), // 4 hours ago
        deadline: Date().addingTimeInterval(244800), // 68 hours from now (72 - 4)
        actionRequired: "Submit breach notification to EDPB",
        transactionId: nil
    )

    static let sampleInsiderTrading = ComplianceAlert(
        id: UUID(),
        regulationId: "http://example.org/regulation#SEC_Rule_10b5",
        regulationLabel: "SEC Rule 10b-5",
        severity: .critical,
        title: "Suspicious Trading Activity",
        message: "Employee on restricted list executed trade for AAPL (1000 shares). Potential insider trading violation.",
        detectedAt: Date().addingTimeInterval(-1800), // 30 minutes ago
        deadline: nil,
        actionRequired: "Investigate and report to compliance officer",
        transactionId: "txn001"
    )

    static let sampleCapitalRatio = ComplianceAlert(
        id: UUID(),
        regulationId: "http://example.org/regulation#Basel_III",
        regulationLabel: "Basel III",
        severity: .high,
        title: "Capital Ratio Below Threshold",
        message: "CET1 ratio dropped to 4.3% (minimum: 4.5%). Immediate capital injection required.",
        detectedAt: Date().addingTimeInterval(-300), // 5 minutes ago
        deadline: Date().addingTimeInterval(86400), // 24 hours
        actionRequired: "Increase capital reserves to meet Basel III requirements",
        transactionId: nil
    )
}
