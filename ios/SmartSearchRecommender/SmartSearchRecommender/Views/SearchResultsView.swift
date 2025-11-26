//
//  SearchResultsView.swift
//  SmartSearchRecommender
//
//  Grid-based search results with filters
//

import SwiftUI

struct SearchResultsView: View {
    @Binding var searchQuery: String
    @EnvironmentObject var movieService: MovieService
    @State private var searchText = ""
    @State private var searchResults: [Movie] = []
    @State private var isSearching = false
    @State private var filter = MovieFilter()

    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Search Bar
                SearchBar(text: $searchText) {
                    performSearch()
                }
                .padding()

                if isSearching {
                    Spacer()
                    ProgressView("Searching...")
                    Spacer()
                } else if searchText.isEmpty {
                    EmptySearchView()
                } else if searchResults.isEmpty {
                    NoResultsView(searchText: searchText)
                } else {
                    // Results Grid
                    ScrollView {
                        LazyVGrid(columns: [
                            GridItem(.adaptive(minimum: 160), spacing: 16)
                        ], spacing: 16) {
                            ForEach(searchResults) { movie in
                                NavigationLink(destination: MovieDetailView(movie: movie)) {
                                    MovieCard(movie: movie)
                                }
                            }
                        }
                        .padding()
                    }
                }
            }
            .navigationTitle("Search")
        }
    }

    private func performSearch() {
        guard !searchText.isEmpty else {
            searchResults = []
            return
        }

        isSearching = true

        Task {
            // Use MovieService to search
            searchResults = await movieService.searchMovies(query: searchText)
            isSearching = false
        }
    }
}

struct EmptySearchView: View {
    var body: some View {
        VStack(spacing: 20) {
            Spacer()

            Image(systemName: "magnifyingglass")
                .font(.system(size: 70))
                .foregroundColor(.secondary)

            Text("Search for Movies")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Find movies by title, director, actor, or genre")
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 40)

            Spacer()
        }
    }
}

struct NoResultsView: View {
    let searchText: String

    var body: some View {
        VStack(spacing: 20) {
            Spacer()

            Image(systemName: "exclamationmark.magnifyingglass")
                .font(.system(size: 70))
                .foregroundColor(.secondary)

            Text("No Results Found")
                .font(.title2)
                .fontWeight(.semibold)

            Text("We couldn't find any movies matching '\(searchText)'")
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 40)

            Spacer()
        }
    }
}

#Preview {
    SearchResultsView(searchQuery: .constant(""))
        .environmentObject(MovieService.shared)
}
