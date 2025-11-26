//
// AlertService.swift
// ComplianceGuardian
//
// Real-time alert monitoring with countdown timers
//

import Foundation
import Combine

@MainActor
class AlertService: ObservableObject {
    @Published var activeAlerts: [ComplianceAlert] = []
    @Published var alertHistory: [ComplianceAlert] = []

    private var timer: Timer?
    private var complianceService: ComplianceService?

    // MARK: - Monitoring

    func startMonitoring(complianceService: ComplianceService) {
        self.complianceService = complianceService

        // Initialize with sample alerts for demo
        generateSampleAlerts()

        // Start countdown timer (updates every second)
        timer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.updateAlerts()
            }
        }
    }

    func stopMonitoring() {
        timer?.invalidate()
        timer = nil
    }

    private func updateAlerts() {
        // Remove expired alerts
        activeAlerts = activeAlerts.filter { alert in
            if alert.isExpired {
                alertHistory.append(alert)
                return false
            }
            return true
        }

        // Trigger notifications for urgent alerts (< 1 hour remaining)
        for alert in activeAlerts {
            if let remaining = alert.timeRemaining, remaining < 3600 && remaining > 3500 {
                sendNotification(for: alert)
            }
        }
    }

    // MARK: - Alert Generation

    func generateSampleAlerts() {
        // GDPR breach notification (72-hour countdown)
        let gdprAlert = ComplianceAlert(
            id: UUID(),
            regulationId: "http://example.org/gdpr#Article_33_Breach_Notification",
            regulationLabel: "GDPR Article 33: Breach Notification",
            severity: .critical,
            title: "Data Breach Detected",
            message: "Unauthorized access to customer database detected. GDPR requires notification to supervisory authority within 72 hours of discovery.",
            detectedAt: Date().addingTimeInterval(-14400), // 4 hours ago
            deadline: Date().addingTimeInterval(244800), // 68 hours remaining (72 - 4)
            actionRequired: "Submit breach notification form to EDPB with details of affected records, nature of breach, and mitigation measures",
            transactionId: nil
        )

        // SEC insider trading violation
        let secAlert = ComplianceAlert(
            id: UUID(),
            regulationId: "http://example.org/regulation#SEC_Rule_10b5",
            regulationLabel: "SEC Rule 10b-5",
            severity: .critical,
            title: "Potential Insider Trading Detected",
            message: "Employee emp123 on restricted trading list executed purchase of 1,000 AAPL shares 24 hours before earnings announcement.",
            detectedAt: Date().addingTimeInterval(-1800), // 30 minutes ago
            deadline: Date().addingTimeInterval(82800), // 23 hours to investigate
            actionRequired: "Investigate trading activity, review internal communications, and report findings to Chief Compliance Officer",
            transactionId: "txn001"
        )

        // Basel III capital ratio breach
        let baselAlert = ComplianceAlert(
            id: UUID(),
            regulationId: "http://example.org/regulation#Basel_III",
            regulationLabel: "Basel III: Capital Requirements",
            severity: .high,
            title: "CET1 Ratio Below Minimum Threshold",
            message: "Common Equity Tier 1 capital ratio dropped to 4.3% (minimum required: 4.5%). Bank is undercapitalized per Basel III standards.",
            detectedAt: Date().addingTimeInterval(-600), // 10 minutes ago
            deadline: Date().addingTimeInterval(86400), // 24 hours to remedy
            actionRequired: "Increase capital reserves by raising equity, reducing risk-weighted assets, or restricting dividend payments",
            transactionId: nil
        )

        // MiFID II transaction reporting delay
        let mifidAlert = ComplianceAlert(
            id: UUID(),
            regulationId: "http://example.org/regulation#MiFID_II",
            regulationLabel: "MiFID II: Transaction Reporting",
            severity: .high,
            title: "T+1 Reporting Deadline Approaching",
            message: "847 trades from yesterday not yet reported to FCA. MiFID II requires reporting by end of next business day (T+1).",
            detectedAt: Date().addingTimeInterval(-3600), // 1 hour ago
            deadline: Date().addingTimeInterval(28800), // 8 hours until deadline
            actionRequired: "Submit transaction reports to Approved Reporting Mechanism (ARM) before 23:59 GMT",
            transactionId: nil
        )

        activeAlerts = [gdprAlert, secAlert, baselAlert, mifidAlert]
    }

    func createAlert(
        regulationId: String,
        regulationLabel: String,
        severity: RiskLevel,
        title: String,
        message: String,
        deadline: Date?,
        actionRequired: String,
        transactionId: String? = nil
    ) {
        let alert = ComplianceAlert(
            id: UUID(),
            regulationId: regulationId,
            regulationLabel: regulationLabel,
            severity: severity,
            title: title,
            message: message,
            detectedAt: Date(),
            deadline: deadline,
            actionRequired: actionRequired,
            transactionId: transactionId
        )

        activeAlerts.append(alert)
        sendNotification(for: alert)
    }

    func dismissAlert(_ alert: ComplianceAlert) {
        if let index = activeAlerts.firstIndex(where: { $0.id == alert.id }) {
            let dismissed = activeAlerts.remove(at: index)
            alertHistory.append(dismissed)
        }
    }

    func acknowledgeAlert(_ alert: ComplianceAlert) {
        // Mark as acknowledged (in production, would update backend)
        print("Alert acknowledged: \(alert.title)")
    }

    // MARK: - Notifications

    private func sendNotification(for alert: ComplianceAlert) {
        // In production, would use UNUserNotificationCenter
        print("🚨 URGENT: \(alert.title) - \(alert.formattedTimeRemaining ?? "No deadline")")
    }

    // MARK: - Breach Detection (Real-time monitoring)

    func simulateBreachDetection(breachType: String) {
        switch breachType {
        case "gdpr":
            createAlert(
                regulationId: "http://example.org/gdpr#Article_33_Breach_Notification",
                regulationLabel: "GDPR Article 33",
                severity: .critical,
                title: "Personal Data Breach",
                message: "Simulated breach: Database backup exposed to unauthorized third party. Contains personal data of 15,000 EU residents.",
                deadline: Date().addingTimeInterval(72 * 3600), // 72 hours
                actionRequired: "Document breach, assess risk, notify EDPB within 72 hours, notify affected individuals if high risk"
            )

        case "capital":
            createAlert(
                regulationId: "http://example.org/regulation#Basel_III",
                regulationLabel: "Basel III",
                severity: .critical,
                title: "Capital Adequacy Alert",
                message: "Simulated scenario: Major loan default caused CET1 ratio to drop to 3.8% (minimum: 4.5%).",
                deadline: Date().addingTimeInterval(24 * 3600), // 24 hours
                actionRequired: "Emergency capital injection required. Contact board of directors and prepare regulatory filing."
            )

        case "insider":
            createAlert(
                regulationId: "http://example.org/regulation#SEC_Rule_10b5",
                regulationLabel: "SEC Rule 10b-5",
                severity: .critical,
                title: "Insider Trading Flagged",
                message: "Simulated detection: Executive traded company stock 48 hours before merger announcement. Pattern matches insider trading.",
                deadline: Date().addingTimeInterval(48 * 3600), // 48 hours to investigate
                actionRequired: "Freeze trading accounts, initiate internal investigation, preserve all communications"
            )

        default:
            break
        }
    }

    // Note: Timer will be automatically invalidated when object is deallocated
    // No need for deinit since deinit can't call @MainActor methods
}
