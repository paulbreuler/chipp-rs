---
description: Chipp API implementation patterns for chipp-rs SDK
behavior: always
---

# Chipp API SDK Implementation

**Official API Reference**: <https://chipp.ai/docs/api/reference>

This rule documents implementation patterns for the chipp-rs SDK based on actual API behavior.

## API Specification

### Base URL

`https://app.chipp.ai/api/v1`

### Authentication

```http
Authorization: Bearer <CHIPP_API_KEY>
```

**Security**: NEVER hardcode API keys. Always accept from user configuration or environment variables.

### Required Headers

```http
Authorization: Bearer <CHIPP_API_KEY>
Content-Type: application/json
X-Correlation-ID: <UUID>           # For request tracing (SDK generates automatically)
X-Chat-Session-ID: <session_id>    # Optional, for session continuity
```

### Request Body Format

```json
{
  "model": "myapp-123",              // Chipp appNameId (user-provided)
  "messages": [
    { "role": "system", "content": "You are a helpful assistant" },
    { "role": "user", "content": "What is Chipp?" }
  ],
  "stream": false,                   // true for SSE streaming
  "chatSessionId": "uuid-string"     // Optional, for conversation continuity
}
```

### Response Body Format (Non-Streaming)

```json
{
  "chatSessionId": "uuid-string",
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": "Chipp is a platform for building AI agents..."
      }
    }
  ]
}
```

## Streaming Implementation

### Actual API Behavior

⚠️ **The actual Chipp API streaming format differs from the official documentation!**

**Documentation shows**: Standard SSE with `data:` prefix and OpenAI-compatible JSON chunks

**Actual API returns**: Custom format with prefixes like `0:`, `e:`, `d:`, `f:`, `8:`

### Streaming Response Format

The API returns lines with custom prefixes:

```text
0:"chatSessionId-value"
0:{"chatSessionId":"...","choices":[{"delta":{"content":"Hello"}}]}
0:{"chatSessionId":"...","choices":[{"delta":{"content":" world"}}]}
e:[DONE]
```

### Parsing Strategy

```rust
// Parse each line, extract prefix and JSON payload
for line in response.lines() {
    if line.starts_with("0:") {
        let json = &line[2..]; // Skip "0:" prefix
        // Parse JSON chunk
    } else if line.starts_with("e:") {
        // End of stream marker
        break;
    }
}
```

### Stream Type

Return `impl Stream<Item = Result<String, ChippClientError>>` using `futures::Stream`.

## Session Management

### Session Lifecycle

- **One session per conversation**: `ChippSession` tracks `chatSessionId`
- **Session creation**: First request returns `chatSessionId` in response
- **Session continuity**: Include `chatSessionId` in subsequent requests
- **Session reset**: User calls `session.reset()` to start new conversation

### Implementation Pattern

```rust
pub struct ChippSession {
    pub chat_session_id: Option<String>,
}

impl ChippSession {
    pub fn new() -> Self {
        Self { chat_session_id: None }
    }

    pub fn reset(&mut self) {
        self.chat_session_id = None;
    }
}
```

**Usage**:

1. Create session: `let mut session = ChippSession::new();`
2. First request: `client.chat(&mut session, &messages).await?`
3. Session ID stored: `session.chat_session_id` now contains ID from response
4. Subsequent requests: Same session object maintains continuity

## Error Handling

### Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum ChippClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),
}
```

### Retry Logic

**Retry transient failures**:

- 5xx server errors
- Network timeouts
- Connection errors

**Do NOT retry**:

- 4xx client errors (invalid API key, bad request)
- Successful responses (2xx)

**Exponential backoff**:

```rust
// Retry with exponential backoff: 1s, 2s, 4s
let delays = [1, 2, 4];
for (attempt, delay) in delays.iter().enumerate() {
    match make_request().await {
        Ok(response) => return Ok(response),
        Err(e) if is_retryable(&e) => {
            tokio::time::sleep(Duration::from_secs(*delay)).await;
        }
        Err(e) => return Err(e),
    }
}
```

### Error Messages

Provide actionable error messages:

```rust
// ❌ Bad: Vague error
Err(ChippClientError::ApiError { status: 401, message: "Unauthorized" })

// ✅ Good: Actionable error
Err(ChippClientError::ApiError {
    status: 401,
    message: "Invalid API key. Check your CHIPP_API_KEY environment variable.".to_string()
})
```

## Configuration

### ChippConfig Structure

```rust
pub struct ChippConfig {
    pub api_key: String,        // User's Chipp API key
    pub base_url: String,       // Default: "https://app.chipp.ai/api/v1"
    pub model: String,          // Chipp appNameId (e.g., "myapp-123")
    pub timeout: Duration,      // Request timeout (default: 30s)
    pub max_retries: usize,     // Max retry attempts (default: 3)
}
```

### Sensible Defaults

```rust
impl Default for ChippConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(), // User must provide
            base_url: "https://app.chipp.ai/api/v1".to_string(),
            model: String::new(),   // User must provide
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}
```

## Telemetry

### Structured Logging

Use `tracing` crate for structured logging:

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self, messages), fields(correlation_id = %correlation_id))]
pub async fn chat(&self, session: &mut ChippSession, messages: &[ChippMessage]) -> Result<String> {
    info!("Sending chat request");
    // ...
}
```

### What to Log

**✅ Log**:

- Request/response latency
- HTTP status codes
- Correlation IDs
- Retry attempts
- Error types

**❌ Do NOT log**:

- API keys
- User messages (privacy)
- Assistant responses (privacy)

### Privacy-First Logging

```rust
// ❌ Bad: Logs user content
info!("User said: {}", message.content);

// ✅ Good: Logs metadata only
info!(message_count = messages.len(), "Sending chat request");
```

## Testing Strategy

### Unit Tests

Test all public APIs, edge cases, error paths:

```rust
#[tokio::test]
async fn test_session_continuity() {
    let mut session = ChippSession::new();
    assert!(session.chat_session_id.is_none());

    session.chat_session_id = Some("test-session-id".to_string());
    assert_eq!(session.chat_session_id.as_deref(), Some("test-session-id"));
}
```

### Integration Tests

Gate behind `integration-tests` feature:

```rust
#[tokio::test]
#[cfg(feature = "integration-tests")]
#[ignore] // Run with: cargo test --features integration-tests -- --ignored
async fn test_real_api_call() {
    let api_key = std::env::var("CHIPP_API_KEY").expect("CHIPP_API_KEY not set");
    let model = std::env::var("CHIPP_APP_NAME_ID").expect("CHIPP_APP_NAME_ID not set");

    let config = ChippConfig {
        api_key,
        model,
        ..Default::default()
    };

    let client = ChippClient::new(config);
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello!".to_string(),
    }];

    let response = client.chat(&mut session, &messages).await.unwrap();
    assert!(!response.is_empty());
}
```

### Doc Tests

All examples in documentation must compile and run:

```rust
/// Send a chat message
///
/// # Example
///
/// ```no_run
/// use chipp::{ChippClient, ChippConfig, ChippMessage, MessageRole};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ChippClient::new(config);
/// let response = client.chat(&mut session, &messages).await?;
/// # Ok(())
/// # }
/// ```
pub async fn chat(&self, session: &mut ChippSession, messages: &[ChippMessage]) -> Result<String> {
    // ...
}
```

## Performance Considerations

### Connection Pooling

`reqwest::Client` handles connection pooling automatically. Reuse the same client instance:

```rust
// ✅ Good: Reuse client
let client = ChippClient::new(config);
for _ in 0..10 {
    client.chat(&mut session, &messages).await?;
}

// ❌ Bad: Create new client each time
for _ in 0..10 {
    let client = ChippClient::new(config.clone());
    client.chat(&mut session, &messages).await?;
}
```

### Timeout Configuration

Allow users to configure timeouts:

```rust
let config = ChippConfig {
    timeout: Duration::from_secs(60), // Longer timeout for complex queries
    ..Default::default()
};
```

### Streaming for Large Responses

Use streaming for better UX with long responses:

```rust
// Non-streaming: Wait for entire response
let response = client.chat(&mut session, &messages).await?;
println!("{}", response);

// Streaming: Show text as it arrives
let mut stream = client.chat_stream(&mut session, &messages).await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```
