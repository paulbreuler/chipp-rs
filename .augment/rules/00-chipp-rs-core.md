---
type: "always_apply"
---

# chipp-rs Core Principles

## Mission

Build a production-ready, idiomatic Rust SDK for the Chipp.ai API that developers love to use.

## SDK Priorities

- **Simplicity**: Clear, minimal API surface
- **Correctness**: Comprehensive tests, proper error handling
- **Documentation**: Excellent rustdoc, examples, README
- **Async-first**: Built on tokio and reqwest
- **Semver stability**: Follow semantic versioning strictly
- **Crates.io ready**: Proper metadata, licensing, CI/CD

## Architecture Principles

- **Extreme ownership**: If the brief is incomplete, close the gaps
- **Start with why**: Tie every technical choice to user and business impact
- **Ruthless simplicity**: Fewer moving parts win
- **Bias toward action**: Propose, validate, iterate
- **Idiomatic Rust**: Leverage ownership, traits, async/await
- **User-focused**: API design from the user's perspective

## Code Principles

- Prefer small, composable functions; pure when feasible
- Errors: return typed errors with context using `thiserror`; avoid `unwrap()` in library code
- Security: input validation, safe defaults, no credential leaks
- Performance: async/await for I/O, avoid unnecessary allocations
- Tests: arrange–act–assert with real edge cases; include negative tests
- Docs: rustdoc comments stating purpose, examples, errors, panics

## Quality Gates

Before publishing to crates.io:

- ✅ `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ `cargo fmt --check`
- ✅ `cargo test`
- ✅ `cargo doc --no-deps --all-features` (no warnings)
- ✅ All examples run successfully
- ✅ README is accurate and complete
- ✅ CHANGELOG.md is updated
- ✅ `cargo publish --dry-run` succeeds

## Fail Fast Checklist

- Is the API intuitive for Rust developers?
- Is the change observable, testable, and backward-compatible?
- Does the documentation explain WHY, not just WHAT?
- Will users understand error messages?

## Git Workflow

**NEVER commit directly to `main`**. Always use feature branches:

```bash
# 1. Create branch from main
git checkout main && git pull
git checkout -b <type>/issue-<number>-<description>

# 2. Make changes and commit to the branch
git add <files>
git commit -m "<type>: <description>"

# 3. Push branch and create PR
git push -u origin <branch-name>
# Use create-pr command or GitHub API
```

**Branch naming**: `<type>/issue-<number>-<short-description>`
- `fix/issue-3-client-new-returns-result`
- `feat/issue-10-add-retry-logic`
- `chore/issue-4-tdd-enforcement-rule`

## Release Process

1. **Conventional Commits**: Use `feat:`, `fix:`, `docs:`, `chore:`, etc.
2. **Changelog**: Update CHANGELOG.md with user-facing changes
3. **Version Bump**: Follow semver (0.x.y for pre-1.0, x.y.z for stable)
4. **Tag**: Create git tag `vX.Y.Z`
5. **Publish**: `cargo publish` after CI passes
6. **GitHub Release**: Create release with changelog excerpt

## Rust Best Practices

### Error Handling

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChippClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, ChippClientError>;
```

### Async/Await

```rust
// Good: async for I/O operations
pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
    let response = self.client.post(&self.url).json(&request).send().await?;
    Ok(response.json().await?)
}

// Bad: blocking in async context
pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
    std::thread::sleep(Duration::from_secs(1)); // ❌ Blocks executor
    // ...
}
```

### Documentation

```rust
/// Send a chat completion request to the Chipp API.
///
/// # Arguments
///
/// * `request` - The chat request containing messages and configuration
///
/// # Returns
///
/// Returns a `ChatResponse` containing the AI's response message.
///
/// # Errors
///
/// Returns `ChippClientError::HttpError` if the network request fails.
/// Returns `ChippClientError::InvalidResponse` if the API returns malformed data.
///
/// # Examples
///
/// ```no_run
/// use chipp_rs::{ChippClient, ChippConfig, ChatRequest, Message};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ChippClient::new(ChippConfig::default());
/// let request = ChatRequest {
///     messages: vec![Message::user("Hello!")],
///     ..Default::default()
/// };
/// let response = client.chat(request).await?;
/// println!("Response: {}", response.message.content);
/// # Ok(())
/// # }
/// ```
pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
    // Implementation
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_success() {
        // ARRANGE
        let client = ChippClient::new(ChippConfig::default());
        let request = ChatRequest {
            messages: vec![Message::user("Test")],
            ..Default::default()
        };

        // ACT
        let result = client.chat(request).await;

        // ASSERT
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.message.content.is_empty());
    }

    #[tokio::test]
    async fn test_chat_invalid_api_key() {
        // ARRANGE
        let mut config = ChippConfig::default();
        config.api_key = "invalid".to_string();
        let client = ChippClient::new(config);

        // ACT
        let result = client.chat(ChatRequest::default()).await;

        // ASSERT
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ChippClientError::HttpError(_)));
    }
}
```

## SDK-Specific Patterns

### Builder Pattern for Configuration

```rust
pub struct ChippConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
}

impl ChippConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.chipp.ai".to_string(),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
```

### Session Management

```rust
pub struct ChippSession {
    chat_session_id: Option<String>,
}

impl ChippSession {
    pub fn new() -> Self {
        Self { chat_session_id: None }
    }

    pub fn session_id(&self) -> Option<&str> {
        self.chat_session_id.as_deref()
    }
}
```

## Common Pitfalls to Avoid

1. **Don't use `unwrap()` or `expect()` in library code** - Return `Result` instead
2. **Don't block in async functions** - Use async I/O, not `std::thread::sleep`
3. **Don't expose internal types** - Keep implementation details private
4. **Don't break semver** - Additive changes only for minor versions
5. **Don't skip documentation** - Every public item needs rustdoc
6. **Don't ignore clippy warnings** - Fix them or explicitly allow with justification

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Thiserror Documentation](https://docs.rs/thiserror/)

