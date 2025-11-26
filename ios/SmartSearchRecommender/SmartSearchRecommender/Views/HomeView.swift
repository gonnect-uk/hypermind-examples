//
//  HomeView.swift
//  SmartSearchRecommender
//
//  Main discovery view with search, filters, and recommendations
//

import SwiftUI

struct HomeView: View {
    @EnvironmentObject var movieService: MovieService
    @StateObject private var recommendationEngine: RecommendationEngineWrapper
    @State private var searchText = ""
    @State private var showFilters = false
    @State private var filter = MovieFilter()
    @State private var recommendations: [Recommendation] = []

    init() {
        _recommendationEngine = StateObject(wrappedValue: RecommendationEngineWrapper())
    }

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 24) {
                    // Search Bar
                    SearchBar(text: $searchText, onSearch: performSearch)
                        .padding(.horizontal)

                    // Filter Chips
                    if filter.isActive {
                        FilterChipsView(filter: $filter)
                            .padding(.horizontal)
                    }

                    // Hero Section - Featured Movie
                    if let featured = movieService.allMovies.first {
                        FeaturedMovieCard(movie: featured)
                            .padding(.horizontal)
                    }

                    // Recommendations Carousel
                    if !recommendations.isEmpty {
                        RecommendationCarousel(
                            title: "Recommended for You",
                            recommendations: recommendations
                        )
                    }

                    // Top Rated Section
                    MovieSection(
                        title: "Top Rated",
                        icon: "star.fill",
                        movies: topRatedMovies
                    )

                    // By Genre Sections
                    ForEach(movieService.allGenres.prefix(3)) { genre in
                        MovieSection(
                            title: genre.name,
                            icon: genre.icon,
                            movies: moviesForGenre(genre.name)
                        )
                    }

                    // Directors Section
                    DirectorsSection()
                        .padding(.horizontal)
                }
                .padding(.vertical)
            }
            .navigationTitle("Discover")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { showFilters.toggle() }) {
                        Image(systemName: "slider.horizontal.3")
                            .foregroundColor(.primary)
                    }
                }
            }
            .sheet(isPresented: $showFilters) {
                FilterSheet(filter: $filter)
            }
            .task {
                await loadRecommendations()
            }
        }
    }

    // MARK: - Computed Properties

    private var topRatedMovies: [Movie] {
        movieService.allMovies
            .sorted { $0.rating > $1.rating }
            .prefix(10)
            .map { $0 }
    }

    private func moviesForGenre(_ genre: String) -> [Movie] {
        movieService.allMovies
            .filter { $0.genres.contains(genre) }
            .sorted { $0.rating > $1.rating }
            .prefix(10)
            .map { $0 }
    }

    // MARK: - Methods

    private func performSearch() {
        // Search is handled by SearchResultsView
    }

    private func loadRecommendations() async {
        recommendationEngine.setMovieService(movieService)
        recommendations = await recommendationEngine.engine.getExploreRecommendations()
    }
}

// MARK: - Supporting Views

struct SearchBar: View {
    @Binding var text: String
    var onSearch: () -> Void

    var body: some View {
        HStack {
            Image(systemName: "magnifyingglass")
                .foregroundColor(.secondary)

            TextField("Search movies, actors, directors...", text: $text)
                .textFieldStyle(PlainTextFieldStyle())
                .onSubmit(onSearch)

            if !text.isEmpty {
                Button(action: { text = "" }) {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(12)
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}

struct FeaturedMovieCard: View {
    let movie: Movie

    var body: some View {
        NavigationLink(destination: MovieDetailView(movie: movie)) {
            ZStack(alignment: .bottomLeading) {
                // Gradient background based on genre
                LinearGradient(
                    colors: movie.posterGradient.map { Color(hex: $0) },
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
                .frame(height: 300)
                .cornerRadius(20)

                // Overlay content
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Image(systemName: movie.posterSymbol)
                            .font(.system(size: 60))
                            .foregroundColor(.white)

                        Spacer()

                        VStack(alignment: .trailing) {
                            Image(systemName: "star.fill")
                                .foregroundColor(.yellow)
                            Text(movie.formattedRating)
                                .font(.title2)
                                .fontWeight(.bold)
                                .foregroundColor(.white)
                        }
                    }

                    Spacer()

                    Text(movie.title)
                        .font(.title)
                        .fontWeight(.bold)
                        .foregroundColor(.white)

                    Text("Directed by \(movie.director)")
                        .font(.subheadline)
                        .foregroundColor(.white.opacity(0.9))

                    HStack {
                        ForEach(movie.genres.prefix(2), id: \.self) { genre in
                            Text(genre)
                                .font(.caption)
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(Color.white.opacity(0.3))
                                .cornerRadius(8)
                                .foregroundColor(.white)
                        }
                    }
                }
                .padding(20)
            }
        }
    }
}

struct RecommendationCarousel: View {
    let title: String
    let recommendations: [Recommendation]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "sparkles")
                    .foregroundColor(.orange)
                Text(title)
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
            }
            .padding(.horizontal)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 16) {
                    ForEach(recommendations, id: \.movie.id) { recommendation in
                        RecommendationCard(recommendation: recommendation)
                    }
                }
                .padding(.horizontal)
            }
        }
    }
}

struct RecommendationCard: View {
    let recommendation: Recommendation
    @State private var showExplanation = false

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
                    .frame(width: 160, height: 220)
                    .cornerRadius(12)

                    Image(systemName: recommendation.movie.posterSymbol)
                        .font(.system(size: 60))
                        .foregroundColor(.white)
                }

                // Title
                Text(recommendation.movie.title)
                    .font(.headline)
                    .lineLimit(2)
                    .frame(width: 160, alignment: .leading)

                // Reason
                HStack(spacing: 4) {
                    Image(systemName: "lightbulb.fill")
                        .font(.caption2)
                        .foregroundColor(.orange)
                    Text(recommendation.primaryReason)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }
                .frame(width: 160, alignment: .leading)

                // Rating
                HStack {
                    Image(systemName: "star.fill")
                        .font(.caption)
                        .foregroundColor(.yellow)
                    Text(recommendation.movie.formattedRating)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .buttonStyle(PlainButtonStyle())
    }
}

struct MovieSection: View {
    let title: String
    let icon: String
    let movies: [Movie]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: icon)
                    .foregroundColor(.blue)
                Text(title)
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
            }
            .padding(.horizontal)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 16) {
                    ForEach(movies) { movie in
                        MovieCard(movie: movie)
                    }
                }
                .padding(.horizontal)
            }
        }
    }
}

struct MovieCard: View {
    let movie: Movie
    @EnvironmentObject var movieService: MovieService

    var body: some View {
        NavigationLink(destination: MovieDetailView(movie: movie)) {
            VStack(alignment: .leading, spacing: 8) {
                // Poster
                ZStack(alignment: .topTrailing) {
                    LinearGradient(
                        colors: movie.posterGradient.map { Color(hex: $0) },
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                    .frame(width: 140, height: 200)
                    .cornerRadius(12)

                    Image(systemName: movie.posterSymbol)
                        .font(.system(size: 50))
                        .foregroundColor(.white)
                        .frame(width: 140, height: 200)

                    // Favorite button
                    Button(action: { movieService.toggleFavorite(movie) }) {
                        Image(systemName: movieService.isFavorite(movie) ? "heart.fill" : "heart")
                            .foregroundColor(movieService.isFavorite(movie) ? .red : .white)
                            .padding(8)
                            .background(Color.black.opacity(0.5))
                            .clipShape(Circle())
                    }
                    .padding(8)
                }

                // Title
                Text(movie.title)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .lineLimit(2)
                    .frame(width: 140, alignment: .leading)

                // Rating
                HStack {
                    Image(systemName: "star.fill")
                        .font(.caption)
                        .foregroundColor(.yellow)
                    Text(movie.formattedRating)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .buttonStyle(PlainButtonStyle())
    }
}

struct DirectorsSection: View {
    @EnvironmentObject var movieService: MovieService

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "video.fill")
                    .foregroundColor(.purple)
                Text("Featured Directors")
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
            }

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 16) {
                    ForEach(movieService.allDirectors.prefix(5)) { director in
                        DirectorCard(director: director)
                    }
                }
            }
        }
    }
}

struct DirectorCard: View {
    let director: Person

    var body: some View {
        NavigationLink(destination: PersonDetailView(person: director)) {
            VStack(spacing: 8) {
                ZStack {
                    Circle()
                        .fill(LinearGradient(
                            colors: [.blue, .purple],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        ))
                        .frame(width: 80, height: 80)

                    Image(systemName: "person.fill")
                        .font(.system(size: 35))
                        .foregroundColor(.white)
                }

                Text(director.name)
                    .font(.caption)
                    .fontWeight(.semibold)
                    .lineLimit(2)
                    .multilineTextAlignment(.center)
                    .frame(width: 80)
            }
        }
        .buttonStyle(PlainButtonStyle())
    }
}

struct FilterChipsView: View {
    @Binding var filter: MovieFilter

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                if !filter.selectedGenres.isEmpty {
                    ForEach(Array(filter.selectedGenres), id: \.self) { genre in
                        FilterChip(text: genre) {
                            filter.selectedGenres.remove(genre)
                        }
                    }
                }

                if filter.minRating > 0 {
                    FilterChip(text: "Rating > \(String(format: "%.1f", filter.minRating))") {
                        filter.minRating = 0.0
                    }
                }

                Button(action: { filter.reset() }) {
                    Text("Clear All")
                        .font(.caption)
                        .foregroundColor(.red)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(Color.red.opacity(0.1))
                        .cornerRadius(16)
                }
            }
        }
    }
}

struct FilterChip: View {
    let text: String
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: 4) {
            Text(text)
                .font(.caption)
                .foregroundColor(.white)

            Button(action: onRemove) {
                Image(systemName: "xmark.circle.fill")
                    .font(.caption)
                    .foregroundColor(.white.opacity(0.8))
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color.blue)
        .cornerRadius(16)
    }
}

struct FilterSheet: View {
    @Environment(\.dismiss) var dismiss
    @Binding var filter: MovieFilter
    @EnvironmentObject var movieService: MovieService

    var body: some View {
        NavigationView {
            Form {
                Section("Genres") {
                    ForEach(movieService.allGenres) { genre in
                        Toggle(isOn: Binding(
                            get: { filter.selectedGenres.contains(genre.name) },
                            set: { isSelected in
                                if isSelected {
                                    filter.selectedGenres.insert(genre.name)
                                } else {
                                    filter.selectedGenres.remove(genre.name)
                                }
                            }
                        )) {
                            HStack {
                                Image(systemName: genre.icon)
                                    .foregroundColor(genre.color)
                                Text(genre.name)
                            }
                        }
                    }
                }

                Section("Rating") {
                    VStack(alignment: .leading) {
                        Text("Minimum Rating: \(String(format: "%.1f", filter.minRating))")
                            .font(.subheadline)
                        Slider(value: $filter.minRating, in: 0...10, step: 0.5)
                    }
                }

                Section("Sort By") {
                    Picker("Sort By", selection: $filter.sortBy) {
                        ForEach(MovieFilter.SortOption.allCases, id: \.self) { option in
                            Text(option.rawValue).tag(option)
                        }
                    }
                    .pickerStyle(SegmentedPickerStyle())
                }
            }
            .navigationTitle("Filters")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Reset") {
                        filter.reset()
                    }
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

// MARK: - Helper Classes

@MainActor
class RecommendationEngineWrapper: ObservableObject {
    private var movieService: MovieService?
    var engine: RecommendationEngine {
        RecommendationEngine(movieService: movieService ?? MovieService.shared)
    }

    func setMovieService(_ service: MovieService) {
        self.movieService = service
    }
}

// MARK: - Color Extension

extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 3: // RGB (12-bit)
            (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6: // RGB (24-bit)
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8: // ARGB (32-bit)
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (255, 0, 0, 0)
        }
        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue: Double(b) / 255,
            opacity: Double(a) / 255
        )
    }
}

#Preview {
    HomeView()
        .environmentObject(MovieService.shared)
}
