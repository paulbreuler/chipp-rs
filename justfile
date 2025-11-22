# Justfile for chipp-rs
# See https://github.com/casey/just for installation and usage

# Default recipe (runs when you type `just`)
default:
    @just --list

# Run all quality checks (fmt, clippy, tests, docs)
quality:
    @echo "🔍 Running quality checks..."
    @echo ""
    @echo "📦 Formatting check..."
    cargo fmt --all -- --check
    @echo "✅ Formatting OK"
    @echo ""
    @echo "📦 Clippy check..."
    cargo clippy --all-targets --all-features -- -D warnings
    @echo "✅ Clippy OK"
    @echo ""
    @echo "📦 Running tests..."
    cargo test --lib
    @echo "✅ Tests OK"
    @echo ""
    @echo "📦 Building docs..."
    cargo doc --no-deps --all-features
    @echo "✅ Docs OK"
    @echo ""
    @echo "✅ All quality checks passed!"

# Format code with rustfmt
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy and automatically fix issues
clippy-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings

# Run unit tests
test:
    cargo test --lib

# Run all tests (including doc tests)
test-all:
    cargo test

# Run integration tests (requires API key)
test-integration:
    @echo "⚠️  Integration tests require CHIPP_API_KEY and CHIPP_APP_NAME_ID"
    cargo test --features integration-tests -- --ignored

# Generate code coverage report
coverage:
    @echo "📊 Generating code coverage report..."
    @echo ""
    @echo "Installing cargo-llvm-cov (if not already installed)..."
    @cargo install cargo-llvm-cov --quiet 2>/dev/null || true
    @echo ""
    @echo "Running tests with coverage..."
    cargo llvm-cov --all-features --workspace --html
    @echo ""
    @echo "✅ Coverage report generated!"
    @echo "📂 Open target/llvm-cov/html/index.html in your browser"
    @echo ""
    @echo "Coverage summary:"
    cargo llvm-cov --all-features --workspace --summary-only

# Generate coverage and check 80% threshold
coverage-check:
    @echo "📊 Checking code coverage threshold (80%)..."
    @cargo install cargo-llvm-cov --quiet 2>/dev/null || true
    @COVERAGE=$$(cargo llvm-cov --all-features --workspace --summary-only | grep -oP 'TOTAL.*\K\d+\.\d+(?=%)' || echo "0"); \
    echo "Current coverage: $${COVERAGE}%"; \
    if [ "$$(echo "$${COVERAGE} < 80.0" | bc -l)" -eq 1 ]; then \
        echo "❌ Coverage $${COVERAGE}% is below the required 80% threshold"; \
        exit 1; \
    else \
        echo "✅ Coverage $${COVERAGE}% meets the 80% threshold"; \
    fi

# Build documentation
docs:
    cargo doc --no-deps --all-features

# Build and open documentation in browser
docs-open:
    cargo doc --no-deps --all-features --open

# Check that code compiles
check:
    cargo check --all-targets --all-features

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Run pre-commit hooks on all files
pre-commit:
    pre-commit run --all-files

# Install pre-commit hooks
pre-commit-install:
    pre-commit install

# Update pre-commit hooks to latest versions
pre-commit-update:
    pre-commit autoupdate

# Run the quality check script (alternative to pre-commit)
check-quality:
    ./scripts/check-quality.sh

# Run all examples
examples:
    @echo "Running simple example..."
    cargo run --example simple
    @echo ""
    @echo "Running streaming example..."
    cargo run --example streaming
    @echo ""
    @echo "Running session example..."
    cargo run --example session

# Dry-run publish to crates.io
publish-dry-run:
    cargo publish --dry-run

# Publish to crates.io (requires confirmation)
publish:
    @echo "⚠️  This will publish to crates.io!"
    @echo "Have you:"
    @echo "  - Updated CHANGELOG.md?"
    @echo "  - Bumped version in Cargo.toml?"
    @echo "  - Run 'just quality'?"
    @echo "  - Run 'just publish-dry-run'?"
    @echo ""
    @read -p "Continue? (y/N) " confirm && [ "$$confirm" = "y" ] || exit 1
    cargo publish

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and run quality checks
watch-quality:
    cargo watch -x fmt -x 'clippy --all-targets --all-features -- -D warnings' -x 'test --lib'

# Show outdated dependencies
outdated:
    cargo outdated

# Update dependencies
update:
    cargo update

# Audit dependencies for security vulnerabilities
audit:
    cargo audit

# Install development tools
install-tools:
    @echo "Installing development tools..."
    cargo install cargo-watch
    cargo install cargo-outdated
    cargo install cargo-audit
    rustup component add rustfmt clippy
    @echo "✅ Development tools installed"

