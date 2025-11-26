//
// GraphDBAdminApp.swift
// GraphDBAdmin
//
// App entry point - uses Gonnect NanoGraphDB (Rust FFI)
//

import SwiftUI

@main
struct GraphDBAdminApp: App {
    init() {
        // Initialize Rust logging for debugging
        initializeLogging()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
