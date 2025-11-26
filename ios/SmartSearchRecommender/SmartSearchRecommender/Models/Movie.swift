//
//  Movie.swift
//  SmartSearchRecommender
//
//  Movie model representing RDF film entities
//

import Foundation

struct Movie: Identifiable, Codable, Hashable {
    let id: String // URI of the movie
    let title: String
    let director: String
    let directorURI: String
    let cast: [String]
    let castURIs: [String]
    let genres: [String]
    let rating: Double
    let releaseDate: Date?
    let description: String

    // Computed properties
    var formattedRating: String {
        String(format: "%.1f", rating)
    }

    var ratingColor: String {
        switch rating {
        case 8.5...10.0: return "green"
        case 7.0..<8.5: return "yellow"
        default: return "orange"
        }
    }

    var releaseYear: String {
        guard let date = releaseDate else { return "Unknown" }
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy"
        return formatter.string(from: date)
    }

    var genresString: String {
        genres.joined(separator: " • ")
    }

    var posterSymbol: String {
        // Map genres to SF Symbols
        if genres.contains("Science Fiction") {
            return "sparkles"
        } else if genres.contains("Action") {
            return "bolt.fill"
        } else if genres.contains("Crime") {
            return "exclamationmark.triangle.fill"
        } else if genres.contains("Thriller") {
            return "exclamationmark.bubble.fill"
        } else if genres.contains("Drama") {
            return "theatermasks.fill"
        } else if genres.contains("Mystery") {
            return "questionmark.circle.fill"
        } else if genres.contains("Comedy") {
            return "face.smiling.fill"
        } else {
            return "film.fill"
        }
    }

    var posterGradient: [String] {
        // Map genres to color gradients
        if genres.contains("Science Fiction") {
            return ["#667eea", "#764ba2"]
        } else if genres.contains("Action") {
            return ["#f12711", "#f5af19"]
        } else if genres.contains("Crime") {
            return ["#000000", "#434343"]
        } else if genres.contains("Thriller") {
            return ["#4b134f", "#c94b4b"]
        } else if genres.contains("Drama") {
            return ["#2193b0", "#6dd5ed"]
        } else if genres.contains("Mystery") {
            return ["#360033", "#0b8793"]
        } else if genres.contains("Comedy") {
            return ["#f7971e", "#ffd200"]
        } else {
            return ["#8e2de2", "#4a00e0"]
        }
    }

    // Initialize from SPARQL query results
    init(
        id: String,
        title: String,
        director: String,
        directorURI: String,
        cast: [String] = [],
        castURIs: [String] = [],
        genres: [String] = [],
        rating: Double = 0.0,
        releaseDate: Date? = nil,
        description: String = ""
    ) {
        self.id = id
        self.title = title
        self.director = director
        self.directorURI = directorURI
        self.cast = cast
        self.castURIs = castURIs
        self.genres = genres
        self.rating = rating
        self.releaseDate = releaseDate
        self.description = description
    }

    // Hash for Hashable conformance
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    // Equatable conformance
    static func == (lhs: Movie, rhs: Movie) -> Bool {
        lhs.id == rhs.id
    }
}

// Extension for sample data (development/testing)
extension Movie {
    static let sampleMovies: [Movie] = [
        Movie(
            id: "http://dbpedia.org/resource/Inception",
            title: "Inception",
            director: "Christopher Nolan",
            directorURI: "http://dbpedia.org/resource/Christopher_Nolan",
            cast: ["Leonardo DiCaprio", "Joseph Gordon-Levitt", "Ellen Page"],
            castURIs: [
                "http://dbpedia.org/resource/Leonardo_DiCaprio",
                "http://dbpedia.org/resource/Joseph_Gordon-Levitt",
                "http://dbpedia.org/resource/Ellen_Page"
            ],
            genres: ["Science Fiction", "Thriller"],
            rating: 8.8,
            releaseDate: ISO8601DateFormatter().date(from: "2010-07-16T00:00:00Z"),
            description: "A thief who steals corporate secrets through dream-sharing technology"
        ),
        Movie(
            id: "http://dbpedia.org/resource/The_Dark_Knight",
            title: "The Dark Knight",
            director: "Christopher Nolan",
            directorURI: "http://dbpedia.org/resource/Christopher_Nolan",
            cast: ["Christian Bale", "Heath Ledger", "Aaron Eckhart"],
            castURIs: [
                "http://dbpedia.org/resource/Christian_Bale",
                "http://dbpedia.org/resource/Heath_Ledger",
                "http://dbpedia.org/resource/Aaron_Eckhart"
            ],
            genres: ["Action", "Crime", "Drama"],
            rating: 9.0,
            releaseDate: ISO8601DateFormatter().date(from: "2008-07-18T00:00:00Z"),
            description: "Batman faces the Joker in a battle for Gotham's soul"
        ),
        Movie(
            id: "http://dbpedia.org/resource/Pulp_Fiction",
            title: "Pulp Fiction",
            director: "Quentin Tarantino",
            directorURI: "http://dbpedia.org/resource/Quentin_Tarantino",
            cast: ["John Travolta", "Samuel L. Jackson", "Uma Thurman"],
            castURIs: [
                "http://dbpedia.org/resource/John_Travolta",
                "http://dbpedia.org/resource/Samuel_L_Jackson",
                "http://dbpedia.org/resource/Uma_Thurman"
            ],
            genres: ["Crime", "Drama"],
            rating: 8.9,
            releaseDate: ISO8601DateFormatter().date(from: "1994-10-14T00:00:00Z"),
            description: "Interconnected stories of Los Angeles criminals"
        )
    ]

    static var sample: Movie {
        sampleMovies[0]
    }
}
