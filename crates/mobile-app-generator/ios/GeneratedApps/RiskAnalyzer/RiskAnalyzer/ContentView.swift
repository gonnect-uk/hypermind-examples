//
// ContentView.swift
// Generated View
//

import SwiftUI

struct ContentView: View {
    @EnvironmentObject var sparqlService: SPARQLService
    @State private var isLoading = false
    @State private var results: [String] = []
    @State private var error: String?

    @State private var policyNumber: String = ""

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 20) {
                    VStack(alignment: .leading, spacing: 8) {
                    Text("Policy Number")
                        .font(.headline)
                    TextField("Enter policy number", text: $policyNumber)
                        .textFieldStyle(.roundedBorder)
                        .border(Color.red.opacity(0.3), width: 1)
                }

                    Button(action: {
                        Task {
                            await executeQuery()
                        }
                    }) {
                        if isLoading {
                            ProgressView()
                                .progressViewStyle(.circular)
                        } else {
                            Label("Search", systemImage: "magnifyingglass")
                        }
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(isLoading)

                    if let error = error {
                        Text(error)
                            .foregroundColor(.red)
                            .font(.caption)
                    }

                    if !results.isEmpty {
                        VStack(alignment: .leading, spacing: 12) {
                            Text("Results")
                                .font(.headline)
                            ForEach(results, id: \.self) { result in
                                Text(result)
                                    .padding()
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .background(Color(.systemGray6))
                                    .cornerRadius(8)
                            }
                        }
                    }
                }
                .padding()
            }
            .navigationTitle("Search Policy")
        }
    }

    func executeQuery() async {
        isLoading = true
        error = nil

        do {
            results = try await sparqlService.executeQuery()
        } catch {
            self.error = error.localizedDescription
        }

        isLoading = false
    }
}

#Preview {
    ContentView()
        .environmentObject(SPARQLService())
}
