# Contributing to chipp-rs

Thank you for your interest in contributing to chipp-rs! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Pre-Commit Hooks](#pre-commit-hooks)
- [Code Quality Standards](#code-quality-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

---

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. All contributors are expected to:

- Be respectful and inclusive in language and actions
- Accept constructive criticism gracefully
- Focus on what is best for the community and project
- Show empathy towards other community members

Harassment, trolling, or exclusionary behavior will not be tolerated.

---

## How to Contribute

### Reporting Bugs

Found a bug? Please [open an issue](https://github.com/paulbreuler/chipp-rs/issues/new) with:

- **Clear title**: Brief description of the bug
- **Environment**: Rust version, OS, chipp version
- **Steps to reproduce**: Minimal code example if possible
- **Expected behavior**: What you expected to happen
- **Actual behavior**: What actually happened
- **Error messages**: Full error output, stack traces

### Suggesting Features

Have an idea? Please [open an issue](https://github.com/paulbreuler/chipp-rs/issues/new) with:

- **Use case**: What problem does this solve?
- **Proposed solution**: How should it work?
- **Alternatives considered**: Other approaches you've thought about
- **Examples**: API design ideas, code snippets

### Contributing Code

1. Check [existing issues](https://github.com/paulbreuler/chipp-rs/issues) for something to work on
2. Comment on the issue to let others know you're working on it
3. Fork the repository and create a feature branch
4. Follow the [Development Setup](#development-setup) below
5. Submit a pull request following the [PR Process](#pull-request-process)

---

## Development Setup

### Prerequisites

- **Rust 1.83+** (MSRV): Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup component add rustfmt clippy
  ```

- **Just** (optional but recommended): Command runner for common tasks
  ```bash
  # Using Homebrew (macOS)
  brew install just

  # Using cargo
  cargo install just
  ```

- **Pre-commit** (optional): Git hooks for automatic quality checks
  ```bash
  # Using pip
  pip install pre-commit

  # Using Homebrew (macOS)
  brew install pre-commit
  ```

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/paulbreuler/chipp-rs.git
cd chipp-rs/chipp-rs

# Install pre-commit hooks (optional)
pre-commit install

# Verify setup
cargo build
cargo test
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

All PRs must pass the same checks as CI. Here's what CI runs:

| Check | Command | Requirement |
|-------|---------|-------------|
| Formatting | `cargo fmt --all -- --check` | No formatting issues |
| Linting | `cargo clippy --all-targets --all-features -- -D warnings` | Zero warnings |
| Tests | `cargo test --verbose` | All tests pass |
| Doc tests | `cargo test --doc` | All doc examples compile |
| Documentation | `cargo doc --no-deps --all-features` | No doc warnings |
| Coverage | `cargo llvm-cov` | ≥80% line coverage |
| MSRV | `cargo check` with Rust 1.83 | Compiles on MSRV |
| Semver | `cargo semver-checks check-release` | No breaking changes (minor/patch) |

### Quick Check: Run All Quality Checks

```bash
# Using just (recommended)
just quality

# Or manually
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --verbose
cargo test --doc
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
cargo doc --no-deps --all-features
```

Preview locally: `cargo doc --open`

Documentation is automatically built on [docs.rs](https://docs.rs/chipp) when published.

### Testing

- **Unit tests** for all public APIs
- **Integration tests** for real API calls (gated behind `integration-tests` feature)
- **Doc tests** for all examples in documentation
- **Code coverage**: Minimum 80% line coverage required

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

### Code Coverage

We enforce a minimum of **80% line coverage** for all code changes.

**Check coverage locally:**

```bash
# Generate HTML coverage report (shows uncovered lines)
just coverage

# Check if coverage meets 80% threshold (shows uncovered lines)
just coverage-check

# View detailed coverage report in browser
open target/llvm-cov/html/index.html
```

**Understanding coverage output:**

The `--show-missing-lines` flag provides detailed information about uncovered code:

```
TOTAL    297    144    51.52%    29    11    62.07%    246    114    53.66%    0    0    -
Uncovered Lines:
/path/to/src/lib.rs: 217, 218, 219, 358, 359, 360, ...
```

This shows:
- **Line coverage**: 53.66% (246 total lines, 114 uncovered)
- **Region coverage**: 51.52% (297 regions, 144 uncovered)
- **Function coverage**: 62.07% (29 functions, 11 uncovered)
- **Uncovered lines**: Exact line numbers that need test coverage

**Install cargo-llvm-cov:**

```bash
cargo install cargo-llvm-cov
```

**Coverage requirements:**

- All new features must include tests
- Bug fixes should include regression tests
- Coverage must not decrease with new PRs
- Integration tests are excluded from coverage (require API keys)

**CI enforcement:**

- GitHub Actions automatically checks coverage on all PRs using `cargo-llvm-cov`
- PRs that drop coverage below 80% will fail CI
- No external services required - all coverage checking is done locally in CI

**Viewing coverage:**

After running `just coverage`, open `target/llvm-cov/html/index.html` in your browser to see:
- Line-by-line coverage
- Function coverage
- Branch coverage
- Uncovered code highlighted in red

---

## Pull Request Process

### Branch Naming

Use descriptive branch names with a type prefix:

```
<type>/issue-<number>-<short-description>
```

Examples:
- `feat/issue-26-add-health-check`
- `fix/issue-15-timeout-handling`
- `docs/contributing-update`

### Before Submitting

1. **Run all checks locally**:
   ```bash
   # Using just (recommended)
   just quality

   # Or manually
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --verbose
   cargo test --doc
   cargo doc --no-deps --all-features
   ```

2. **Update documentation**:
   - Add/update doc comments for new/changed APIs
   - Update README.md if adding features

3. **Add tests**:
   - Unit tests for new functionality
   - Integration tests for API changes (feature-gated)
   - Doc tests for examples

### Conventional Commit Format

We use [Conventional Commits](https://www.conventionalcommits.org/) for all commits and PR titles:

| Type | Description |
|------|-------------|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `docs:` | Documentation only |
| `style:` | Formatting, whitespace |
| `refactor:` | Code change (no new feature or fix) |
| `perf:` | Performance improvement |
| `test:` | Adding or updating tests |
| `chore:` | Maintenance, dependencies |

**Breaking changes**: Add `!` after the type (e.g., `feat!: remove deprecated API`)

Examples:
```
feat: add exponential backoff retry logic
fix: handle partial SSE events in streaming
docs: add error handling examples
test: add unit tests for retry behavior
feat!: change chat() return type to Result
```

### PR Guidelines

- **Title**: Use conventional commit format
- **Description**: Explain what changed and why
- **Link issues**: Reference related issues (e.g., `Closes #123`)
- **Keep focused**: One feature/fix per PR
- **Rebase**: Keep commits clean and rebased on latest main

### Review Process

1. **Automated checks**: All 8 CI checks must pass
2. **Code review**: At least one maintainer approval required
3. **Documentation**: Verify docs are complete and accurate
4. **Testing**: Verify tests cover new functionality

---

## Release Process

> **Note**: Releases are automated via [Release Please](https://github.com/googleapis/release-please). Contributors don't need to manage versions or changelogs manually.

### How It Works

1. **Write conventional commits** - Your commit messages determine the version bump:
   - `feat:` → Minor version bump (0.1.0 → 0.2.0)
   - `fix:` → Patch version bump (0.1.0 → 0.1.1)
   - `feat!:` or `fix!:` → Major version bump (0.1.0 → 1.0.0)

2. **Merge to main** - When PRs are merged, Release Please:
   - Analyzes conventional commits since last release
   - Creates/updates a Release PR with version bump and changelog

3. **Merge Release PR** - When maintainers merge the Release PR:
   - Git tag is created automatically
   - GitHub Release is published
   - Crate is published to crates.io via CI

### Versioning

We follow [Semantic Versioning](https://semver.org/):

| Version | When to use | Example commits |
|---------|-------------|-----------------|
| **MAJOR** (1.0.0) | Breaking changes | `feat!: remove deprecated API` |
| **MINOR** (0.2.0) | New features | `feat: add health check methods` |
| **PATCH** (0.1.1) | Bug fixes | `fix: handle timeout correctly` |

### Manual Changelog (git-cliff)

For local previews or manual changelog generation, we also support [git-cliff](https://git-cliff.org/):

```bash
# Preview unreleased changes
git-cliff --unreleased

# Generate full changelog
git-cliff --output CHANGELOG.md
```

The repository includes a `cliff.toml` configuration file for changelog formatting.

---

## Getting Help

- **Issues**: [Open an issue](https://github.com/paulbreuler/chipp-rs/issues/new) for bugs or feature requests
- **Documentation**: See [README.md](README.md) and [docs.rs/chipp](https://docs.rs/chipp)

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

This is a standard open-source licensing practice. You retain copyright to your contributions while granting the project permission to use them.

