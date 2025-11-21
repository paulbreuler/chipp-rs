# Contributing to chipp-rs

Thank you for your interest in contributing to chipp-rs! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Development Setup](#development-setup)
- [Pre-Commit Hooks](#pre-commit-hooks)
- [Code Quality Standards](#code-quality-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

---

## Development Setup

### Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Just** (optional but recommended): Command runner for common tasks
  ```bash
  # Using Homebrew (macOS)
  brew install just

  # Using cargo
  cargo install just

  # Using apt (Ubuntu/Debian)
  sudo apt install just
  ```

- **Pre-commit** (optional): Git hooks for automatic quality checks
  ```bash
  # Using pip
  pip install pre-commit

  # Using Homebrew (macOS)
  brew install pre-commit

  # Using apt (Ubuntu/Debian)
  sudo apt install pre-commit
  ```

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/paulbreuler/chipp-rs.git
cd chipp-rs

# Install pre-commit hooks (optional)
pre-commit install

# Verify installation
pre-commit --version
```

### Quick Start with Just

If you have `just` installed, you can run common tasks easily:

```bash
# Run all quality checks (fmt, clippy, tests, docs)
just quality

# Format code
just fmt

# Run tests
just test

# Build docs and open in browser
just docs-open

# See all available commands
just --list
```

---

## Pre-Commit Hooks

Pre-commit hooks automatically enforce code quality standards before commits are allowed.

### What Gets Checked

The pre-commit hooks run the following checks:

1. **`cargo fmt --check`** - Verifies code is formatted correctly
2. **`cargo clippy`** - Lints code and enforces zero warnings
3. **File quality checks** - Trailing whitespace, file endings, YAML/TOML syntax, etc.

### Installing Hooks

After cloning the repository, install the hooks:

```bash
pre-commit install
```

This creates a `.git/hooks/pre-commit` script that runs automatically on every commit.

### Running Hooks Manually

You can run the hooks manually without committing:

```bash
# Run on all files
pre-commit run --all-files

# Run on staged files only
pre-commit run

# Run a specific hook
pre-commit run cargo-fmt
pre-commit run cargo-clippy
```

### Bypassing Hooks (Emergency Only)

If you need to commit without running hooks (not recommended):

```bash
git commit --no-verify -m "Emergency fix"
```

**Note**: CI will still run these checks, so bypassing locally only delays the feedback.

### Updating Hooks

To update pre-commit hooks to the latest versions:

```bash
pre-commit autoupdate
```

### Common Issues

**Issue**: `cargo fmt` or `cargo clippy` not found

**Solution**: Ensure Rust toolchain is installed and in your PATH:
```bash
rustup component add rustfmt clippy
```

**Issue**: Hooks are slow

**Solution**: Hooks run `cargo clippy` on the entire workspace, which can be slow. Consider:
- Running `cargo clippy` manually before committing
- Using `--no-verify` for WIP commits (but fix before pushing)

---

## Code Quality Standards

### Quick Check: Run All Quality Checks

The easiest way to verify your code meets all standards:

```bash
# Using just (recommended)
just quality

# Or manually
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib
cargo doc --no-deps --all-features
```

### Formatting

All code must be formatted with `rustfmt`:

```bash
cargo fmt --all
```

We use the default `rustfmt` configuration. Do not commit unformatted code.

### Linting

All code must pass `clippy` with zero warnings:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Fix all clippy warnings before committing. If you believe a warning is a false positive, use `#[allow(clippy::lint_name)]` with a comment explaining why.

### Documentation

- **Every public item** must have documentation
- **Examples** in doc comments must compile and run
- **Crate-level docs** in `src/lib.rs` must be comprehensive

Verify docs build without warnings:

```bash
cargo doc --no-deps
```

### Testing

- **Unit tests** for all public APIs
- **Integration tests** for real API calls (gated behind `integration-tests` feature)
- **Doc tests** for all examples in documentation

Run tests:

```bash
# Unit tests
cargo test --lib

# Integration tests (requires API key)
export CHIPP_API_KEY="your-api-key"
export CHIPP_APP_NAME_ID="your-app-name-id"
cargo test --features integration-tests -- --ignored

# Doc tests
cargo test --doc
```

---

## Pull Request Process

### Before Submitting

1. **Run all checks locally**:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --lib
   cargo doc --no-deps
   ```

2. **Update documentation**:
   - Add/update doc comments for new/changed APIs
   - Update README.md if adding features
   - Update CHANGELOG.md (see [Release Process](#release-process))

3. **Add tests**:
   - Unit tests for new functionality
   - Integration tests for API changes
   - Doc tests for examples

### PR Guidelines

- **Title**: Use conventional commit format (e.g., `feat: add retry logic`, `fix: handle timeout errors`)
- **Description**: Explain what changed and why
- **Link issues**: Reference related issues (e.g., `Closes #123`)
- **Keep focused**: One feature/fix per PR
- **Rebase**: Keep commits clean and rebased on latest main

### Conventional Commit Format

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `style:` - Formatting, missing semicolons, etc.
- `refactor:` - Code change that neither fixes a bug nor adds a feature
- `perf:` - Performance improvement
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks, dependency updates

Examples:
```
feat: add exponential backoff retry logic
fix: handle partial SSE events in streaming
docs: add error handling examples
test: add unit tests for retry behavior
```

### Review Process

1. **Automated checks**: CI must pass (tests, clippy, fmt)
2. **Code review**: At least one maintainer approval required
3. **Documentation**: Verify docs are complete and accurate
4. **Testing**: Verify tests cover new functionality

---

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backward compatible
- **PATCH** (0.0.1): Bug fixes, backward compatible

### CHANGELOG.md

All changes must be documented in `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [Unreleased]

### Added
- New feature description

### Changed
- Changed behavior description

### Fixed
- Bug fix description

### Deprecated
- Deprecated feature description

### Removed
- Removed feature description

### Security
- Security fix description
```

### Pre-Release Checklist

Before releasing a new version:

- [ ] All tests pass
- [ ] CHANGELOG.md updated
- [ ] Version bumped in `Cargo.toml`
- [ ] Documentation reviewed
- [ ] Examples tested
- [ ] `cargo publish --dry-run` succeeds

See `.augment/rules/10-library-publishing.md` for detailed publishing standards.

---

## Getting Help

- **Issues**: Open an issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: See [README.md](README.md) and [docs.rs/chipp](https://docs.rs/chipp)

---

## Code of Conduct

Be respectful, inclusive, and professional. We're all here to build great software together.

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

