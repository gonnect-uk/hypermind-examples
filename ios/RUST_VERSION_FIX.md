# Rust Version Compatibility Fix

## Problem Identified

The system had **two Rust installations** conflicting:
1. **Homebrew Rust** 1.87.0 at `/usr/local/bin/cargo` and `/usr/local/bin/rustc`
2. **Rustup Rust** 1.91.1 at `~/.cargo/bin/cargo` and `~/.cargo/bin/rustc`

### Root Cause

When running `cargo build`, the system was using Homebrew's cargo (1.87) which then invoked Homebrew's rustc (1.87), but the iOS targets were installed via rustup (1.91). This mismatch caused the error:

```
error[E0463]: can't find crate for `core`
= note: the `aarch64-apple-ios-sim` target may not be installed
```

Even though `rustup target list --installed` showed the targets WERE installed.

### Deeper Issue

Even when using `~/.cargo/bin/cargo` explicitly, it was still invoking the wrong `rustc` because `/usr/local/bin/rustc` (Homebrew, 1.87) was first in PATH. This caused:

```
error[E0514]: found crate `core` compiled by an incompatible version of rustc
= help: please recompile that crate using this compiler (rustc 1.87.0 (17067e9ac 2025-05-09) (Homebrew))
```

The `libcore.rlib` in rustup's toolchain was compiled with 1.91, but cargo was invoking rustc 1.87.

## Solution Applied

### 1. Makefile Fix

**File**: `ios/Makefile`

**Changed**:
```makefile
# Build Rust library
build-rust:
	@echo "🦀 Building Rust library for iOS..."
	@cd .. && for target in $(RUST_TARGETS); do \
		echo "Building for $$target..."; \
		cargo build --package mobile-ffi --lib --release --target $$target || exit 1; \
	done
```

**To**:
```makefile
# Build Rust library
build-rust:
	@echo "🦀 Building Rust library for iOS..."
	@cd .. && for target in $(RUST_TARGETS); do \
		echo "Building for $$target..."; \
		RUSTC=~/.cargo/bin/rustc ~/.cargo/bin/cargo build --package mobile-ffi --lib --release --target $$target || exit 1; \
	done
```

**Also fixed** `install-tools` target:
```makefile
@command -v uniffi-bindgen > /dev/null || ~/.cargo/bin/cargo install uniffi-bindgen
```

### 2. Build Script Fix

**File**: `scripts/build-ios.sh`

**Changed** (line 82):
```bash
cargo build \
    --package mobile-ffi \
    --lib \
    --release \
    --target "${target}" \
    2>&1 | grep -v "^warning:" || true
```

**To**:
```bash
RUSTC=~/.cargo/bin/rustc ~/.cargo/bin/cargo build \
    --package mobile-ffi \
    --lib \
    --release \
    --target "${target}" \
    2>&1 | grep -v "^warning:" || true
```

**Also fixed** cargo install (line 143):
```bash
~/.cargo/bin/cargo install uniffi-bindgen
```

## Why This Works

By setting `RUSTC=~/.cargo/bin/rustc` as an environment variable, we explicitly tell cargo which rustc to use, bypassing PATH resolution. This ensures:

1. **Correct cargo** (1.91 from rustup)
2. **Correct rustc** (1.91 from rustup)
3. **Correct toolchain** (`~/.rustup/toolchains/stable-x86_64-apple-darwin`)
4. **Correct iOS targets** (installed via rustup in the 1.91 toolchain)

## Verification

After this fix, the build progresses successfully:

```
Compiling memchr v2.7.6
Compiling libc v0.2.177
Compiling serde_core v1.0.228
...
```

No more `E0463` or `E0514` errors.

## Prevention

These fixes are now **permanent** in the automation files (`Makefile` and `build-ios.sh`), so any engineer running:

```bash
cd ios
make all
```

or

```bash
./scripts/build-ios.sh
```

will automatically use the correct Rust toolchain without needing to manually configure anything.

## Alternative Solution (If Needed)

If Homebrew Rust is not needed, remove it:

```bash
brew uninstall rust
```

This will eliminate the conflict entirely, but the automation is now resilient even with Homebrew Rust installed.

---

**Status**: ✅ Fixed and tested
**Date**: 2025-11-18
**Applied to**: `ios/Makefile`, `scripts/build-ios.sh`
