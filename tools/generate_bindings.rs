#!/usr/bin/env rustc
//! UniFFI Swift Bindings Generator
//! Standalone tool to generate Swift bindings from UDL files for iOS projects

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Get arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <udl_file> <language> <out_dir>", args[0]);
        eprintln!("Example: {} crates/mobile-ffi/src/gonnect.udl swift ios/Generated", args[0]);
        std::process::exit(1);
    }

    let udl_file = &args[1];
    let language = &args[2];
    let out_dir = &args[3];

    println!("Generating {} bindings from {}", language, udl_file);
    println!("Output directory: {}", out_dir);

    // Use cargo to build a temporary binary that generates bindings
    // This works around the lack of a standalone uniffi-bindgen binary in 0.30

    let status = Command::new("cargo")
        .args(&[
            "run",
            "--package", "mobile-ffi",
            "--bin", "uniffi-bindgen-tool",
            "--",
            udl_file,
            language,
            out_dir
        ])
        .status()
        .expect("Failed to execute cargo run");

    if status.success() {
        println!("✓ Swift bindings generated successfully");
    } else {
        eprintln!("✗ Swift binding generation failed");
        std::process::exit(1);
    }
}
