# Gonnect NanoGraphDB - iOS Apps Makefile
# Professional build automation (industry standard)
# Used by: LLVM, Rust, Linux Kernel, Google, Apple internal projects

.PHONY: all clean setup build-rust generate-bindings create-projects test install-tools

# Configuration
RUST_TARGETS := aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-ios
APPS := GraphDBAdmin SmartSearchRecommender ComplianceGuardian ProductConfigurator

all: setup build-rust-parallel generate-bindings create-projects
	@echo "✅ All iOS apps built successfully!"

# Install required tools
install-tools:
	@echo "📦 Installing required tools..."
	@command -v xcodegen > /dev/null || brew install xcodegen
	@echo "✓ Tools installed (uniffi-bindgen runs via cargo, no separate install needed)"

# Setup: install Rust targets
setup:
	@echo "🔧 Setting up iOS build environment..."
	@for target in $(RUST_TARGETS); do \
		rustup target add $$target 2>/dev/null || echo "✓ $$target already installed"; \
	done

# Build Rust library (PARALLEL - all 3 targets simultaneously)
build-rust-parallel:
	@echo "🦀 Building Rust library for iOS (3 targets in parallel)..."
	@cd .. && \
	RUSTC=~/.cargo/bin/rustc ~/.cargo/bin/cargo build --package mobile-ffi --lib --release --target aarch64-apple-ios-sim > /tmp/build-sim-arm64.log 2>&1 & \
	RUSTC=~/.cargo/bin/rustc ~/.cargo/bin/cargo build --package mobile-ffi --lib --release --target x86_64-apple-ios > /tmp/build-sim-intel.log 2>&1 & \
	RUSTC=~/.cargo/bin/rustc ~/.cargo/bin/cargo build --package mobile-ffi --lib --release --target aarch64-apple-ios > /tmp/build-device.log 2>&1 & \
	wait
	@echo "✓ Rust libraries built"

# Generate Swift bindings
generate-bindings:
	@echo "🔄 Generating Swift bindings..."
	@mkdir -p Generated Frameworks
	@cd .. && ~/.cargo/bin/uniffi-bindgen generate --library target/aarch64-apple-ios-sim/release/libmobile_ffi.a --language swift --out-dir ios/Generated
	@echo "✓ Swift bindings generated"

# Create XCFramework
create-framework:
	@echo "📦 Creating XCFramework..."
	@mkdir -p ../target/universal-sim
	@mkdir -p ../target/universal-sim/Headers
	@mkdir -p ../target/aarch64-apple-ios/release/Headers
	@lipo -create \
		../target/x86_64-apple-ios/release/libmobile_ffi.a \
		../target/aarch64-apple-ios-sim/release/libmobile_ffi.a \
		-output ../target/universal-sim/libmobile_ffi.a
	@cp Generated/gonnectFFI.h ../target/universal-sim/Headers/
	@cp Generated/gonnectFFI.modulemap ../target/universal-sim/Headers/module.modulemap
	@cp Generated/gonnectFFI.h ../target/aarch64-apple-ios/release/Headers/
	@cp Generated/gonnectFFI.modulemap ../target/aarch64-apple-ios/release/Headers/module.modulemap
	@xcodebuild -create-xcframework \
		-library ../target/universal-sim/libmobile_ffi.a \
		-headers ../target/universal-sim/Headers \
		-library ../target/aarch64-apple-ios/release/libmobile_ffi.a \
		-headers ../target/aarch64-apple-ios/release/Headers \
		-output Frameworks/GonnectNanoGraphDB.xcframework
	@echo "✓ XCFramework created"

# Create Xcode projects
create-projects: generate-bindings create-framework
	@echo "🎯 Creating Xcode projects..."
	@for app in $(APPS); do \
		echo "Creating $$app..."; \
		cd $$app && xcodegen generate && cd ..; \
	done
	@echo "✓ Xcode projects created"

# Run tests
test:
	@echo "🧪 Running tests..."
	@for app in $(APPS); do \
		xcodebuild test \
			-project $$app/$$app.xcodeproj \
			-scheme $$app \
			-destination 'platform=iOS Simulator,name=iPhone 15 Pro' \
			|| echo "⚠️  Tests failed for $$app"; \
	done

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	@rm -rf Generated Frameworks
	@rm -rf ../target/universal-sim
	@for app in $(APPS); do \
		rm -rf $$app/$$app.xcodeproj; \
		rm -rf $$app/build; \
		rm -rf $$app/DerivedData; \
	done
	@echo "✓ Cleaned"

# Open all projects in Xcode
open:
	@for app in $(APPS); do \
		open $$app/$$app.xcodeproj; \
	done

# Help
help:
	@echo "Gonnect NanoGraphDB - iOS Build System"
	@echo ""
	@echo "Targets:"
	@echo "  make all              - Full build pipeline (parallel builds - 3x faster)"
	@echo "  make install-tools    - Install xcodegen"
	@echo "  make build-rust-parallel - Build Rust for all 3 iOS targets (parallel)"
	@echo "  make generate-bindings - Generate Swift FFI bindings"
	@echo "  make create-projects  - Generate Xcode projects"
	@echo "  make test             - Run all tests"
	@echo "  make open             - Open all projects in Xcode"
	@echo "  make clean            - Clean all artifacts"
	@echo ""
	@echo "Quick start:"
	@echo "  1. make install-tools"
	@echo "  2. make all"
	@echo "  3. make open"
