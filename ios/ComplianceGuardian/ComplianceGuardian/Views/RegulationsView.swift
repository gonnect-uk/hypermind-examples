//
// RegulationsView.swift
// ComplianceGuardian
//
// Swipeable regulation cards with penalty details
//

import SwiftUI

struct RegulationsView: View {
    @EnvironmentObject var complianceService: ComplianceService
    @State private var selectedRisk: RiskLevel? = nil

    var filteredRegulations: [Regulation] {
        if let selectedRisk = selectedRisk {
            return complianceService.regulations.filter { $0.riskLevel == selectedRisk }
        }
        return complianceService.regulations
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Risk Filter
                RiskFilterBar(selectedRisk: $selectedRisk)

                if complianceService.isLoading {
                    ProgressView("Loading regulations...")
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if filteredRegulations.isEmpty {
                    EmptyStateView()
                } else {
                    ScrollView {
                        LazyVStack(spacing: 16) {
                            ForEach(filteredRegulations) { regulation in
                                RegulationCard(regulation: regulation)
                            }
                        }
                        .padding()
                    }
                }
            }
            .navigationTitle("Regulations")
            .background(Color(.systemGroupedBackground))
        }
    }
}

// MARK: - Risk Filter Bar

struct RiskFilterBar: View {
    @Binding var selectedRisk: RiskLevel?

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 12) {
                FilterChip(label: "All", isSelected: selectedRisk == nil) {
                    selectedRisk = nil
                }

                ForEach(RiskLevel.allCases, id: \.self) { risk in
                    FilterChip(
                        label: risk.rawValue,
                        isSelected: selectedRisk == risk,
                        color: Color(hex: risk.color)
                    ) {
                        selectedRisk = risk
                    }
                }
            }
            .padding(.horizontal)
            .padding(.vertical, 12)
        }
        .background(Color(.systemBackground))
    }
}

struct FilterChip: View {
    let label: String
    let isSelected: Bool
    var color: Color = .blue

    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(label)
                .font(.subheadline)
                .fontWeight(.semibold)
                .foregroundColor(isSelected ? .white : color)
                .padding(.horizontal, 16)
                .padding(.vertical, 8)
                .background(isSelected ? color : color.opacity(0.1))
                .cornerRadius(20)
        }
    }
}

// MARK: - Regulation Card

struct RegulationCard: View {
    let regulation: Regulation
    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            // Header
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Image(systemName: regulation.riskLevel.icon)
                            .font(.title3)
                            .foregroundColor(Color(hex: regulation.riskLevel.color))

                        Text(regulation.riskLevel.rawValue)
                            .font(.caption)
                            .fontWeight(.bold)
                            .foregroundColor(Color(hex: regulation.riskLevel.color))
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .background(Color(hex: regulation.riskLevel.color).opacity(0.15))
                            .cornerRadius(6)

                        Spacer()
                    }

                    Text(regulation.label)
                        .font(.title2)
                        .fontWeight(.bold)
                        .lineLimit(2)
                }
            }

            // Jurisdiction & Body
            HStack(spacing: 16) {
                InfoPill(icon: "globe", text: regulation.jurisdiction)
                InfoPill(icon: "building.2", text: regulation.regulatoryBody)
            }

            Divider()

            // Description
            Text(regulation.description)
                .font(.subheadline)
                .foregroundColor(.secondary)
                .lineLimit(isExpanded ? nil : 3)

            // Penalty Section
            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Image(systemName: "dollarsign.circle.fill")
                        .foregroundColor(.red)
                    Text("Maximum Penalty")
                        .font(.headline)
                        .fontWeight(.bold)
                }

                VStack(alignment: .leading, spacing: 8) {
                    Text(regulation.formattedPenalty)
                        .font(.title3)
                        .fontWeight(.bold)
                        .foregroundColor(.red)

                    if let prison = regulation.maxPrison {
                        HStack(spacing: 6) {
                            Image(systemName: "figure.stand")
                                .font(.caption)
                            Text("Up to \(prison) years imprisonment")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }

                    if let sanctions = regulation.additionalSanctions {
                        HStack(alignment: .top, spacing: 6) {
                            Image(systemName: "exclamationmark.triangle.fill")
                                .font(.caption)
                                .foregroundColor(.orange)
                            Text(sanctions)
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                }
                .padding()
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.red.opacity(0.05))
                .cornerRadius(12)
            }

            // Response Time (for GDPR)
            if let countdown = regulation.breachCountdown {
                HStack {
                    Image(systemName: "timer")
                        .foregroundColor(.orange)
                    Text("Response time: \(countdown)")
                        .font(.subheadline)
                        .fontWeight(.semibold)
                        .foregroundColor(.orange)
                }
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.orange.opacity(0.1))
                .cornerRadius(12)
            }

            // Dates
            VStack(alignment: .leading, spacing: 8) {
                DateRow(label: "Enacted", date: regulation.enactmentDate)
                if let amended = regulation.lastAmended {
                    DateRow(label: "Last Amended", date: amended)
                }
            }

            // Expand/Collapse Button
            Button {
                withAnimation(.spring()) {
                    isExpanded.toggle()
                }
            } label: {
                HStack {
                    Text(isExpanded ? "Show Less" : "Show More")
                        .font(.subheadline)
                        .fontWeight(.semibold)
                    Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                        .font(.caption)
                }
                .foregroundColor(.blue)
                .frame(maxWidth: .infinity)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.05), radius: 8, x: 0, y: 2)
    }
}

struct InfoPill: View {
    let icon: String
    let text: String

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: icon)
                .font(.caption)
            Text(text)
                .font(.caption)
                .fontWeight(.medium)
        }
        .foregroundColor(.secondary)
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color(.secondarySystemGroupedBackground))
        .cornerRadius(12)
    }
}

struct DateRow: View {
    let label: String
    let date: Date

    var formattedDate: String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        return formatter.string(from: date)
    }

    var body: some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
            Spacer()
            Text(formattedDate)
                .font(.caption)
                .fontWeight(.medium)
        }
    }
}

// MARK: - Empty State

struct EmptyStateView: View {
    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "doc.text.magnifyingglass")
                .font(.system(size: 64))
                .foregroundColor(.secondary)

            Text("No Regulations Found")
                .font(.title2)
                .fontWeight(.bold)

            Text("Regulations will appear here once the dataset is loaded")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
        }
        .padding()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

#Preview {
    RegulationsView()
        .environmentObject(ComplianceService())
}
