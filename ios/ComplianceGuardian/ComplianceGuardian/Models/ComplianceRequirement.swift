//
// ComplianceRequirement.swift
// ComplianceGuardian
//
// Individual compliance requirement model
//

import Foundation

struct ComplianceRequirement: Identifiable, Codable {
    let id: String
    let label: String
    let description: String
    let monitoringFrequency: String?
    let deadline: TimeInterval?
    let automatable: Bool
    let implementationCost: String?
    let threshold: Double?
    let critical: Bool

    var deadlineFormatted: String? {
        guard let deadline = deadline else { return nil }
        let hours = Int(deadline / 3600)
        let days = hours / 24

        if days > 0 {
            return "\(days) day\(days == 1 ? "" : "s")"
        } else {
            return "\(hours) hour\(hours == 1 ? "" : "s")"
        }
    }

    var statusIcon: String {
        if critical {
            return "exclamationmark.triangle.fill"
        } else if automatable {
            return "gear.badge.checkmark"
        } else {
            return "person.fill.checkmark"
        }
    }
}

// Sample data
extension ComplianceRequirement {
    static let sampleInsiderTrading = ComplianceRequirement(
        id: "http://example.org/compliance#requirement_insider_trading_monitoring",
        label: "Insider Trading Monitoring",
        description: "Real-time monitoring of employee trading activities",
        monitoringFrequency: "Real-time",
        deadline: nil,
        automatable: true,
        implementationCost: "High",
        threshold: nil,
        critical: true
    )

    static let sampleBreachNotification = ComplianceRequirement(
        id: "http://example.org/compliance#requirement_breach_reporting",
        label: "Breach Reporting Process",
        description: "Report data breaches to authorities within 72 hours",
        monitoringFrequency: nil,
        deadline: 72 * 3600,
        automatable: false,
        implementationCost: nil,
        threshold: nil,
        critical: true
    )

    static let sampleCapitalRatio = ComplianceRequirement(
        id: "http://example.org/compliance#requirement_capital_ratio",
        label: "Minimum Capital Ratio",
        description: "Maintain Common Equity Tier 1 (CET1) ratio ≥ 4.5%",
        monitoringFrequency: "Daily",
        deadline: nil,
        automatable: true,
        implementationCost: nil,
        threshold: 4.5,
        critical: true
    )
}
