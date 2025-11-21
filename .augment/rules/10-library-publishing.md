---
description: Standards for publishing chipp-rs to crates.io, semver compliance, and release process
behavior: auto
---

# Library Publishing Standards

## Pre-Publishing Checklist

Before publishing ANY version to crates.io:

### Code Quality

- [ ] `cargo build --release` succeeds with zero warnings
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo fmt --check` passes (all code formatted)
- [ ] `cargo test` passes (all unit tests)
- [ ] `cargo test --doc` passes (all doc tests compile and run)
- [ ] `cargo doc --no-deps` builds without warnings

### Documentation

- [ ] Every public item has doc comments
- [ ] Crate-level docs (`src/lib.rs`) include:
  - Overview of what the library does
  - Quick start example
  - Link to examples directory
  - Link to official API docs
- [ ] All doc examples compile (use `no_run` if they need API keys)
- [ ] README.md is up-to-date and matches crate docs
- [ ] CHANGELOG.md updated with all changes since last version

### Examples

- [ ] All examples in `examples/` directory run successfully
- [ ] Examples demonstrate common use cases:
  - Simple non-streaming chat
  - Streaming chat
  - Session continuity
  - Error handling
- [ ] Examples include clear comments explaining what they do

### Metadata

- [ ] `Cargo.toml` metadata complete:
  - `description` (concise, < 100 chars)
  - `repository` (GitHub URL)
  - `documentation` (docs.rs URL)
  - `homepage` (chipp.ai)
  - `keywords` (max 5, relevant)
  - `categories` (from crates.io list)
  - `license` (MIT OR Apache-2.0)
  - `readme` (README.md)
- [ ] Version number follows semver (see below)

### Testing

- [ ] Integration tests pass (if applicable)
- [ ] Tested on multiple platforms (macOS, Linux, Windows if possible)
- [ ] No hardcoded API keys or secrets in code or tests

## Semantic Versioning (Semver)

Follow strict semver: `MAJOR.MINOR.PATCH`

### Pre-1.0 (0.x.y)

- **0.x.0**: Breaking changes allowed in minor version
- **0.x.y**: Patch for bug fixes and non-breaking additions

### Post-1.0 (1.x.y)

- **MAJOR (1.x.x → 2.0.0)**: Breaking changes
  - Changed function signatures
  - Removed public APIs
  - Changed behavior that breaks existing code
- **MINOR (1.1.x → 1.2.0)**: New features, non-breaking
  - New public APIs
  - New optional parameters with defaults
  - Deprecations (with warnings)
- **PATCH (1.1.1 → 1.1.2)**: Bug fixes only
  - No API changes
  - No new features
  - Only fixes for incorrect behavior

### Breaking Changes

**Examples of breaking changes**:

```rust
// ❌ Breaking: Changed return type
// Before: pub fn chat(...) -> Result<String>
// After:  pub fn chat(...) -> Result<Response>

// ❌ Breaking: Removed public field
// Before: pub struct ChippConfig { pub api_key: String }
// After:  pub struct ChippConfig { api_key: String }

// ❌ Breaking: Changed required parameter
// Before: pub fn new(api_key: String) -> Self
// After:  pub fn new(config: ChippConfig) -> Self
```

**Examples of non-breaking changes**:

```rust
// ✅ Non-breaking: Added optional parameter with default
impl ChippConfig {
    // Before: Only had api_key, base_url, model
    // After:  Added timeout with default value
    pub timeout: Duration, // Default in impl Default
}

// ✅ Non-breaking: Added new method
impl ChippClient {
    pub fn chat_stream(...) -> ... { } // New method, doesn't affect existing code
}

// ✅ Non-breaking: Deprecated old API
#[deprecated(since = "0.2.0", note = "Use `chat_stream` instead")]
pub fn stream_chat(...) -> ... { }
```

## CHANGELOG.md Format

Use [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New feature X

### Changed
- Improved Y

### Fixed
- Bug Z

## [0.2.0] - 2024-01-15

### Added
- Streaming support with `chat_stream()` method
- Retry logic with exponential backoff
- Configurable timeouts

### Changed
- Improved error messages with actionable context

### Fixed
- Session ID not persisting across requests

## [0.1.0] - 2024-01-01

### Added
- Initial release
- Non-streaming chat with `chat()` method
- Session management with `ChippSession`
- Comprehensive error handling
```

## Publishing Process

### 1. Update Version

```bash
# Edit Cargo.toml
version = "0.2.0"  # Bump according to semver
```

### 2. Update CHANGELOG.md

Move `[Unreleased]` changes to new version section:

```markdown
## [0.2.0] - 2024-01-15

### Added
- Feature X

## [0.1.0] - 2024-01-01
...
```

### 3. Run Pre-Publishing Checks

```bash
# Format code
cargo fmt

# Check for warnings
cargo clippy -- -D warnings

# Run all tests
cargo test
cargo test --doc

# Build docs
cargo doc --no-deps

# Test examples
cargo run --example simple
cargo run --example streaming
cargo run --example session
```

### 4. Commit and Tag

```bash
git add .
git commit -m "Release v0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin main
git push origin v0.2.0
```

### 5. Publish to crates.io

```bash
# Dry run first
cargo publish --dry-run

# Publish for real
cargo publish
```

### 6. Verify Publication

- Check crates.io page: <https://crates.io/crates/chipp>
- Check docs.rs page: <https://docs.rs/chipp>
- Test installation: `cargo add chipp` in a new project

## docs.rs Configuration

Configure docs.rs in `Cargo.toml`:

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

This ensures docs.rs builds with all features enabled and shows feature-gated items.

## Yanking Versions

If a version has critical bugs or security issues:

```bash
# Yank version (prevents new projects from using it)
cargo yank --vers 0.1.5

# Un-yank if it was a mistake
cargo yank --vers 0.1.5 --undo
```

**When to yank**:

- Critical security vulnerability
- Data corruption bug
- Completely broken functionality

**When NOT to yank**:

- Minor bugs (publish a patch instead)
- Deprecations (use semver and deprecation warnings)

## Release Cadence

- **Patch releases**: As needed for bug fixes (no schedule)
- **Minor releases**: When new features are ready and tested
- **Major releases**: Rare, only for significant breaking changes

## Communication

### Release Announcements

After publishing:

1. Create GitHub release with changelog
2. Announce on relevant channels (if applicable)
3. Update README.md badges if needed

### Deprecation Warnings

Give users time to migrate:

```rust
#[deprecated(since = "0.3.0", note = "Use `chat_stream` instead. Will be removed in 1.0.0")]
pub fn stream_chat(...) -> ... { }
```

- Deprecate in minor version (e.g., 0.3.0)
- Remove in next major version (e.g., 1.0.0)
- Give at least 2-3 minor versions before removal

## Security

### Reporting Vulnerabilities

Include `SECURITY.md` in repository:

```markdown
# Security Policy

## Reporting a Vulnerability

Please report security vulnerabilities to: security@example.com

Do NOT open public GitHub issues for security vulnerabilities.
```

### Dependency Audits

Run `cargo audit` regularly:

```bash
# Install cargo-audit
cargo install cargo-audit

# Check for vulnerabilities
cargo audit
```

## License

Dual-license under MIT OR Apache-2.0:

```toml
[package]
license = "MIT OR Apache-2.0"
```

Include both `LICENSE-MIT` and `LICENSE-APACHE` files in repository.

