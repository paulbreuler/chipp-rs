---
description: Core principles and standards for chipp-rs Rust SDK
behavior: always
---

# chipp-rs SDK Core Principles

## Mission

Ship a production-ready, ergonomic Rust SDK for the Chipp.ai API that developers trust.
Provide excellent documentation, comprehensive error handling, and a delightful developer experience.

## Project Type

**Rust library crate** for crates.io distribution. Not an application - this is infrastructure code that other developers depend on.

## Library Design Principles

- **Ergonomic API**: Intuitive, hard to misuse, follows Rust conventions
- **Comprehensive documentation**: Every public item documented with examples
- **Robust error handling**: Typed errors with context, never panic in library code
- **Async-first**: Built on tokio, non-blocking I/O throughout
- **Minimal dependencies**: Only essential, well-maintained crates
- **Semver compliance**: Strict semantic versioning, clear changelog
- **Zero-cost abstractions**: No runtime overhead for convenience features

## API Design Standards

### Public API Surface

- **Minimal and focused**: Expose only what users need
- **Hard to misuse**: Use type system to prevent invalid states
- **Consistent naming**: Follow Rust API guidelines (RFC 199, RFC 344)
- **Builder patterns**: For complex configuration (e.g., `ChippConfig`)
- **Sensible defaults**: Common use cases work with minimal configuration

### Error Handling

```rust
// Use thiserror for domain errors
#[derive(thiserror::Error, Debug)]
pub enum ChippClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}
```

- **Never panic**: Use `Result<T, E>` for all fallible operations
- **Rich context**: Include request IDs, status codes, helpful messages
- **Typed errors**: Enum variants for different error categories
- **Actionable messages**: Tell users what went wrong and how to fix it

### Async Patterns

- **Runtime-agnostic where possible**: But optimize for tokio (primary use case)
- **Stream for streaming**: Use `futures::Stream` for SSE responses
- **Cancellation-safe**: All async operations respect cancellation
- **Timeout support**: Configurable timeouts on all network operations

## Code Quality Standards

### Documentation

- **Every public item**: Module, struct, enum, function, method documented
- **Examples in docs**: Runnable examples in doc comments (tested with `cargo test`)
- **Crate-level docs**: Comprehensive guide in `src/lib.rs`
- **README.md**: Quick start, installation, common use cases
- **CHANGELOG.md**: All changes documented per semver

### Testing

- **Unit tests**: Test all public APIs, edge cases, error paths
- **Integration tests**: Real API calls (gated behind `integration-tests` feature)
- **Doc tests**: All examples in documentation must compile and run
- **Property tests**: For complex logic (if applicable)
- **Coverage target**: > 80% line coverage for core logic

### Code Style

- **rustfmt**: All code formatted with `cargo fmt`
- **clippy**: Zero warnings with `cargo clippy -- -D warnings`
- **No unsafe**: Unless absolutely necessary and documented
- **Idiomatic Rust**: Follow Rust API guidelines and community conventions

## Dependency Management

### Allowed Dependencies

- **tokio**: Async runtime (required)
- **reqwest**: HTTP client (required)
- **serde**: Serialization (required)
- **thiserror**: Error types (required)
- **tracing**: Structured logging (optional, feature-gated if possible)

### Dependency Criteria

- **Well-maintained**: Active development, responsive maintainers
- **Widely used**: Proven in production, large user base
- **Minimal transitive deps**: Avoid dependency bloat
- **Compatible licenses**: MIT/Apache-2.0 compatible

## Publishing Standards

### Before Publishing to crates.io

- ✅ All tests pass (`cargo test`)
- ✅ No clippy warnings (`cargo clippy -- -D warnings`)
- ✅ Formatted (`cargo fmt --check`)
- ✅ Documentation builds (`cargo doc --no-deps`)
- ✅ Examples run successfully
- ✅ CHANGELOG.md updated
- ✅ Version bumped per semver
- ✅ README.md accurate and up-to-date

### Semver Policy

- **Breaking changes**: Major version bump (1.x.x → 2.0.0)
- **New features**: Minor version bump (1.1.x → 1.2.0)
- **Bug fixes**: Patch version bump (1.1.1 → 1.1.2)
- **Pre-1.0**: 0.x.y allows breaking changes in minor versions

## Security Standards

- **API keys**: NEVER hardcode, always from env vars or config
- **TLS**: Use HTTPS for all API calls, validate certificates
- **Secrets in logs**: NEVER log API keys or sensitive data
- **Dependencies**: Regular `cargo audit` checks for vulnerabilities

## Performance Standards

- **Zero-cost abstractions**: No runtime overhead for convenience
- **Lazy evaluation**: Don't do work until needed
- **Efficient serialization**: Minimize allocations in hot paths
- **Connection pooling**: Reuse HTTP connections (reqwest handles this)

## Fail Fast Checklist

Before merging any PR:

- Does this follow Rust API guidelines?
- Is every public item documented with examples?
- Do all tests pass (including doc tests)?
- Is the error handling comprehensive and actionable?
- Would I want to use this API as a library consumer?
