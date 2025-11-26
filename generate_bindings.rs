// Simple binding generator using uniffi_bindgen library
use std::path::PathBuf;

fn main() {
    let udl_file = PathBuf::from("crates/mobile-ffi/src/gonnect.udl");
    let out_dir = PathBuf::from("ios/Generated");

    uniffi_bindgen::generate_bindings(
        &udl_file,
        None,
        vec!["swift".to_string()],
        &out_dir,
        false,
    ).expect("Failed to generate bindings");

    println!("Swift bindings generated successfully!");
}
