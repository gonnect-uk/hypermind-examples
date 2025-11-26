//
//  RecommendationEngine.swift
//  SmartSearchRecommender
//
//  Graph-based recommendation engine using SPARQL path queries
//

import Foundation

struct RecommendationPath {
    let reason: String
    let confidence: Double
    let nodes: [String] // URIs in the path
    let edges: [String] // Predicates
}

struct Recommendation {
    let movie: Movie
    let score: Double
    let paths: [RecommendationPath]
    let primaryReason: String
}

class RecommendationEngine {
    private let movieService: MovieService

    init(movieService: MovieService) {
        self.movieService = movieService
    }

    // MARK: - Main Recommendation Methods

    /// Find similar movies using graph traversal
    func findSimilarMovies(to movie: Movie, limit: Int = 10) async -> [Recommendation] {
        var recommendations: [Recommendation] = []

        // Strategy 1: Same director
        let sameDirector = await findMoviesBySameDirector(movie)
        recommendations.append(contentsOf: sameDirector)

        // Strategy 2: Shared cast
        let sharedCast = await findMoviesBySharedCast(movie)
        recommendations.append(contentsOf: sharedCast)

        // Strategy 3: Similar genres
        let similarGenres = await findMoviesBySimilarGenres(movie)
        recommendations.append(contentsOf: similarGenres)

        // Strategy 4: High-rated in same genre
        let topRated = await findTopRatedInGenres(movie.genres)
        recommendations.append(contentsOf: topRated)

        // Deduplicate and sort by score
        return deduplicateAndScore(recommendations, excluding: movie)
            .prefix(limit)
            .map { $0 }
    }

    /// Discover movies based on user preferences
    func discoverMovies(
        favoriteGenres: [String] = [],
        favoriteDirectors: [String] = [],
        minRating: Double = 7.5
    ) async -> [Recommendation] {
        var recommendations: [Recommendation] = []

        // Find highly-rated movies in favorite genres
        if !favoriteGenres.isEmpty {
            let genreMovies = await findTopRatedInGenres(favoriteGenres, minRating: minRating)
            recommendations.append(contentsOf: genreMovies)
        }

        // Find movies by favorite directors
        if !favoriteDirectors.isEmpty {
            for director in favoriteDirectors {
                let directorMovies = await findMoviesByDirectorName(director)
                recommendations.append(contentsOf: directorMovies)
            }
        }

        return deduplicateAndScore(recommendations, excluding: nil)
            .prefix(10)
            .map { $0 }
    }

    /// Find movies for "Explore" carousel
    func getExploreRecommendations() async -> [Recommendation] {
        // Return diverse set of high-rated movies
        let allMovies = await movieService.allMovies
        let topRated = allMovies
            .filter { $0.rating >= 8.0 }
            .sorted { $0.rating > $1.rating }
            .prefix(10)

        return topRated.map { movie in
            Recommendation(
                movie: movie,
                score: movie.rating / 10.0,
                paths: [
                    RecommendationPath(
                        reason: "Highly rated movie",
                        confidence: movie.rating / 10.0,
                        nodes: [movie.id],
                        edges: []
                    )
                ],
                primaryReason: "Top rated with \(movie.formattedRating) stars"
            )
        }
    }

    // MARK: - SPARQL-based Recommendation Strategies

    private func findMoviesBySameDirector(_ movie: Movie) async -> [Recommendation] {
        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?movie ?title ?rating
        WHERE {
          ?movie a dbo:Film ;
                 dbo:director <\(movie.directorURI)> ;
                 rdfs:label ?title ;
                 schema:aggregateRating ?rating .
          FILTER(?movie != <\(movie.id)>)
        }
        ORDER BY DESC(?rating)
        LIMIT 5
        """

        // In production: Execute SPARQL query
        // For now, use local filtering
        let similarMovies = await movieService.allMovies.filter { m in
            m.directorURI == movie.directorURI && m.id != movie.id
        }

        return similarMovies.map { similar in
            Recommendation(
                movie: similar,
                score: 0.8,
                paths: [
                    RecommendationPath(
                        reason: "Same director",
                        confidence: 0.8,
                        nodes: [movie.id, movie.directorURI, similar.id],
                        edges: ["dbo:director", "dbo:director"]
                    )
                ],
                primaryReason: "Also directed by \(movie.director)"
            )
        }
    }

    private func findMoviesBySharedCast(_ movie: Movie) async -> [Recommendation] {
        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?movie ?title ?rating ?actor ?actorName (COUNT(?actor) as ?sharedCount)
        WHERE {
          <\(movie.id)> dbo:starring ?actor .
          ?movie dbo:starring ?actor ;
                 a dbo:Film ;
                 rdfs:label ?title ;
                 schema:aggregateRating ?rating .
          ?actor rdfs:label ?actorName .
          FILTER(?movie != <\(movie.id)>)
        }
        GROUP BY ?movie ?title ?rating ?actor ?actorName
        ORDER BY DESC(?sharedCount) DESC(?rating)
        LIMIT 5
        """

        // Local implementation
        let movieCastSet = Set(movie.castURIs)
        let similarMovies = await movieService.allMovies.filter { m in
            m.id != movie.id && !Set(m.castURIs).isDisjoint(with: movieCastSet)
        }

        return similarMovies.map { similar in
            let sharedActors = Set(similar.castURIs).intersection(movieCastSet)
            let sharedNames = similar.cast.filter { name in
                movie.cast.contains(name)
            }

            return Recommendation(
                movie: similar,
                score: 0.7,
                paths: [
                    RecommendationPath(
                        reason: "Shared cast",
                        confidence: 0.7,
                        nodes: [movie.id] + Array(sharedActors) + [similar.id],
                        edges: Array(repeating: "dbo:starring", count: sharedActors.count * 2)
                    )
                ],
                primaryReason: "Stars \(sharedNames.first ?? "shared actor")"
            )
        }
    }

    private func findMoviesBySimilarGenres(_ movie: Movie) async -> [Recommendation] {
        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?movie ?title ?rating ?genre (COUNT(?genre) as ?sharedGenres)
        WHERE {
          {
            SELECT ?genre WHERE {
              <\(movie.id)> dbo:genre ?genre .
            }
          }
          ?movie dbo:genre ?genre ;
                 a dbo:Film ;
                 rdfs:label ?title ;
                 schema:aggregateRating ?rating .
          FILTER(?movie != <\(movie.id)>)
        }
        GROUP BY ?movie ?title ?rating ?genre
        HAVING (COUNT(?genre) >= 1)
        ORDER BY DESC(?sharedGenres) DESC(?rating)
        LIMIT 10
        """

        // Local implementation
        let movieGenreSet = Set(movie.genres)
        let similarMovies = await movieService.allMovies.filter { m in
            m.id != movie.id && !Set(m.genres).isDisjoint(with: movieGenreSet)
        }

        return similarMovies.map { similar in
            let sharedGenres = Set(similar.genres).intersection(movieGenreSet)

            return Recommendation(
                movie: similar,
                score: 0.6,
                paths: [
                    RecommendationPath(
                        reason: "Similar genres",
                        confidence: 0.6,
                        nodes: [movie.id, similar.id],
                        edges: ["dbo:genre"]
                    )
                ],
                primaryReason: "Similar genre: \(sharedGenres.first ?? "")"
            )
        }
    }

    private func findTopRatedInGenres(_ genres: [String], minRating: Double = 8.0) async -> [Recommendation] {
        guard !genres.isEmpty else { return [] }

        let genreFilter = genres.map { "dbo:\($0.replacingOccurrences(of: " ", with: "_"))" }.joined(separator: " ")

        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?movie ?title ?rating
        WHERE {
          ?movie a dbo:Film ;
                 dbo:genre ?genre ;
                 rdfs:label ?title ;
                 schema:aggregateRating ?rating .
          FILTER(?rating >= \(minRating))
          FILTER(?genre IN (\(genreFilter)))
        }
        ORDER BY DESC(?rating)
        LIMIT 10
        """

        // Local implementation
        let topMovies = await movieService.allMovies.filter { m in
            m.rating >= minRating && !Set(m.genres).isDisjoint(with: Set(genres))
        }.sorted { $0.rating > $1.rating }

        return topMovies.map { topMovie in
            Recommendation(
                movie: topMovie,
                score: topMovie.rating / 10.0,
                paths: [
                    RecommendationPath(
                        reason: "Top rated in genre",
                        confidence: topMovie.rating / 10.0,
                        nodes: [topMovie.id],
                        edges: []
                    )
                ],
                primaryReason: "Top rated \(topMovie.genres.first ?? "movie")"
            )
        }
    }

    private func findMoviesByDirectorName(_ directorName: String) async -> [Recommendation] {
        let movies = await movieService.allMovies.filter { $0.director == directorName }

        return movies.map { movie in
            Recommendation(
                movie: movie,
                score: 0.7,
                paths: [
                    RecommendationPath(
                        reason: "Directed by favorite director",
                        confidence: 0.7,
                        nodes: [movie.directorURI, movie.id],
                        edges: ["dbo:director"]
                    )
                ],
                primaryReason: "Directed by \(directorName)"
            )
        }
    }

    // MARK: - Scoring and Deduplication

    private func deduplicateAndScore(
        _ recommendations: [Recommendation],
        excluding: Movie?
    ) -> [Recommendation] {
        var movieScores: [String: (movie: Movie, totalScore: Double, paths: [RecommendationPath])] = [:]

        for rec in recommendations {
            // Skip excluded movie
            if let excluded = excluding, rec.movie.id == excluded.id {
                continue
            }

            if let existing = movieScores[rec.movie.id] {
                // Combine scores and paths
                movieScores[rec.movie.id] = (
                    movie: rec.movie,
                    totalScore: existing.totalScore + rec.score,
                    paths: existing.paths + rec.paths
                )
            } else {
                movieScores[rec.movie.id] = (
                    movie: rec.movie,
                    totalScore: rec.score,
                    paths: rec.paths
                )
            }
        }

        // Convert back to recommendations
        return movieScores.values.map { entry in
            let primaryReason = entry.paths.max(by: { $0.confidence < $1.confidence })?.reason
                ?? "Recommended for you"

            return Recommendation(
                movie: entry.movie,
                score: entry.totalScore,
                paths: entry.paths,
                primaryReason: primaryReason
            )
        }.sorted { $0.score > $1.score }
    }

    // MARK: - Graph Path Explanation

    func explainRecommendation(_ recommendation: Recommendation) -> String {
        var explanation = "Why we recommend \"\(recommendation.movie.title)\":\n\n"

        for (index, path) in recommendation.paths.enumerated() {
            explanation += "\(index + 1). \(path.reason) (confidence: \(Int(path.confidence * 100))%)\n"
        }

        return explanation
    }

    func formatGraphPath(_ path: RecommendationPath) -> String {
        var result = ""
        for (index, node) in path.nodes.enumerated() {
            result += node.split(separator: "/").last.map(String.init) ?? node
            if index < path.edges.count {
                result += " → \(path.edges[index]) → "
            }
        }
        return result
    }
}
