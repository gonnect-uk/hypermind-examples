//
//  Genre.swift
//  SmartSearchRecommender
//
//  Genre model and filtering
//

import Foundation
import SwiftUI

struct Genre: Identifiable, Codable, Hashable {
    let id: String // URI of the genre
    let name: String
    var movieCount: Int = 0

    // Genre-specific styling
    var icon: String {
        switch name {
        case "Science Fiction":
            return "sparkles"
        case "Action":
            return "bolt.fill"
        case "Crime":
            return "exclamationmark.triangle.fill"
        case "Thriller":
            return "exclamationmark.bubble.fill"
        case "Drama":
            return "theatermasks.fill"
        case "Mystery":
            return "questionmark.circle.fill"
        case "Comedy":
            return "face.smiling.fill"
        case "Biography":
            return "person.text.rectangle.fill"
        case "Western":
            return "figure.equestrian.sports"
        default:
            return "film.fill"
        }
    }

    var color: Color {
        switch name {
        case "Science Fiction":
            return .purple
        case "Action":
            return .orange
        case "Crime":
            return .black
        case "Thriller":
            return .red
        case "Drama":
            return .blue
        case "Mystery":
            return .indigo
        case "Comedy":
            return .yellow
        case "Biography":
            return .green
        case "Western":
            return .brown
        default:
            return .gray
        }
    }

    // Initialize from SPARQL query results
    init(id: String, name: String, movieCount: Int = 0) {
        self.id = id
        self.name = name
        self.movieCount = movieCount
    }

    // Hash for Hashable conformance
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    // Equatable conformance
    static func == (lhs: Genre, rhs: Genre) -> Bool {
        lhs.id == rhs.id
    }
}

// Extension for sample data
extension Genre {
    static let allGenres: [Genre] = [
        Genre(id: "http://dbpedia.org/resource/Science_Fiction", name: "Science Fiction", movieCount: 3),
        Genre(id: "http://dbpedia.org/resource/Action", name: "Action", movieCount: 1),
        Genre(id: "http://dbpedia.org/resource/Crime", name: "Crime", movieCount: 4),
        Genre(id: "http://dbpedia.org/resource/Thriller", name: "Thriller", movieCount: 5),
        Genre(id: "http://dbpedia.org/resource/Drama", name: "Drama", movieCount: 6),
        Genre(id: "http://dbpedia.org/resource/Mystery", name: "Mystery", movieCount: 3),
        Genre(id: "http://dbpedia.org/resource/Comedy", name: "Comedy", movieCount: 1),
        Genre(id: "http://dbpedia.org/resource/Biography", name: "Biography", movieCount: 1),
        Genre(id: "http://dbpedia.org/resource/Western", name: "Western", movieCount: 1)
    ]

    static var sample: Genre {
        allGenres[0]
    }
}

// Filter configuration for movies
struct MovieFilter {
    var selectedGenres: Set<String> = []
    var minRating: Double = 0.0
    var sortBy: SortOption = .rating

    enum SortOption: String, CaseIterable {
        case rating = "Rating"
        case title = "Title"
        case releaseDate = "Release Date"
        case director = "Director"
    }

    var isActive: Bool {
        !selectedGenres.isEmpty || minRating > 0.0
    }

    mutating func reset() {
        selectedGenres.removeAll()
        minRating = 0.0
        sortBy = .rating
    }

    func matches(_ movie: Movie) -> Bool {
        let genreMatch = selectedGenres.isEmpty || !selectedGenres.isDisjoint(with: Set(movie.genres))
        let ratingMatch = movie.rating >= minRating
        return genreMatch && ratingMatch
    }

    func sort(_ movies: [Movie]) -> [Movie] {
        switch sortBy {
        case .rating:
            return movies.sorted { $0.rating > $1.rating }
        case .title:
            return movies.sorted { $0.title < $1.title }
        case .releaseDate:
            return movies.sorted {
                ($0.releaseDate ?? Date.distantPast) > ($1.releaseDate ?? Date.distantPast)
            }
        case .director:
            return movies.sorted { $0.director < $1.director }
        }
    }
}
