//
//  ExplainRecommendationView.swift
//  SmartSearchRecommender
//
//  Shows graph paths explaining why movies were recommended
//

import SwiftUI

struct ExplainRecommendationView: View {
    @Environment(\.dismiss) var dismiss
    let recommendations: [Recommendation]

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 20) {
                    // Header
                    VStack(spacing: 12) {
                        Image(systemName: "lightbulb.fill")
                            .font(.system(size: 60))
                            .foregroundColor(.orange)

                        Text("Why These Recommendations?")
                            .font(.title2)
                            .fontWeight(.bold)

                        Text("Our graph-based engine found these connections")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding()

                    Divider()

                    // Recommendations with explanations
                    ForEach(recommendations, id: \.movie.id) { recommendation in
                        RecommendationExplanationCard(recommendation: recommendation)
                            .padding(.horizontal)
                    }
                }
                .padding(.vertical)
            }
            .navigationTitle("Graph Insights")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct RecommendationExplanationCard: View {
    let recommendation: Recommendation
    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Movie Header
            HStack(spacing: 12) {
                ZStack {
                    LinearGradient(
                        colors: recommendation.movie.posterGradient.map { Color(hex: $0) },
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                    .frame(width: 60, height: 80)
                    .cornerRadius(8)

                    Image(systemName: recommendation.movie.posterSymbol)
                        .font(.title2)
                        .foregroundColor(.white)
                }

                VStack(alignment: .leading, spacing: 4) {
                    Text(recommendation.movie.title)
                        .font(.headline)

                    HStack {
                        Image(systemName: "star.fill")
                            .font(.caption)
                            .foregroundColor(.yellow)
                        Text(recommendation.movie.formattedRating)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }

                    // Match score
                    HStack(spacing: 4) {
                        Text("Match:")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                        Text("\(Int(recommendation.score * 100))%")
                            .font(.caption)
                            .fontWeight(.semibold)
                            .foregroundColor(.green)
                    }
                }

                Spacer()

                Button(action: { withAnimation { isExpanded.toggle() } }) {
                    Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                        .foregroundColor(.secondary)
                }
            }

            // Paths (expanded)
            if isExpanded {
                VStack(alignment: .leading, spacing: 12) {
                    ForEach(Array(recommendation.paths.enumerated()), id: \.offset) { index, path in
                        PathView(path: path, index: index + 1)
                    }
                }
                .padding(.top, 8)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}

struct PathView: View {
    let path: RecommendationPath
    let index: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Path \(index)")
                    .font(.caption)
                    .fontWeight(.semibold)
                    .foregroundColor(.blue)

                Spacer()

                // Confidence badge
                Text("\(Int(path.confidence * 100))%")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(confidenceColor(path.confidence).opacity(0.2))
                    .foregroundColor(confidenceColor(path.confidence))
                    .cornerRadius(8)
            }

            Text(path.reason)
                .font(.subheadline)
                .foregroundColor(.secondary)

            // Graph path visualization
            GraphPathVisualizer(path: path)
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(8)
    }

    private func confidenceColor(_ confidence: Double) -> Color {
        switch confidence {
        case 0.8...1.0: return .green
        case 0.6..<0.8: return .yellow
        default: return .orange
        }
    }
}

struct GraphPathVisualizer: View {
    let path: RecommendationPath

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            ForEach(Array(path.nodes.enumerated()), id: \.offset) { index, node in
                HStack(spacing: 8) {
                    // Node
                    HStack(spacing: 4) {
                        Image(systemName: nodeIcon(for: node))
                            .font(.caption)
                            .foregroundColor(.blue)
                        Text(formatNodeName(node))
                            .font(.caption2)
                            .lineLimit(1)
                    }
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.blue.opacity(0.1))
                    .cornerRadius(6)

                    // Edge
                    if index < path.edges.count {
                        HStack(spacing: 2) {
                            Text(formatEdgeName(path.edges[index]))
                                .font(.caption2)
                                .foregroundColor(.secondary)
                            Image(systemName: "arrow.right")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                    }
                }
            }
        }
        .padding(8)
        .background(Color(.systemGray6))
        .cornerRadius(8)
    }

    private func nodeIcon(for uri: String) -> String {
        if uri.contains("Film") {
            return "film"
        } else if uri.contains("Person") || uri.contains("Nolan") || uri.contains("DiCaprio") {
            return "person.fill"
        } else if uri.contains("Genre") {
            return "tag.fill"
        } else {
            return "circle.fill"
        }
    }

    private func formatNodeName(_ uri: String) -> String {
        let components = uri.split(separator: "/")
        let name = components.last?.replacingOccurrences(of: "_", with: " ") ?? uri
        return String(name)
    }

    private func formatEdgeName(_ edge: String) -> String {
        let name = edge.split(separator: ":").last ?? edge.split(separator: "/").last ?? ""
        return String(name)
    }
}

struct GraphInsightCard: View {
    let icon: String
    let title: String
    let description: String
    let color: Color

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(color.opacity(0.2))
                    .frame(width: 50, height: 50)

                Image(systemName: icon)
                    .foregroundColor(color)
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.headline)
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}

#Preview {
    ExplainRecommendationView(recommendations: [
        Recommendation(
            movie: Movie.sample,
            score: 0.85,
            paths: [
                RecommendationPath(
                    reason: "Same director",
                    confidence: 0.8,
                    nodes: ["Inception", "Christopher Nolan", "The Dark Knight"],
                    edges: ["directed by", "directed"]
                )
            ],
            primaryReason: "Also directed by Christopher Nolan"
        )
    ])
}
