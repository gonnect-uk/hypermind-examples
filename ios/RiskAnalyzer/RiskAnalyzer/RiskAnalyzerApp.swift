//
// RiskAnalyzerApp.swift
// RiskAnalyzer
//
// App entry point - uses Gonnect NanoGraphDB (Rust FFI)
//

import SwiftUI

@main
struct RiskAnalyzerApp: App {
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
