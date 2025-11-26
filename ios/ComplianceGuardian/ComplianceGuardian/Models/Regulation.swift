//
// Regulation.swift
// ComplianceGuardian
//
// Regulatory compliance rule model
//

import Foundation

struct Regulation: Identifiable, Codable {
    let id: String
    let label: String
    let jurisdiction: String
    let regulatoryBody: String
    let enactmentDate: Date
    let lastAmended: Date?
    let riskLevel: RiskLevel
    let description: String
    let maxFine: Int
    let maxPrison: Int?
    let currency: String
    let percentageRevenue: Int?
    let responseTime: TimeInterval?
    let requirements: [ComplianceRequirement]
    let additionalSanctions: String?

    var formattedFine: String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = currency
        formatter.maximumFractionDigits = 0
        return formatter.string(from: NSNumber(value: maxFine)) ?? "\(currency) \(maxFine)"
    }

    var formattedPenalty: String {
        if let percentageRevenue = percentageRevenue {
            return "Up to \(formattedFine) or \(percentageRevenue)% of annual revenue (whichever is higher)"
        }
        return formattedFine
    }

    var riskColor: String {
        switch riskLevel {
        case .critical:
            return "FF3B30"
        case .high:
            return "FF9500"
        case .medium:
            return "FFCC00"
        case .low:
            return "34C759"
        }
    }

    var breachCountdown: String? {
        guard let responseTime = responseTime else { return nil }
        let hours = Int(responseTime / 3600)
        return "\(hours) hours"
    }
}

enum RiskLevel: String, Codable, CaseIterable {
    case critical = "Critical"
    case high = "High"
    case medium = "Medium"
    case low = "Low"

    var priority: Int {
        switch self {
        case .critical: return 1
        case .high: return 2
        case .medium: return 3
        case .low: return 4
        }
    }

    var color: String {
        switch self {
        case .critical: return "FF3B30"
        case .high: return "FF9500"
        case .medium: return "FFCC00"
        case .low: return "34C759"
        }
    }

    var icon: String {
        switch self {
        case .critical: return "exclamationmark.triangle.fill"
        case .high: return "exclamationmark.circle.fill"
        case .medium: return "info.circle.fill"
        case .low: return "checkmark.circle.fill"
        }
    }
}

// Sample data for previews
extension Regulation {
    static let sampleSEC = Regulation(
        id: "http://example.org/regulation#SEC_Rule_10b5",
        label: "SEC Rule 10b-5",
        jurisdiction: "United States",
        regulatoryBody: "SEC",
        enactmentDate: Date(timeIntervalSince1970: -880588800), // 1942-05-21
        lastAmended: Date(timeIntervalSince1970: 1279670400), // 2010-07-21
        riskLevel: .critical,
        description: "Prohibits fraud in connection with the purchase or sale of securities",
        maxFine: 5000000,
        maxPrison: 20,
        currency: "USD",
        percentageRevenue: nil,
        responseTime: nil,
        requirements: [],
        additionalSanctions: nil
    )

    static let sampleGDPR = Regulation(
        id: "http://example.org/gdpr#Article_33_Breach_Notification",
        label: "GDPR Article 33",
        jurisdiction: "European Union",
        regulatoryBody: "EDPB",
        enactmentDate: Date(timeIntervalSince1970: 1527206400), // 2018-05-25
        lastAmended: nil,
        riskLevel: .critical,
        description: "Requires notification within 72 hours of becoming aware of breach",
        maxFine: 10000000,
        maxPrison: nil,
        currency: "EUR",
        percentageRevenue: 2,
        responseTime: 72 * 3600, // 72 hours in seconds
        requirements: [],
        additionalSanctions: nil
    )
}
