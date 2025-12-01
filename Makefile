# ============================================================================
# rust-kgdb Makefile
# ============================================================================
#
# Quick Start:
#   make test          - Run all tests
#   make bench         - Run all benchmarks
#   make release-019   - Create v0.1.9 release (example)
#   make publish-npm   - Publish to npm
#   make publish-pypi  - Publish to PyPI
#   make all           - Full CI pipeline (test + bench + build)
#
# ============================================================================

.PHONY: help test bench build clean release publish-npm publish-pypi all regression
.DEFAULT_GOAL := help

# Version (update this for new releases)
VERSION ?= 0.1.9

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
NC := \033[0m # No Color

help: ## Show this help message
	@echo ""
	@echo "$(BLUE)rust-kgdb Automation$(NC)"
	@echo "====================="
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""

# ============================================================================
# Testing
# ============================================================================

test: ## Run all workspace tests
	@echo "$(BLUE)Running workspace tests...$(NC)"
	cargo test --workspace --quiet
	@echo "$(GREEN)✅ All tests passing$(NC)"

test-verbose: ## Run tests with verbose output
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	cargo test --workspace

regression: ## Run regression tests
	@echo "$(BLUE)Running regression tests...$(NC)"
	cargo test --workspace --release
	@echo "$(GREEN)✅ Regression tests complete$(NC)"

# ============================================================================
# Benchmarking
# ============================================================================

bench: ## Run all benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cargo bench --workspace

bench-lubm: ## Run LUBM benchmarks only
	@echo "$(BLUE)Running LUBM benchmarks...$(NC)"
	cargo bench --package sparql --bench lubm_wcoj_benchmark

bench-sp2: ## Run SP2Bench benchmarks only
	@echo "$(BLUE)Running SP2Bench benchmarks...$(NC)"
	cargo bench --package sparql --bench sp2bench_benchmark

bench-storage: ## Run storage benchmarks only
	@echo "$(BLUE)Running storage benchmarks...$(NC)"
	cargo bench --package storage --bench triple_store_benchmark

# ============================================================================
# Building
# ============================================================================

build: ## Build workspace in release mode
	@echo "$(BLUE)Building workspace (release)...$(NC)"
	cargo build --workspace --release
	@echo "$(GREEN)✅ Build complete$(NC)"

build-debug: ## Build workspace in debug mode
	@echo "$(BLUE)Building workspace (debug)...$(NC)"
	cargo build --workspace

clean: ## Clean all build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cargo clean
	rm -rf sdks/typescript/dist
	rm -rf sdks/python/dist
	rm -rf sdks/python/build
	@echo "$(GREEN)✅ Cleanup complete$(NC)"

# ============================================================================
# Release Automation
# ============================================================================

release: ## Create new release (specify VERSION=x.y.z)
	@if [ -z "$(VERSION)" ]; then \
		echo "$(YELLOW)Error: VERSION not specified$(NC)"; \
		echo "Usage: make release VERSION=0.1.9"; \
		exit 1; \
	fi
	@echo "$(BLUE)Creating release v$(VERSION)...$(NC)"
	./scripts/release.sh $(VERSION)
	@echo "$(GREEN)✅ Release v$(VERSION) complete$(NC)"

release-dry: ## Preview release without making changes
	@if [ -z "$(VERSION)" ]; then \
		echo "$(YELLOW)Error: VERSION not specified$(NC)"; \
		echo "Usage: make release-dry VERSION=0.1.9"; \
		exit 1; \
	fi
	@echo "$(BLUE)Previewing release v$(VERSION)...$(NC)"
	./scripts/release.sh $(VERSION) --dry-run

release-019: ## Quick shortcut for v0.1.9 release
	$(MAKE) release VERSION=0.1.9

# ============================================================================
# Publishing
# ============================================================================

publish-npm: ## Publish TypeScript SDK to npm
	@echo "$(BLUE)Publishing to npm...$(NC)"
	cd sdks/typescript && npm publish
	@echo "$(GREEN)✅ Published to npm$(NC)"

publish-pypi: ## Publish Python SDK to PyPI
	@echo "$(BLUE)Publishing to PyPI...$(NC)"
	cd sdks/python && twine upload dist/rust_kgdb-$(VERSION)*
	@echo "$(GREEN)✅ Published to PyPI$(NC)"

build-npm: ## Build TypeScript SDK
	@echo "$(BLUE)Building TypeScript SDK...$(NC)"
	cd sdks/typescript && npm run build
	@echo "$(GREEN)✅ TypeScript SDK built$(NC)"

build-python: ## Build Python package
	@echo "$(BLUE)Building Python package...$(NC)"
	cd sdks/python && python3 -m build
	@echo "$(GREEN)✅ Python package built$(NC)"

# ============================================================================
# Full CI Pipeline
# ============================================================================

all: test regression bench build ## Full CI pipeline (test + bench + build)
	@echo ""
	@echo "$(GREEN)========================================$(NC)"
	@echo "$(GREEN)✅ Full CI Pipeline Complete$(NC)"
	@echo "$(GREEN)========================================$(NC)"
	@echo ""
	@echo "Summary:"
	@echo "  • Tests: PASSING"
	@echo "  • Regression: PASSING"
	@echo "  • Benchmarks: COMPLETE"
	@echo "  • Build: SUCCESS"
	@echo ""

# ============================================================================
# Development Helpers
# ============================================================================

fmt: ## Format code with rustfmt
	@echo "$(BLUE)Formatting code...$(NC)"
	cargo fmt --all
	@echo "$(GREEN)✅ Code formatted$(NC)"

lint: ## Lint code with clippy
	@echo "$(BLUE)Linting code...$(NC)"
	cargo clippy --workspace -- -D warnings
	@echo "$(GREEN)✅ Linting complete$(NC)"

check: ## Quick check without building
	@echo "$(BLUE)Checking code...$(NC)"
	cargo check --workspace
	@echo "$(GREEN)✅ Check complete$(NC)"

doc: ## Generate and open documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	cargo doc --no-deps --open
	@echo "$(GREEN)✅ Documentation generated$(NC)"

# ============================================================================
# Version Management
# ============================================================================

version: ## Show current version
	@echo "Current version: $(shell grep '^version = ' Cargo.toml | head -1 | cut -d'\"' -f2)"

versions: ## Show all SDK versions
	@echo "Workspace:   $(shell grep '^version = ' Cargo.toml | head -1 | cut -d'\"' -f2)"
	@echo "TypeScript:  $(shell grep '\"version\"' sdks/typescript/package.json | head -1 | cut -d'\"' -f4)"
	@echo "Python:      $(shell grep 'version=' sdks/python/setup.py | head -1 | cut -d'\"' -f2)"

# ============================================================================
# Git Helpers
# ============================================================================

git-status: ## Show git status and tags
	@git status -s
	@echo ""
	@echo "Latest tags:"
	@git tag --sort=-version:refname | head -5

git-push: ## Push to origin with tags
	@echo "$(BLUE)Pushing to GitHub...$(NC)"
	git push origin main --tags
	@echo "$(GREEN)✅ Pushed to GitHub$(NC)"

# ============================================================================
# Information
# ============================================================================

info: ## Show project information
	@echo ""
	@echo "$(BLUE)rust-kgdb Project Info$(NC)"
	@echo "======================"
	@echo ""
	@echo "Version:     $(shell grep '^version = ' Cargo.toml | head -1 | cut -d'\"' -f2)"
	@echo "Rust:        $(shell rustc --version)"
	@echo "Cargo:       $(shell cargo --version)"
	@echo ""
	@echo "Workspace Crates:"
	@cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | "  • \(.name) v\(.version)"'
	@echo ""
	@echo "Features:"
	@echo "  • SPARQL 1.1 (100% W3C compliant)"
	@echo "  • RDF 1.2 (100% W3C compliant)"
	@echo "  • WCOJ execution (LeapFrog TrieJoin)"
	@echo "  • SIMD + PGO optimizations"
	@echo "  • 64 builtin functions"
	@echo ""
