---
type: agent_requested
description: Rust SDK patterns including async/await, error handling, API design, testing, and idiomatic Rust for HTTP client libraries
---

# Rust SDK Development Patterns

## Principles

- **Async-first**: Use Tokio for all I/O operations
- **Type safety**: Leverage Rust's type system; prefer newtypes over primitives
- **Error handling**: Use `thiserror` for typed errors; avoid `unwrap()` in library code
- **Observability**: Support tracing with `tracing` crate
- **Security**: Validate all inputs; never leak credentials; safe defaults
- **Performance**: Avoid allocations in hot paths; use references
- **API design**: Follow Rust API Guidelines; implement common traits

## Dependencies for HTTP Client SDKs

```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
# HTTP client
reqwest = { version = "0.11", features = ["json", "stream"] }
# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# Error handling
thiserror = "1"
# Observability (optional)
tracing = "0.1"
# Utilities
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio-test = "0.4"
mockito = "1"  # HTTP mocking for tests
```

## Idiomatic Rust Patterns

### Ownership and Borrowing

```rust
// Prefer borrowing over cloning
pub async fn send_request(&self, request: &ChatRequest) -> Result<ChatResponse> {
    // Read-only access, no ownership transfer
    let response = self.client.post(&self.url).json(request).send().await?;
    Ok(response.json().await?)
}

// Take ownership when consuming
pub fn into_session(self) -> ChippSession {
    // Self is consumed and cannot be used after
    ChippSession {
        chat_session_id: self.session_id,
    }
}

// Use mutable references for in-place updates
pub fn update_session(&mut self, session_id: String) {
    self.chat_session_id = Some(session_id);
}
```

### Newtypes for Type Safety

```rust
// Wrap primitives to prevent misuse
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Compiler prevents mixing different ID types
fn get_session(session_id: SessionId) -> Result<Session> {
    // Type-safe: cannot accidentally pass wrong ID type
}
```

### Builder Pattern for Configuration

```rust
pub struct ClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
}

impl ClientConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.example.com".to_string(),
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

// Usage
let config = ClientConfig::new("api-key")
    .with_base_url("https://custom.api.com")
    .with_timeout(Duration::from_secs(60));
```

## Async Best Practices with Tokio

### Async Functions for I/O

```rust
// Good: async for I/O operations
pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
    let response = self.client
        .post(&self.url)
        .json(&request)
        .send()
        .await?;
    
    Ok(response.json().await?)
}

// Bad: blocking in async context
pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
    std::thread::sleep(Duration::from_secs(1)); // ❌ Blocks executor
    // ...
}
```

### Streaming Responses

```rust
use futures::stream::Stream;
use futures::StreamExt;

pub async fn chat_stream(
    &self,
    request: ChatRequest,
) -> Result<impl Stream<Item = Result<StreamEvent>>> {
    let response = self.client
        .post(&self.url)
        .json(&request)
        .send()
        .await?;

    let stream = response
        .bytes_stream()
        .map(|result| {
            result
                .map_err(ChippClientError::from)
                .and_then(|bytes| parse_stream_event(&bytes))
        });

    Ok(stream)
}
```

### Timeout Handling

```rust
use tokio::time::{timeout, Duration};

pub async fn chat_with_timeout(
    &self,
    request: ChatRequest,
    timeout_duration: Duration,
) -> Result<ChatResponse> {
    timeout(timeout_duration, self.chat(request))
        .await
        .map_err(|_| ChippClientError::Timeout)?
}
```

## Error Handling

### Typed Errors with thiserror

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChippClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
    
    #[error("Authentication failed: {0}")]
    AuthError(String),
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ChippClientError>;
```

### Error Context

```rust
// Add context to errors
pub async fn get_session(&self, id: &str) -> Result<Session> {
    let response = self.client
        .get(&format!("{}/sessions/{}", self.base_url, id))
        .send()
        .await
        .map_err(|e| ChippClientError::InvalidResponse(
            format!("Failed to fetch session {}: {}", id, e)
        ))?;
    
    response.json().await.map_err(Into::into)
}
```

### Never Panic in Library Code

```rust
// Bad: panics in library code
pub fn parse_response(data: &str) -> Response {
    serde_json::from_str(data).unwrap() // ❌ Panics on invalid JSON
}

// Good: return Result
pub fn parse_response(data: &str) -> Result<Response> {
    serde_json::from_str(data).map_err(Into::into)
}
```

## Testing Patterns

### Unit Tests with Arrange-Act-Assert

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

### Integration Tests with HTTP Mocking

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_api_integration() {
        // ARRANGE
        let _m = mock("POST", "/chat")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message": {"content": "Hello!"}}"#)
            .create();

        let config = ChippConfig::new("test-key")
            .with_base_url(server_url());
        let client = ChippClient::new(config);

        // ACT
        let response = client.chat(ChatRequest::default()).await;

        // ASSERT
        assert!(response.is_ok());
    }
}
```

## Documentation Standards

### Rustdoc Comments

```rust
/// Send a chat completion request to the Chipp API.
///
/// This method sends a non-streaming chat request and waits for the complete response.
/// For streaming responses, use [`chat_stream`](Self::chat_stream).
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
/// Returns `ChippClientError::AuthError` if the API key is invalid.
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

## Common Pitfalls to Avoid

1. **Don't use `unwrap()` or `expect()` in library code** - Return `Result` instead
2. **Don't block in async functions** - Use async I/O, not `std::thread::sleep`
3. **Don't expose internal types** - Keep implementation details private
4. **Don't break semver** - Additive changes only for minor versions
5. **Don't skip documentation** - Every public item needs rustdoc
6. **Don't ignore clippy warnings** - Fix them or explicitly allow with justification
7. **Don't leak credentials** - Never log API keys or tokens

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Thiserror Documentation](https://docs.rs/thiserror/)
- [Reqwest Documentation](https://docs.rs/reqwest/)

