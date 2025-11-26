//
// ComplianceCheckerApp.swift
// ComplianceChecker
//
// App entry point - uses Gonnect NanoGraphDB (Rust FFI)
//

import SwiftUI

@main
struct ComplianceCheckerApp: App {
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
