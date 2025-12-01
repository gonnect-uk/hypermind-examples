#!/bin/bash
# Quick npm publishing script for @gonnect/rust-kgdb
# This publishes ONLY compiled binaries, NO source code

set -e  # Exit on error

echo "🚀 Publishing @gonnect/rust-kgdb to npm"
echo "========================================"
echo ""

# Navigate to NAPI package
cd "$(dirname "$0")/../sdks/typescript/native/rust-kgdb-napi"

# Check if package.json exists
if [ ! -f "package.json" ]; then
    echo "❌ Error: package.json not found"
    exit 1
fi

# Get version
VERSION=$(grep '"version"' package.json | head -1 | sed 's/.*: "\(.*\)".*/\1/')
echo "📦 Package: @gonnect/rust-kgdb"
echo "📌 Version: $VERSION"
echo ""

# Step 1: Check npm login
echo "🔑 Checking npm authentication..."
if ! npm whoami > /dev/null 2>&1; then
    echo "❌ Not logged in to npm. Please run: npm login"
    exit 1
fi
NPMUSER=$(npm whoami)
echo "✅ Logged in as: $NPMUSER"
echo ""

# Step 2: Build release binary
echo "🔨 Building release binary..."
cd ../../../..  # Back to root
cargo build --release -p rust-kgdb-napi
echo "✅ Binary built"
echo ""

# Step 3: Copy binary to npm package
echo "📋 Copying binary to npm package..."
cd sdks/typescript/native/rust-kgdb-napi

# Detect platform and copy binary
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    ARCH=$(uname -m)
    if [[ "$ARCH" == "arm64" ]]; then
        BINARY_NAME="rust-kgdb-napi.darwin-arm64.node"
    else
        BINARY_NAME="rust-kgdb-napi.darwin-x64.node"
    fi
    cp ../../../../target/release/librust_kgdb_napi.dylib "$BINARY_NAME"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    BINARY_NAME="rust-kgdb-napi.linux-x64-gnu.node"
    cp ../../../../target/release/librust_kgdb_napi.so "$BINARY_NAME"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    # Windows
    BINARY_NAME="rust-kgdb-napi.win32-x64-msvc.node"
    cp ../../../../target/release/rust_kgdb_napi.dll "$BINARY_NAME"
else
    echo "❌ Unsupported platform: $OSTYPE"
    exit 1
fi

echo "✅ Binary copied: $BINARY_NAME"
echo ""

# Step 4: Verify package contents (dry run)
echo "🔍 Verifying package contents (dry run)..."
npm pack --dry-run

echo ""
echo "📦 Package will include:"
echo "  ✅ $BINARY_NAME (compiled binary)"
echo "  ✅ README.md (documentation)"
echo "  ✅ package.json (metadata)"
echo "  ✅ index.js (if exists)"
echo "  ✅ index.d.ts (if exists)"
echo "  ❌ NO src/ directory"
echo "  ❌ NO Cargo.toml"
echo "  ❌ NO Rust source code"
echo ""

# Step 5: Ask for confirmation
read -p "❓ Ready to publish to npm? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Publishing cancelled"
    exit 1
fi

# Step 6: Publish to npm
echo ""
echo "📤 Publishing to npm..."
npm publish --access public

# Step 7: Success!
echo ""
echo "✅ Successfully published @gonnect/rust-kgdb@$VERSION to npm!"
echo ""
echo "📚 Next steps:"
echo "  1. Test installation: npm install @gonnect/rust-kgdb"
echo "  2. Verify on npm: https://www.npmjs.com/package/@gonnect/rust-kgdb"
echo "  3. Check download stats: https://npmcharts.com/@gonnect/rust-kgdb"
echo ""
echo "🎉 Done!"
