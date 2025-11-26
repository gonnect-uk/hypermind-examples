//
//  MovieService.swift
//  SmartSearchRecommender
//
//  Service for querying movies using SPARQL via Rust KGDB FFI
//

import Foundation
import Combine

@MainActor
class MovieService: ObservableObject {
    static let shared = MovieService()

    // Published properties
    @Published var allMovies: [Movie] = []
    @Published var favoriteMovies: [Movie] = []
    @Published var allActors: [Person] = []
    @Published var allDirectors: [Person] = []
    @Published var allGenres: [Genre] = []
    @Published var isLoading = false
    @Published var errorMessage: String?
    @Published var tripleCount: Int = 0

    // FFI database handle (placeholder - will be replaced with actual FFI)
    private var dbHandle: UInt64 = 0

    // Cache
    private var movieCache: [String: Movie] = [:]
    private var personCache: [String: Person] = [:]

    private init() {
        // Initialize with sample data for development
        self.allMovies = Movie.sampleMovies
        self.allActors = Person.samplePeople.filter { $0.roles.contains(.actor) }
        self.allDirectors = Person.samplePeople.filter { $0.roles.contains(.director) }
        self.allGenres = Genre.allGenres
    }

    // MARK: - Database Initialization

    func loadMoviesCatalog() async {
        isLoading = true
        errorMessage = nil

        do {
            // In production: Use FFI to load TTL file
            // let catalogPath = Bundle.main.path(forResource: "movies_catalog", ofType: "ttl")!
            // dbHandle = try await loadTTLFile(catalogPath)

            // For now, simulate loading with delay
            try await Task.sleep(nanoseconds: 1_000_000_000)

            // Execute SPARQL queries to populate data
            await loadAllMovies()
            await loadAllPeople()
            await loadAllGenres()
            await updateTripleCount()

            isLoading = false
        } catch {
            isLoading = false
            errorMessage = "Failed to load movies catalog: \(error.localizedDescription)"
        }
    }

    // MARK: - SPARQL Query Execution

    private func executeSPARQL(_ query: String) async throws -> [[String: String]] {
        // In production: Use FFI to execute SPARQL
        // let resultJSON = try await kgdb_query(dbHandle, query)
        // return parseSPARQLResults(resultJSON)

        // For development: Return sample data based on query pattern
        if query.contains("dbo:Film") {
            return sampleMovieResults()
        } else if query.contains("dbo:Person") {
            return samplePersonResults()
        } else if query.contains("dbo:Genre") {
            return sampleGenreResults()
        }
        return []
    }

    // MARK: - Load Data Methods

    private func loadAllMovies() async {
        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX dbr: <http://dbpedia.org/resource/>
        PREFIX schema: <http://schema.org/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX dc: <http://purl.org/dc/elements/1.1/>

        SELECT ?movie ?title ?director ?directorName ?rating ?date ?description
        WHERE {
          ?movie a dbo:Film ;
                 rdfs:label ?title ;
                 dbo:director ?director ;
                 schema:aggregateRating ?rating .
          ?director rdfs:label ?directorName .
          OPTIONAL { ?movie schema:datePublished ?date }
          OPTIONAL { ?movie dc:description ?description }
        }
        ORDER BY DESC(?rating)
        """

        do {
            let results = try await executeSPARQL(query)
            var movies: [Movie] = []

            for result in results {
                if let movieURI = result["movie"],
                   let title = result["title"],
                   let director = result["directorName"],
                   let directorURI = result["director"],
                   let ratingStr = result["rating"],
                   let rating = Double(ratingStr) {

                    // Fetch additional details
                    let cast = await loadCast(for: movieURI)
                    let genres = await loadGenres(for: movieURI)

                    let movie = Movie(
                        id: movieURI,
                        title: title,
                        director: director,
                        directorURI: directorURI,
                        cast: cast.map { $0.name },
                        castURIs: cast.map { $0.id },
                        genres: genres.map { $0.name },
                        rating: rating,
                        releaseDate: parseDate(result["date"]),
                        description: result["description"] ?? ""
                    )

                    movies.append(movie)
                    movieCache[movieURI] = movie
                }
            }

            allMovies = movies
        } catch {
            errorMessage = "Failed to load movies: \(error.localizedDescription)"
        }
    }

    private func loadCast(for movieURI: String) async -> [Person] {
        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?actor ?actorName
        WHERE {
          <\(movieURI)> dbo:starring ?actor .
          ?actor rdfs:label ?actorName .
        }
        """

        do {
            let results = try await executeSPARQL(query)
            return results.compactMap { result in
                guard let actorURI = result["actor"],
                      let actorName = result["actorName"] else { return nil }

                if let cached = personCache[actorURI] {
                    return cached
                }

                let person = Person(id: actorURI, name: actorName, roles: [.actor])
                personCache[actorURI] = person
                return person
            }
        } catch {
            return []
        }
    }

    private func loadGenres(for movieURI: String) async -> [Genre] {
        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?genre ?genreName
        WHERE {
          <\(movieURI)> dbo:genre ?genre .
          ?genre rdfs:label ?genreName .
        }
        """

        do {
            let results = try await executeSPARQL(query)
            return results.compactMap { result in
                guard let genreURI = result["genre"],
                      let genreName = result["genreName"] else { return nil }
                return Genre(id: genreURI, name: genreName)
            }
        } catch {
            return []
        }
    }

    private func loadAllPeople() async {
        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT DISTINCT ?person ?name ?birthDate ?occupation
        WHERE {
          ?person a dbo:Person ;
                  rdfs:label ?name .
          OPTIONAL { ?person dbo:birthDate ?birthDate }
          OPTIONAL { ?person dbo:occupation ?occupation }
        }
        """

        do {
            let results = try await executeSPARQL(query)
            var actors: [Person] = []
            var directors: [Person] = []

            for result in results {
                if let personURI = result["person"],
                   let name = result["name"] {

                    let roles = parseRoles(result["occupation"])
                    let person = Person(
                        id: personURI,
                        name: name,
                        birthDate: parseDate(result["birthDate"]),
                        roles: roles
                    )

                    personCache[personURI] = person

                    if roles.contains(.actor) {
                        actors.append(person)
                    }
                    if roles.contains(.director) {
                        directors.append(person)
                    }
                }
            }

            allActors = actors
            allDirectors = directors
        } catch {
            errorMessage = "Failed to load people: \(error.localizedDescription)"
        }
    }

    private func loadAllGenres() async {
        let query = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?genre ?genreName (COUNT(?movie) as ?count)
        WHERE {
          ?genre a dbo:Genre ;
                 rdfs:label ?genreName .
          OPTIONAL { ?movie dbo:genre ?genre }
        }
        GROUP BY ?genre ?genreName
        ORDER BY DESC(?count)
        """

        do {
            let results = try await executeSPARQL(query)
            allGenres = results.compactMap { result in
                guard let genreURI = result["genre"],
                      let genreName = result["genreName"] else { return nil }

                let count = Int(result["count"] ?? "0") ?? 0
                return Genre(id: genreURI, name: genreName, movieCount: count)
            }
        } catch {
            errorMessage = "Failed to load genres: \(error.localizedDescription)"
        }
    }

    private func updateTripleCount() async {
        let query = """
        SELECT (COUNT(*) as ?count)
        WHERE {
          ?s ?p ?o .
        }
        """

        do {
            let results = try await executeSPARQL(query)
            if let countStr = results.first?["count"],
               let count = Int(countStr) {
                tripleCount = count
            } else {
                // Estimate from sample data
                tripleCount = 89
            }
        } catch {
            tripleCount = 89
        }
    }

    // MARK: - Search Methods

    func searchMovies(query: String) async -> [Movie] {
        guard !query.isEmpty else { return allMovies }

        let sparqlQuery = """
        PREFIX dbo: <http://dbpedia.org/ontology/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX dc: <http://purl.org/dc/elements/1.1/>

        SELECT ?movie ?title ?rating
        WHERE {
          ?movie a dbo:Film ;
                 rdfs:label ?title ;
                 schema:aggregateRating ?rating .
          FILTER(CONTAINS(LCASE(?title), LCASE("\(query)")) ||
                 CONTAINS(LCASE(?description), LCASE("\(query)")))
        }
        ORDER BY DESC(?rating)
        LIMIT 20
        """

        do {
            let results = try await executeSPARQL(sparqlQuery)
            return results.compactMap { result in
                guard let movieURI = result["movie"] else { return nil }
                return movieCache[movieURI]
            }
        } catch {
            // Fallback to local search
            return allMovies.filter { movie in
                movie.title.localizedCaseInsensitiveContains(query) ||
                movie.description.localizedCaseInsensitiveContains(query) ||
                movie.director.localizedCaseInsensitiveContains(query)
            }
        }
    }

    func filterMovies(by filter: MovieFilter) -> [Movie] {
        let filtered = allMovies.filter { filter.matches($0) }
        return filter.sort(filtered)
    }

    // MARK: - Favorites Management

    func toggleFavorite(_ movie: Movie) {
        if let index = favoriteMovies.firstIndex(of: movie) {
            favoriteMovies.remove(at: index)
        } else {
            favoriteMovies.append(movie)
        }
    }

    func isFavorite(_ movie: Movie) -> Bool {
        favoriteMovies.contains(movie)
    }

    // MARK: - Utility Methods

    private func parseDate(_ dateString: String?) -> Date? {
        guard let dateString = dateString else { return nil }
        let formatter = ISO8601DateFormatter()
        return formatter.date(from: dateString)
    }

    private func parseRoles(_ occupationString: String?) -> [PersonRole] {
        guard let occupationString = occupationString else { return [.actor] }

        var roles: [PersonRole] = []
        if occupationString.contains("Director") {
            roles.append(.director)
        }
        if occupationString.contains("Actor") {
            roles.append(.actor)
        }
        if occupationString.contains("Producer") {
            roles.append(.producer)
        }
        if occupationString.contains("Screenwriter") {
            roles.append(.screenwriter)
        }

        return roles.isEmpty ? [.actor] : roles
    }

    // MARK: - Sample Data (for development)

    private func sampleMovieResults() -> [[String: String]] {
        return Movie.sampleMovies.map { movie in
            [
                "movie": movie.id,
                "title": movie.title,
                "director": movie.directorURI,
                "directorName": movie.director,
                "rating": String(movie.rating),
                "description": movie.description
            ]
        }
    }

    private func samplePersonResults() -> [[String: String]] {
        return Person.samplePeople.map { person in
            [
                "person": person.id,
                "name": person.name,
                "occupation": person.rolesString
            ]
        }
    }

    private func sampleGenreResults() -> [[String: String]] {
        return Genre.allGenres.map { genre in
            [
                "genre": genre.id,
                "genreName": genre.name,
                "count": String(genre.movieCount)
            ]
        }
    }
}
