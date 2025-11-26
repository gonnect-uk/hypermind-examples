//
// ProductFinderApp.swift
// ProductFinder
//
// App entry point - uses Gonnect NanoGraphDB (Rust FFI)
//

import SwiftUI

@main
struct ProductFinderApp: App {
    init() {
        initializeLogging()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
