//
//  MovieDetailView.swift
//  SmartSearchRecommender
//
//  Detailed movie view with cast, ratings, and graph-based recommendations
//

import SwiftUI

struct MovieDetailView: View {
    let movie: Movie
    @EnvironmentObject var movieService: MovieService
    @StateObject private var recommendationEngine: RecommendationEngineWrapper
    @State private var similarMovies: [Recommendation] = []
    @State private var showExplanation = false
    @State private var isLoading = false

    init(movie: Movie) {
        self.movie = movie
        _recommendationEngine = StateObject(wrappedValue: RecommendationEngineWrapper())
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                // Hero Section with Poster
                HeroSection(movie: movie)

                // Movie Info
                VStack(alignment: .leading, spacing: 16) {
                    // Title and Rating
                    HStack(alignment: .top) {
                        VStack(alignment: .leading, spacing: 8) {
                            Text(movie.title)
                                .font(.title)
                                .fontWeight(.bold)

                            Text(movie.releaseYear)
                                .font(.subheadline)
                                .foregroundColor(.secondary)
                        }

                        Spacer()

                        // Rating Badge
                        VStack(spacing: 4) {
                            Image(systemName: "star.fill")
                                .foregroundColor(.yellow)
                                .font(.title2)
                            Text(movie.formattedRating)
                                .font(.title3)
                                .fontWeight(.bold)
                        }
                        .padding()
                        .background(Color(.systemGray6))
                        .cornerRadius(12)
                    }

                    // Genres
                    GenresRow(genres: movie.genres)

                    // Description
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Synopsis")
                            .font(.headline)
                        Text(movie.description)
                            .font(.body)
                            .foregroundColor(.secondary)
                    }

                    Divider()

                    // Director
                    DirectorRow(director: movie.director, directorURI: movie.directorURI)

                    Divider()

                    // Cast
                    CastSection(cast: movie.cast, castURIs: movie.castURIs)

                    Divider()

                    // Similar Movies
                    SimilarMoviesSection(
                        similarMovies: similarMovies,
                        isLoading: isLoading,
                        onShowExplanation: { showExplanation = true }
                    )
                }
                .padding()
            }
        }
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: { movieService.toggleFavorite(movie) }) {
                    Image(systemName: movieService.isFavorite(movie) ? "heart.fill" : "heart")
                        .foregroundColor(movieService.isFavorite(movie) ? .red : .primary)
                }
            }
        }
        .sheet(isPresented: $showExplanation) {
            ExplainRecommendationView(recommendations: similarMovies)
        }
        .task {
            await loadSimilarMovies()
        }
    }

    private func loadSimilarMovies() async {
        isLoading = true
        recommendationEngine.setMovieService(movieService)
        similarMovies = await recommendationEngine.engine.findSimilarMovies(to: movie, limit: 10)
        isLoading = false
    }
}

struct HeroSection: View {
    let movie: Movie

    var body: some View {
        ZStack(alignment: .bottom) {
            // Poster/Background
            LinearGradient(
                colors: movie.posterGradient.map { Color(hex: $0) },
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .frame(height: 400)

            // Icon
            Image(systemName: movie.posterSymbol)
                .font(.system(size: 120))
                .foregroundColor(.white.opacity(0.8))
                .frame(height: 400)

            // Gradient overlay
            LinearGradient(
                colors: [.clear, .clear, Color(.systemBackground)],
                startPoint: .top,
                endPoint: .bottom
            )
            .frame(height: 200)
        }
    }
}

struct GenresRow: View {
    let genres: [String]

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(genres, id: \.self) { genre in
                    let genreObj = Genre.allGenres.first { $0.name == genre } ?? Genre(id: "", name: genre)

                    HStack(spacing: 4) {
                        Image(systemName: genreObj.icon)
                        Text(genre)
                    }
                    .font(.subheadline)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 6)
                    .background(genreObj.color.opacity(0.2))
                    .foregroundColor(genreObj.color)
                    .cornerRadius(16)
                }
            }
        }
    }
}

struct DirectorRow: View {
    let director: String
    let directorURI: String
    @EnvironmentObject var movieService: MovieService

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(LinearGradient(
                        colors: [.blue, .purple],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    ))
                    .frame(width: 50, height: 50)

                Image(systemName: "person.fill")
                    .foregroundColor(.white)
                    .font(.title3)
            }

            VStack(alignment: .leading, spacing: 4) {
                Text("Director")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text(director)
                    .font(.headline)
            }

            Spacer()

            if let directorPerson = movieService.allDirectors.first(where: { $0.id == directorURI }) {
                NavigationLink(destination: PersonDetailView(person: directorPerson)) {
                    Image(systemName: "chevron.right")
                        .foregroundColor(.secondary)
                }
            }
        }
    }
}

struct CastSection: View {
    let cast: [String]
    let castURIs: [String]
    @EnvironmentObject var movieService: MovieService

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Cast")
                .font(.headline)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 16) {
                    ForEach(Array(zip(cast, castURIs)), id: \.0) { actor, uri in
                        CastCard(name: actor, uri: uri)
                    }
                }
            }
        }
    }
}

struct CastCard: View {
    let name: String
    let uri: String
    @EnvironmentObject var movieService: MovieService

    var body: some View {
        VStack(spacing: 8) {
            ZStack {
                Circle()
                    .fill(LinearGradient(
                        colors: [.orange, .pink],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    ))
                    .frame(width: 70, height: 70)

                Image(systemName: "person.fill")
                    .foregroundColor(.white)
                    .font(.title2)
            }

            Text(name)
                .font(.caption)
                .fontWeight(.semibold)
                .lineLimit(2)
                .multilineTextAlignment(.center)
                .frame(width: 70)
        }
        .onTapGesture {
            // Navigate to actor detail if available
            if let actor = movieService.allActors.first(where: { $0.id == uri }) {
                // Navigation handled by parent
            }
        }
    }
}

struct SimilarMoviesSection: View {
    let similarMovies: [Recommendation]
    let isLoading: Bool
    let onShowExplanation: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("You Might Also Like")
                    .font(.headline)

                Spacer()

                if !similarMovies.isEmpty {
                    Button(action: onShowExplanation) {
                        HStack(spacing: 4) {
                            Image(systemName: "lightbulb.fill")
                            Text("Why?")
                        }
                        .font(.caption)
                        .foregroundColor(.orange)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(Color.orange.opacity(0.1))
                        .cornerRadius(16)
                    }
                }
            }

            if isLoading {
                HStack {
                    Spacer()
                    ProgressView()
                    Spacer()
                }
                .frame(height: 200)
            } else if similarMovies.isEmpty {
                Text("No similar movies found")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.vertical, 40)
            } else {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 16) {
                        ForEach(similarMovies, id: \.movie.id) { recommendation in
                            SimilarMovieCard(recommendation: recommendation)
                        }
                    }
                }
            }
        }
    }
}

struct SimilarMovieCard: View {
    let recommendation: Recommendation

    var body: some View {
        NavigationLink(destination: MovieDetailView(movie: recommendation.movie)) {
            VStack(alignment: .leading, spacing: 8) {
                // Poster
                ZStack {
                    LinearGradient(
                        colors: recommendation.movie.posterGradient.map { Color(hex: $0) },
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                    .frame(width: 120, height: 170)
                    .cornerRadius(12)

                    Image(systemName: recommendation.movie.posterSymbol)
                        .font(.system(size: 40))
                        .foregroundColor(.white)

                    // Similarity badge
                    VStack {
                        Spacer()
                        HStack {
                            Spacer()
                            Text("\(Int(recommendation.score * 100))%")
                                .font(.caption2)
                                .fontWeight(.bold)
                                .foregroundColor(.white)
                                .padding(6)
                                .background(Color.green.opacity(0.8))
                                .clipShape(Circle())
                                .padding(6)
                        }
                    }
                    .frame(width: 120, height: 170)
                }

                // Title
                Text(recommendation.movie.title)
                    .font(.caption)
                    .fontWeight(.semibold)
                    .lineLimit(2)
                    .frame(width: 120, alignment: .leading)

                // Reason
                Text(recommendation.primaryReason)
                    .font(.caption2)
                    .foregroundColor(.secondary)
                    .lineLimit(1)
                    .frame(width: 120, alignment: .leading)

                // Rating
                HStack {
                    Image(systemName: "star.fill")
                        .font(.caption2)
                        .foregroundColor(.yellow)
                    Text(recommendation.movie.formattedRating)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
        }
        .buttonStyle(PlainButtonStyle())
    }
}

#Preview {
    NavigationView {
        MovieDetailView(movie: Movie.sample)
            .environmentObject(MovieService.shared)
    }
}
