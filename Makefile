# MCP Helper Makefile
# Cross-platform development tasks

# Default target
.DEFAULT_GOAL := help

# Variables
CARGO := cargo
BINARY_NAME := mcp-helper
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DEBUG_DIR := $(TARGET_DIR)/debug

# Detect OS for platform-specific commands
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    OPEN_CMD := open
else ifeq ($(UNAME_S),Linux)
    OPEN_CMD := xdg-open
else
    OPEN_CMD := start
endif

# Phony targets
.PHONY: all help clean build build-release test test-all test-unit test-integration test-e2e \
        run install fmt fmt-check lint lint-all check doc audit hooks dev ci quick-test \
        bench bench-startup bench-performance coverage coverage-ci coverage-detailed test-property test-security \
        test-performance test-errors watch pre-push pre-commit

# Help target
help:
	@echo "MCP Helper - Development Tasks"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Main targets:"
	@echo "  clean          Remove build artifacts and temporary files"
	@echo "  build          Build debug binary"
	@echo "  test           Run all tests"
	@echo ""
	@echo "Additional targets:"
	@echo "  build-release  Build optimized release binary"
	@echo "  test-unit      Run unit tests only"
	@echo "  test-integration Run integration tests only"
	@echo "  test-e2e       Run end-to-end tests"
	@echo "  fmt            Format code using rustfmt"
	@echo "  lint           Run clippy linter"
	@echo "  check          Run cargo check"
	@echo "  doc            Generate and open documentation"
	@echo "  audit          Check for security vulnerabilities"
	@echo "  hooks          Install/update git hooks"
	@echo "  install        Install release binary to ~/.cargo/bin"
	@echo "  run            Run debug binary with example"
	@echo ""
	@echo "Test targets:"
	@echo "  test-errors    Run error handling tests"
	@echo "  test-security  Run security-focused tests"
	@echo "  test-property  Run property-based tests"
	@echo "  test-performance Run performance tests"
	@echo "  coverage-detailed Generate detailed coverage report"
	@echo ""
	@echo "Benchmark targets:"
	@echo "  bench          Run all benchmarks"
	@echo "  bench-startup  Run startup time benchmarks"
	@echo "  bench-performance Run performance benchmarks"

# Clean target - remove all build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@$(CARGO) clean
	@rm -f Cargo.lock
	@rm -f tarpaulin-report.html
	@echo "✓ Clean complete"

# Build target - build debug binary
build:
	@echo "Building debug binary..."
	@$(CARGO) build
	@echo "✓ Debug build complete: $(DEBUG_DIR)/$(BINARY_NAME)"

# Build release target - build optimized binary
build-release:
	@echo "Building release binary..."
	@$(CARGO) build --release
	@echo "✓ Release build complete: $(RELEASE_DIR)/$(BINARY_NAME)"

# Test target - run all tests
test: test-unit test-integration test-e2e
	@echo "✓ All tests passed"

# Test unit - run unit tests only
test-unit:
	@echo "Running unit tests..."
	@$(CARGO) test --lib -- --test-threads=1

# Test integration - run integration tests only  
test-integration:
	@echo "Running integration tests..."
	@$(CARGO) test --test '*' -- --test-threads=1

# Test E2E - run end-to-end tests
test-e2e:
	@echo "Building binary for E2E tests..."
	@$(CARGO) build --quiet
	@echo "Running E2E tests..."
	@$(CARGO) test --test 'e2e_*' -- --test-threads=1
	@echo "✓ E2E tests passed"

# Test E2E with specific scenario
test-e2e-%:
	@echo "Running E2E test: $*..."
	@$(CARGO) test --test 'e2e_$*' -- --test-threads=1

# Test all with verbose output
test-all:
	@echo "Running all tests with verbose output..."
	@$(CARGO) test --all -- --nocapture --test-threads=1

# Format code
fmt:
	@echo "Formatting code..."
	@$(CARGO) fmt
	@echo "✓ Code formatted"

# Check formatting without modifying files
fmt-check:
	@echo "Checking code formatting..."
	@$(CARGO) fmt -- --check
	@echo "✓ Formatting check complete"

# Run linter
lint:
	@echo "Running clippy..."
	@$(CARGO) clippy -- -D warnings
	@echo "✓ Linting complete"

# Run linter on all targets including tests
lint-all:
	@echo "Running clippy on all targets..."
	@$(CARGO) clippy --all-targets -- -D warnings
	@echo "✓ Linting complete"

# Check code without building
check:
	@echo "Checking code..."
	@$(CARGO) check --all-targets
	@echo "✓ Check complete"

# Generate documentation
doc:
	@echo "Generating documentation..."
	@$(CARGO) doc --no-deps --open
	@echo "✓ Documentation generated"

# Security audit
audit:
	@echo "Running security audit..."
	@$(CARGO) audit || (echo "Install cargo-audit with: cargo install cargo-audit" && exit 1)

# Install to cargo bin directory
install: build-release
	@echo "Installing to ~/.cargo/bin..."
	@mkdir -p ~/.cargo/bin
	@cp $(RELEASE_DIR)/$(BINARY_NAME) ~/.cargo/bin/
	@echo "✓ Installed to ~/.cargo/bin/$(BINARY_NAME)"

# Run example
run: build
	@echo "Running example..."
	@$(DEBUG_DIR)/$(BINARY_NAME) run cowsay "Hello from MCP Helper!"

# Development workflow shortcuts
dev: fmt lint build test
	@echo "✓ Development checks complete"

# CI workflow
ci: clean fmt-check lint build audit coverage-ci
	@echo "✓ CI checks complete"

# Quick test - only run fast unit tests
quick-test:
	@$(CARGO) test --lib -- --test-threads=1

# Benchmark - run all benchmarks
bench: build-release bench-startup bench-performance
	@echo "✓ All benchmarks complete"

# Benchmark - startup time only
bench-startup:
	@echo "Running startup time benchmarks..."
	@$(CARGO) bench --bench startup_time

# Benchmark - performance suite
bench-performance:
	@echo "Running performance benchmarks..."
	@$(CARGO) bench --bench performance

# Coverage report (requires cargo-tarpaulin)
coverage:
	@echo "Generating coverage report..."
	@$(CARGO) tarpaulin --out Html || (echo "Install cargo-tarpaulin with: cargo install cargo-tarpaulin" && exit 1)
	@$(OPEN_CMD) tarpaulin-report.html

# Coverage for CI - doesn't open browser
coverage-ci:
	@echo "Generating coverage report for CI..."
	@$(CARGO) tarpaulin --out Html --out Lcov --lib --bins --timeout 600 --exclude-files "src/cache.rs" || (echo "Install cargo-tarpaulin with: cargo install cargo-tarpaulin" && exit 1)

# Detailed coverage report with better exclusions
coverage-detailed:
	@echo "Generating detailed coverage report..."
	@$(CARGO) tarpaulin --out Html --out Lcov --lib --bins --tests --timeout 600 \
		--exclude-files "tests/common/*" --follow-exec \
		--ignore-panics --ignore-tests || (echo "Install cargo-tarpaulin with: cargo install cargo-tarpaulin" && exit 1)
	@$(OPEN_CMD) tarpaulin-report.html

# Run property-based tests
test-property:
	@echo "Running property-based tests..."
	@$(CARGO) test property_ -- --test-threads=1 --nocapture

# Run security-focused tests
test-security:
	@echo "Running security tests..."
	@$(CARGO) test security_ -- --test-threads=1 --nocapture

# Run performance/resource tests
test-performance:
	@echo "Running performance tests..."
	@$(CARGO) test --release perf_ -- --test-threads=1 --nocapture

# Run error handling tests specifically
test-errors:
	@echo "Running error handling tests..."
	@$(CARGO) test error_ -- --test-threads=1 --nocapture

# Watch for changes and rebuild (requires cargo-watch)
watch:
	@$(CARGO) watch -x check -x test -x build || (echo "Install cargo-watch with: cargo install cargo-watch" && exit 1)

# Git hooks setup
hooks:
	@echo "Setting up git hooks with rusty-hook..."
	@$(CARGO) test --quiet > /dev/null 2>&1 || true
	@echo "✓ Git hooks installed"

# Pre-push checks - comprehensive checks before pushing
pre-push: fmt-check lint test audit
	@echo "✓ All pre-push checks passed"

# Pre-commit checks - quick checks before committing
# Now includes linting all targets to catch test issues
pre-commit: fmt-check lint-all quick-test
	@echo "✓ All pre-commit checks passed"