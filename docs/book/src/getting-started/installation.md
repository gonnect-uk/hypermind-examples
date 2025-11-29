# Installation Guide

This guide covers all installation methods and configuration options for rust-kgdb.

## System Requirements

- **OS**: Linux, macOS, Windows
- **Rust**: 1.70 or later
- **Memory**: Minimum 512 MB (recommended 2+ GB for large datasets)
- **Disk**: 100 MB for compilation and dependencies

## Installation Methods

### 1. Cargo (Recommended)

Add rust-kgdb to your project's `Cargo.toml`:

```toml
[dependencies]
rust-kgdb = "0.1"
```

### 2. With All Features

To use all storage backends (InMemory, RocksDB, LMDB):

```toml
[dependencies]
rust-kgdb = { version = "0.1", features = ["all-backends"] }
```

### 3. Specific Backends Only

**InMemory only** (default, smallest binary):
```toml
rust-kgdb = "0.1"
```

**With RocksDB**:
```toml
rust-kgdb = { version = "0.1", features = ["rocksdb-backend"] }
```

**With LMDB**:
```toml
rust-kgdb = { version = "0.1", features = ["lmdb-backend"] }
```

### 4. From Source

Clone the repository and build locally:

```bash
git clone https://github.com/zenya-graphdb/rust-kgdb.git
cd rust-kgdb
cargo build --release
```

Build time: approximately 5-6 minutes on modern hardware.

## Platform-Specific Notes

### macOS
All features work on both Intel and Apple Silicon. Install Xcode command-line tools:

```bash
xcode-select --install
```

### Linux
Install build essentials:

```bash
# Debian/Ubuntu
sudo apt-get install build-essential pkg-config

# Fedora/RHEL
sudo dnf install gcc pkg-config
```

### Windows
Install Visual Studio Build Tools with C++ workload, or use the Rust installer's recommended VS Community setup.

## Verify Installation

Create a test file to verify your setup:

```rust
use rust_kgdb::rdf_model::Dictionary;

fn main() {
    let dict = Dictionary::new();
    println!("rust-kgdb installed successfully!");
}
```

Run it:
```bash
cargo run
```

## iOS/Android Development

For mobile development, see [Mobile Development](../sdk/rust/index.md#mobile).

## Troubleshooting

**Build fails with "linker error"**: Install build tools (see platform-specific notes above)

**Tests fail**: Run `cargo test --workspace` to ensure all dependencies are correct

**Performance is slow**: Use `--release` flag: `cargo build --release`

## Next Steps

- [Quick Start](./quick-start.md) - Run your first program in 5 minutes
- [First Steps](./first-steps.md) - Interactive tutorial
- [Core Concepts](./core-concepts.md) - Understand RDF fundamentals
