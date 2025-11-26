//
//  Person.swift
//  SmartSearchRecommender
//
//  Person model for actors and directors
//

import Foundation

enum PersonRole: String, Codable {
    case actor = "Actor"
    case director = "Director"
    case producer = "Producer"
    case screenwriter = "Screenwriter"
}

struct Person: Identifiable, Codable, Hashable {
    let id: String // URI of the person
    let name: String
    let birthDate: Date?
    let roles: [PersonRole]
    var filmography: [String] // Movie URIs

    // Computed properties
    var age: Int? {
        guard let birthDate = birthDate else { return nil }
        let calendar = Calendar.current
        let ageComponents = calendar.dateComponents([.year], from: birthDate, to: Date())
        return ageComponents.year
    }

    var birthYear: String {
        guard let date = birthDate else { return "Unknown" }
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy"
        return formatter.string(from: date)
    }

    var rolesString: String {
        roles.map { $0.rawValue }.joined(separator: ", ")
    }

    var primaryRole: PersonRole {
        roles.first ?? .actor
    }

    var roleIcon: String {
        switch primaryRole {
        case .actor:
            return "person.fill"
        case .director:
            return "video.fill"
        case .producer:
            return "star.fill"
        case .screenwriter:
            return "doc.text.fill"
        }
    }

    // Initialize from SPARQL query results
    init(
        id: String,
        name: String,
        birthDate: Date? = nil,
        roles: [PersonRole] = [],
        filmography: [String] = []
    ) {
        self.id = id
        self.name = name
        self.birthDate = birthDate
        self.roles = roles
        self.filmography = filmography
    }

    // Hash for Hashable conformance
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    // Equatable conformance
    static func == (lhs: Person, rhs: Person) -> Bool {
        lhs.id == rhs.id
    }
}

// Extension for sample data
extension Person {
    static let samplePeople: [Person] = [
        Person(
            id: "http://dbpedia.org/resource/Christopher_Nolan",
            name: "Christopher Nolan",
            birthDate: ISO8601DateFormatter().date(from: "1970-07-30T00:00:00Z"),
            roles: [.director, .screenwriter, .producer],
            filmography: [
                "http://dbpedia.org/resource/Inception",
                "http://dbpedia.org/resource/The_Dark_Knight",
                "http://dbpedia.org/resource/Interstellar",
                "http://dbpedia.org/resource/The_Prestige"
            ]
        ),
        Person(
            id: "http://dbpedia.org/resource/Leonardo_DiCaprio",
            name: "Leonardo DiCaprio",
            birthDate: ISO8601DateFormatter().date(from: "1974-11-11T00:00:00Z"),
            roles: [.actor, .producer],
            filmography: [
                "http://dbpedia.org/resource/Inception",
                "http://dbpedia.org/resource/The_Wolf_of_Wall_Street",
                "http://dbpedia.org/resource/Shutter_Island"
            ]
        ),
        Person(
            id: "http://dbpedia.org/resource/Quentin_Tarantino",
            name: "Quentin Tarantino",
            birthDate: ISO8601DateFormatter().date(from: "1963-03-27T00:00:00Z"),
            roles: [.director, .screenwriter, .actor],
            filmography: [
                "http://dbpedia.org/resource/Pulp_Fiction",
                "http://dbpedia.org/resource/Django_Unchained"
            ]
        )
    ]

    static var sample: Person {
        samplePeople[0]
    }
}
