// UniFFI Bindgen CLI Tool for uniffi 0.30+
// Professional-grade binding generator using latest uniffi library API
//
// uniffi_bindgen 0.30+ removed the official CLI, so we build our own
// This is the PROPER professional approach using LATEST version

use camino::Utf8PathBuf;
use std::env;
use std::process;
use uniffi_bindgen::bindings::{SwiftBindingGenerator, KotlinBindingGenerator};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 7 || args[1] != "generate" || args[3] != "--language" || args[5] != "--out-dir" {
        eprintln!("Usage: uniffi-bindgen generate <udl_file> --language <language> --out-dir <directory>");
        eprintln!("Example: uniffi-bindgen generate crates/mobile-ffi/src/gonnect.udl --language swift --out-dir ios/Generated");
        process::exit(1);
    }

    // Parse command-line arguments - uniffi 0.30 requires Utf8PathBuf (camino crate)
    let udl_file = Utf8PathBuf::from(&args[2]);
    let language = &args[4];
    let out_dir = Utf8PathBuf::from(&args[6]);

    println!("🔧 UniFFI Bindgen 0.30 - Professional Binding Generator");
    println!("   UDL File: {}", udl_file);
    println!("   Language: {}", language);
    println!("   Output: {}", out_dir);
    println!();

    // Ensure output directory exists
    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| {
            eprintln!("✗ Failed to create output directory: {}", e);
            process::exit(1);
        });
    }

    // Ensure UDL file exists
    if !udl_file.exists() {
        eprintln!("✗ UDL file not found: {}", udl_file);
        process::exit(1);
    }

    // uniffi_bindgen 0.30 API signature:
    // pub fn generate_bindings<T: BindingGenerator>(
    //     udl_file: &Utf8Path,
    //     config_file_override: Option<&Utf8Path>,
    //     binding_generator: T,
    //     out_dir_override: Option<&Utf8Path>,
    //     library_file: Option<&Utf8Path>,
    //     crate_name: Option<&str>,
    //     try_format_code: bool,
    // ) -> Result<()>

    let result = match language.as_str() {
        "swift" => {
            println!("Generating Swift bindings...");
            uniffi_bindgen::generate_bindings(
                &udl_file,                      // udl_file: &Utf8Path
                None,                           // config_file_override: Option<&Utf8Path>
                SwiftBindingGenerator,          // binding_generator: SwiftBindingGenerator
                Some(&out_dir),                 // out_dir_override: Option<&Utf8Path>
                None,                           // library_file: Option<&Utf8Path>
                None,                           // crate_name: Option<&str>
                false                           // try_format_code: bool
            )
        }
        "kotlin" => {
            println!("Generating Kotlin bindings...");
            uniffi_bindgen::generate_bindings(
                &udl_file,
                None,
                KotlinBindingGenerator,
                Some(&out_dir),
                None,
                None,
                false
            )
        }
        _ => {
            eprintln!("✗ Unsupported language: {}", language);
            eprintln!("   Supported: swift, kotlin");
            process::exit(1);
        }
    };

    match result {
        Ok(_) => {
            println!("✓ {} bindings generated successfully", language);
            println!("  Files created in {}", out_dir);
        }
        Err(e) => {
            eprintln!("✗ Binding generation failed: {}", e);
            process::exit(1);
        }
    }
}
