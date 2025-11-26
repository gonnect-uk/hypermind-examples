//! Template Definitions
//!
//! Contains embedded templates for code generation (Swift, Kotlin).

/// Swift templates (future: migrate to Tera)
pub mod swift {
    pub const APP_TEMPLATE: &str = r#"
// Generated Swift App
import SwiftUI

@main
struct GeneratedApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
"#;
}

/// Kotlin templates (future: migrate to Tera)
pub mod kotlin {
    pub const APP_TEMPLATE: &str = r#"
// Generated Kotlin App
package com.generated.app

import androidx.compose.runtime.Composable

@Composable
fun App() {
    // Generated Jetpack Compose UI
}
"#;
}
