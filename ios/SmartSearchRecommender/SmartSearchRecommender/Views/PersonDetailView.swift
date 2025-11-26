//
//  PersonDetailView.swift
//  SmartSearchRecommender
//
//  Actor/Director detail view with filmography
//

import SwiftUI

struct PersonDetailView: View {
    let person: Person
    @EnvironmentObject var movieService: MovieService
    @State private var filmography: [Movie] = []

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                // Header
                VStack(spacing: 16) {
                    // Avatar
                    ZStack {
                        Circle()
                            .fill(LinearGradient(
                                colors: [.blue, .purple, .pink],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            ))
                            .frame(width: 150, height: 150)

                        Image(systemName: person.roleIcon)
                            .font(.system(size: 70))
                            .foregroundColor(.white)
                    }

                    // Name
                    Text(person.name)
                        .font(.title)
                        .fontWeight(.bold)

                    // Roles
                    HStack(spacing: 8) {
                        ForEach(person.roles, id: \.self) { role in
                            Text(role.rawValue)
                                .font(.subheadline)
                                .padding(.horizontal, 12)
                                .padding(.vertical, 6)
                                .background(Color.blue.opacity(0.2))
                                .foregroundColor(.blue)
                                .cornerRadius(16)
                        }
                    }

                    // Birth info
                    if let age = person.age {
                        Text("Born \(person.birthYear) · Age \(age)")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 20)

                Divider()

                // Filmography
                VStack(alignment: .leading, spacing: 16) {
                    HStack {
                        Image(systemName: "film.stack")
                            .foregroundColor(.orange)
                        Text("Filmography")
                            .font(.title2)
                            .fontWeight(.bold)
                        Spacer()
                        Text("\(filmography.count) movies")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal)

                    if filmography.isEmpty {
                        VStack(spacing: 12) {
                            Image(systemName: "film")
                                .font(.system(size: 50))
                                .foregroundColor(.secondary)
                            Text("No movies found")
                                .font(.subheadline)
                                .foregroundColor(.secondary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 40)
                    } else {
                        LazyVGrid(columns: [
                            GridItem(.adaptive(minimum: 160), spacing: 16)
                        ], spacing: 16) {
                            ForEach(filmography) { movie in
                                NavigationLink(destination: MovieDetailView(movie: movie)) {
                                    MovieCard(movie: movie)
                                }
                            }
                        }
                        .padding(.horizontal)
                    }
                }

                // Bio section (placeholder)
                VStack(alignment: .leading, spacing: 12) {
                    Text("About")
                        .font(.title2)
                        .fontWeight(.bold)

                    Text("Known for their work in \(person.roles.map { $0.rawValue.lowercased() }.joined(separator: " and ")) with \(filmography.count) notable films.")
                        .font(.body)
                        .foregroundColor(.secondary)
                }
                .padding(.horizontal)
            }
            .padding(.vertical)
        }
        .navigationBarTitleDisplayMode(.inline)
        .task {
            loadFilmography()
        }
    }

    private func loadFilmography() {
        // Load movies featuring this person
        filmography = movieService.allMovies.filter { movie in
            // Check if person is director
            if person.roles.contains(.director) && movie.directorURI == person.id {
                return true
            }
            // Check if person is in cast
            if person.roles.contains(.actor) && movie.castURIs.contains(person.id) {
                return true
            }
            return false
        }.sorted { $0.rating > $1.rating }
    }
}

#Preview {
    NavigationView {
        PersonDetailView(person: Person.sample)
            .environmentObject(MovieService.shared)
    }
}
