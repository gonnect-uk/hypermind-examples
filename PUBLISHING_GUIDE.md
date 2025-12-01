# Publishing Guide - rust-kgdb v0.1.3

Complete guide to publishing rust-kgdb to crates.io, npm, and PyPI **without disclosing source code** but with comprehensive documentation.

## Overview

This guide covers publishing:
1. **Rust crate** to crates.io (source required by Cargo)
2. **TypeScript/Node.js** package to npm (binary-only distribution)
3. **Python** package to PyPI (binary-only distribution)

## Prerequisites

### Required Accounts
- [ ] crates.io account (GitHub login)
- [ ] npm account (`npm adduser`)
- [ ] PyPI account (`pip install twine`)

### Required Tools
```bash
# Install publishing tools
cargo install cargo-release
npm install -g @napi-rs/cli
pip install build twine
```

### Authentication Tokens
```bash
# Cargo
cargo login <YOUR_CRATES_IO_TOKEN>

# npm
npm login
# OR use token: npm set //registry.npmjs.org/:_authToken=<TOKEN>

# PyPI
# Create ~/.pypirc with your API token (see below)
```

---

## 1. Publishing to crates.io (Rust)

### Important Note
**crates.io REQUIRES source code** - it's designed for open-source distribution. If you want proprietary distribution, skip crates.io and use private registries.

### Steps for Open Source Publishing

```bash
# 1. Ensure Cargo.toml has correct metadata
cd /Users/gauravmalhotra/Documents/MyGithub/zenya-graphdb/rust-kgdb

# 2. Run tests
cargo test --workspace --exclude rust-kgdb-napi

# 3. Build release
cargo build --release --workspace

# 4. Check package contents
cargo package --list -p rust-kgdb-sdk

# 5. Dry run publish
cargo publish --dry-run -p rdf-model
cargo publish --dry-run -p storage
cargo publish --dry-run -p sparql
cargo publish --dry-run -p rust-kgdb-sdk

# 6. Publish (publish dependencies first!)
cargo publish -p rdf-model
sleep 30  # Wait for crates.io to index
cargo publish -p storage
sleep 30
cargo publish -p sparql
sleep 30
cargo publish -p rust-kgdb-sdk
```

### Alternative: Private Cargo Registry
Use [Cloudsmith](https://cloudsmith.io/), [JFrog Artifactory](https://jfrog.com/), or self-hosted registry for proprietary code.

---

## 2. Publishing to npm (TypeScript/Node.js) - BINARY ONLY

### Strategy: Publish Prebuilt Binaries Only

The npm package will contain:
- ✅ Compiled `.node` binaries
- ✅ TypeScript definitions (`index.d.ts`)
- ✅ JavaScript wrapper (`index.js`)
- ✅ README with documentation
- ✅ LICENSE
- ❌ NO Rust source code

### Build Setup

```bash
cd sdks/typescript/native/rust-kgdb-napi

# Install NAPI-RS CLI
npm install -g @napi-rs/cli

# Initialize NAPI project (already done)
# napi init
```

### Build for Multiple Platforms

```bash
# Build for current platform
cargo build --release -p rust-kgdb-napi

# Copy binary to npm package
cp ../../target/release/librust_kgdb_napi.dylib rust-kgdb-napi.darwin-arm64.node
# OR for Linux: .so file
# OR for Windows: .dll file

# For multi-platform, use GitHub Actions or Docker
# See: https://napi.rs/docs/cross-build
```

### Create Binary Distribution

```bash
# 1. Build index.js wrapper (NAPI auto-generates during build)
napi build --platform --release

# 2. Verify package contents
npm pack --dry-run

# You should see:
# - index.js
# - index.d.ts
# - README.md
# - *.node (binary)
# - package.json
# NO src/ or Cargo.toml!

# 3. Test locally
npm pack
npm install ./zenya-rust-kgdb-0.1.3.tgz

# 4. Publish to npm
npm publish --access public

# For scoped package (@gonnect/rust-kgdb):
npm publish --access public
```

### Multi-Platform Binaries (Recommended)

For professional deployment, build binaries for all platforms:

```bash
# Use GitHub Actions with napi-rs templates
# See: https://github.com/napi-rs/package-template

# Platforms to build:
# - darwin-x64 (macOS Intel)
# - darwin-arm64 (macOS Apple Silicon)
# - linux-x64-gnu
# - linux-arm64-gnu
# - win32-x64-msvc
```

Example GitHub Actions workflow:
```yaml
# .github/workflows/publish-npm.yml
name: Publish to npm
on:
  release:
    types: [created]

jobs:
  build:
    strategy:
      matrix:
        settings:
          - host: macos-latest
            target: x86_64-apple-darwin
          - host: macos-latest
            target: aarch64-apple-darwin
          - host: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - host: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.settings.host }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - name: Build
        run: |
          cd sdks/typescript/native/rust-kgdb-napi
          npm install
          npm run build -- --target ${{ matrix.settings.target }}
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: bindings-${{ matrix.settings.target }}
          path: sdks/typescript/native/rust-kgdb-napi/*.node

  publish:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/download-artifact@v3
      - name: Publish
        run: |
          cd sdks/typescript/native/rust-kgdb-napi
          npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

---

## 3. Publishing to PyPI (Python) - BINARY ONLY

### Strategy: Wheel Distribution with Binaries

The Python package will contain:
- ✅ Compiled `.so`/`.dylib`/`.pyd` binaries
- ✅ Python wrapper code
- ✅ README with documentation
- ✅ LICENSE
- ❌ NO Rust source code

### Build Wheels

```bash
cd sdks/python

# 1. Install build tools
pip install build wheel maturin

# 2. Build wheel for current platform
python -m build --wheel

# This creates dist/rust_kgdb-0.1.3-<platform>.whl

# 3. Verify wheel contents
unzip -l dist/rust_kgdb-0.1.3-*.whl

# You should see:
# - rust_kgdb/*.so (or .dylib/.pyd)
# - rust_kgdb/__init__.py
# - rust_kgdb-0.1.3.dist-info/
# NO Cargo.toml or Rust sources!
```

### Build for Multiple Platforms

Use `maturin` with Docker for multi-platform builds:

```bash
# Install maturin
pip install maturin

# Build for Linux (using Docker)
docker run --rm -v $(pwd):/io \
  quay.io/pypa/manylinux2014_x86_64 \
  bash -c "cd /io && maturin build --release --out dist"

# Build for macOS (on macOS)
maturin build --release --target universal2-apple-darwin

# Build for Windows (on Windows)
maturin build --release --target x86_64-pc-windows-msvc
```

### Test and Publish

```bash
# 1. Test wheel locally
pip install dist/rust_kgdb-0.1.3-*.whl
python -c "import rust_kgdb; print(rust_kgdb.__version__)"

# 2. Upload to TestPyPI (optional)
twine upload --repository testpypi dist/*

# 3. Test from TestPyPI
pip install --index-url https://test.pypi.org/simple/ rust-kgdb

# 4. Upload to PyPI (production)
twine upload dist/*
```

### PyPI Configuration (~/.pypirc)

```ini
[distutils]
index-servers =
    pypi
    testpypi

[pypi]
username = __token__
password = pypi-<YOUR_PYPI_API_TOKEN>

[testpypi]
repository = https://test.pypi.org/legacy/
username = __token__
password = pypi-<YOUR_TESTPYPI_API_TOKEN>
```

---

## Documentation Strategy (Public Without Source)

### What to Include in Packages

Each package should include:

#### 1. README.md (Comprehensive)
- Installation instructions
- Quick start guide
- API reference with examples
- Supported platforms
- Performance benchmarks
- Links to issue tracker

#### 2. API Documentation
- **Rust:** `cargo doc` (if publishing to crates.io)
- **TypeScript:** TypeScript definitions (.d.ts files)
- **Python:** Docstrings in wrapper code

#### 3. Examples Directory
```
examples/
├── basic_query.js          # TypeScript
├── basic_query.py          # Python
├── complex_sparql.js
├── complex_sparql.py
├── reasoning_example.js
└── reasoning_example.py
```

#### 4. Online Documentation
Host documentation on:
- GitHub Pages
- Read the Docs
- Your own website

Example structure:
```
docs/
├── getting-started.md
├── api-reference.md
├── examples.md
├── performance.md
└── troubleshooting.md
```

---

## Summary: Publishing Checklist

### Pre-Publishing (All Platforms)
- [ ] Update version in all `Cargo.toml`, `package.json`, `setup.py`
- [ ] Run full test suite: `make -f Makefile.sdk regression`
- [ ] Update CHANGELOG.md
- [ ] Create git tag: `git tag v0.1.3 && git push --tags`

### npm Publishing
```bash
cd sdks/typescript/native/rust-kgdb-napi
npm run build
npm pack --dry-run  # Verify contents
npm publish --access public
```

### PyPI Publishing
```bash
cd sdks/python
python -m build --wheel
twine check dist/*
twine upload dist/*
```

### Cargo Publishing (Optional - Open Source Only)
```bash
cargo publish -p rdf-model
cargo publish -p storage
cargo publish -p sparql
cargo publish -p rust-kgdb-sdk
```

---

## Post-Publishing

### 1. Verify Installation
```bash
# npm
npm install @gonnect/rust-kgdb
node -e "const {GraphDB} = require('@gonnect/rust-kgdb'); console.log('OK')"

# PyPI
pip install rust-kgdb
python -c "import rust_kgdb; print('OK')"

# Cargo
cargo new test-project && cd test-project
cargo add rust-kgdb-sdk
cargo build
```

### 2. Announce Release
- GitHub Releases with changelog
- Tweet/LinkedIn announcement
- Update project website
- Notify users via mailing list

### 3. Monitor Issues
- Watch GitHub issues
- Monitor npm download stats: https://npmcharts.com/@gonnect/rust-kgdb
- Monitor PyPI stats: https://pypistats.org/packages/rust-kgdb

---

## Security Considerations

### Binary Distribution Security
- Sign binaries with code signing certificates
- Provide checksums (SHA256) for all binaries
- Use HTTPS for all downloads
- Document build reproducibility

### Example Checksums File
```bash
# Generate checksums
cd dist
sha256sum * > SHA256SUMS.txt

# Users verify:
sha256sum -c SHA256SUMS.txt
```

---

## Support & Maintenance

### Version Updates
Use semantic versioning (SemVer):
- **Patch** (0.1.x): Bug fixes
- **Minor** (0.x.0): New features, backward compatible
- **Major** (x.0.0): Breaking changes

### Deprecation Policy
- Announce deprecations 1 minor version in advance
- Support last 2 major versions
- Maintain LTS branches for enterprise customers

---

## Contact & Support

- **Issues:** https://github.com/gonnect-uk/rust-kgdb/issues
- **Email:** support@gonnect.com
- **Documentation:** https://github.com/gonnect-uk/rust-kgdb#readme

---

**Version:** 0.1.3
**Last Updated:** November 30, 2025
**License:** Apache-2.0
