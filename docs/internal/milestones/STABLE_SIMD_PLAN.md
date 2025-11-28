# Stable Rust SIMD Implementation Plan

## Problem

Current SIMD implementation uses `std::simd` which requires:
- Rust nightly toolchain
- `#![feature(portable_simd)]` unstable feature
- Not suitable for production stable Rust builds

## Solution: packed_simd_2 Crate

Use `packed_simd_2` which:
- ✅ Works on **stable Rust**
- ✅ Same API as `std::simd` (easy migration)
- ✅ Supports x86_64 (AVX2) and ARM (NEON)
- ✅ Production-ready (used by many crates)

### Migration Plan

#### 1. Update Cargo.toml

```toml
[dependencies]
# Remove std::simd, add packed_simd_2
packed_simd_2 = { version = "0.3", optional = true }

[features]
simd = ["packed_simd_2"]
```

#### 2. Update Code (Minimal Changes)

**Before** (unstable std::simd):
```rust
use std::simd::{u8x16, u64x4, Simd, SimdPartialEq};
```

**After** (stable packed_simd_2):
```rust
use packed_simd_2::{u8x16, u64x4, FromCast, Simd};
```

**Changes Required**:
- Replace `std::simd::` → `packed_simd_2::`
- Replace `.simd_eq()` → `.eq()`
- Replace `.to_array()` → `.into()` or `.as_array()`
- Minor API differences (well-documented)

#### 3. Build on Stable Rust

```bash
# Works on stable Rust!
cargo build --features simd
cargo test --features simd
cargo bench --features simd
```

### Performance Expectations

**Same performance** as std::simd:
- Both use LLVM intrinsics
- Both compile to same assembly (AVX2/NEON)
- No runtime overhead

### Timeline

**1 hour** to migrate existing SIMD code:
- Update dependencies
- Replace imports
- Fix minor API differences
- Test on stable Rust

## Recommendation

**Migrate to packed_simd_2** for production deployment:
1. Stable Rust compatibility
2. Same performance
3. Production-proven
4. Easier deployment (no nightly required)

## Alternative: Conditional Compilation

Support **both** std::simd (nightly) AND packed_simd_2 (stable):

```rust
#[cfg(feature = "simd_nightly")]
use std::simd::{u8x16, u64x4};

#[cfg(all(feature = "simd", not(feature = "simd_nightly")))]
use packed_simd_2::{u8x16, u64x4};
```

This allows:
- Nightly users get bleeding-edge std::simd
- Stable users get packed_simd_2
- Same codebase supports both

## Next Steps

1. Add `packed_simd_2` dependency
2. Update `crates/storage/src/simd_encode.rs`
3. Test on stable Rust
4. Merge to main
