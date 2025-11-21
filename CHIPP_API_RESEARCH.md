# Chipp API Research Summary

**Research Date**: 2025-11-20  
**Goal**: Document existing Chipp API integration in CEC codebase

---

## Summary

Chipp API integration is **well-documented and partially implemented** in this codebase.

**What exists**:
- ✅ Comprehensive integration rules (`.augment/rules/chipp-api-integration.md`)
- ✅ Working Rust client implementation (`crates/cec-chipp-client/src/lib.rs`)
- ✅ Architecture documentation (`chipp_edge_companion_docs/cloud_architecture.md`)
- ✅ Configuration support (`crates/cec-config/src/lib.rs`)
- ✅ Orchestrator integration (`crates/cec-orchestrator/src/orchestrator.rs`)

**What's missing**:
- ❌ Streaming (SSE) implementation (marked as TODO)
- ❌ Retry logic with exponential backoff
- ❌ Health endpoint ping for connectivity checks
- ❌ Integration tests with real API
- ❌ Example CLI command to test Chipp API

---

## 1. API Documentation

### 1.1 Official Reference
- **URL**: https://chipp.ai/docs/api/reference
- **Base URL**: `https://app.chipp.ai/api/v1`
- **Endpoint**: `POST /chat/completions` (OpenAI-compatible)

### 1.2 Authentication
```
Authorization: Bearer <CHIPP_API_KEY>
```

API key sources (priority order):
1. Environment variable: `CHIPP_API_KEY`
2. Configuration file: `/etc/cec/cec.toml`
3. Keychain storage (via `cec-cli config set`)

**Security**: NEVER hardcode API keys. Always use env vars or secure storage.

### 1.3 Required Headers
```
Authorization: Bearer <CHIPP_API_KEY>
Content-Type: application/json
X-Correlation-ID: <UUID>           # For tracing (always include)
X-Chat-Session-ID: <session_id>    # Optional, for session continuity
```

---

## 2. Request/Response Format

### 2.1 Request Body
```json
{
  "model": "myapp-123",              // Chipp appNameId
  "messages": [
    { "role": "system", "content": "You are a helpful assistant" },
    { "role": "user", "content": "What is Chipp?" }
  ],
  "stream": false,                   // true for SSE streaming
  "chatSessionId": "uuid-string"     // Optional, for conversation continuity
}
```

### 2.2 Response Body (Non-Streaming)
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

### 2.3 Streaming Response (SSE)
**Status**: Not yet implemented (marked as TODO in `crates/cec-chipp-client/src/lib.rs:254`)

Expected format:
```
data: {"chatSessionId":"...","choices":[{"delta":{"content":"Hello"}}]}
data: {"chatSessionId":"...","choices":[{"delta":{"content":" world"}}]}
data: [DONE]
```

---

## 3. Session Management

### 3.1 Session Lifecycle
- **One session per device**: `ChippSession` tracks `chatSessionId`
- **Session creation**: First request returns `chatSessionId` in response
- **Session continuity**: Include `chatSessionId` in subsequent requests
- **Session reset triggers**:
  - Device reboot
  - User command ("new conversation")
  - 30 minutes of inactivity

### 3.2 Implementation
Location: `crates/cec-chipp-client/src/lib.rs:84-109`

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

**Usage pattern**:
1. Create session: `let mut session = ChippSession::new();`
2. First request: `client.chat(&mut session, &messages).await?`
3. Session ID stored: `session.chat_session_id` now contains ID from response
4. Subsequent requests: Same session object maintains continuity

---

## 4. Rust Client Implementation

### 4.1 Location
- **Crate**: `crates/cec-chipp-client`
- **Main file**: `src/lib.rs` (295 lines)
- **Dependencies**: reqwest, serde, anyhow, thiserror, uuid, tracing

### 4.2 API Surface
```rust
pub struct ChippConfig {
    pub api_key: String,
    pub base_url: String,    // Default: https://app.chipp.ai/api/v1
    pub model: String,       // Chipp appNameId (e.g., "myapp-123")
}

pub struct ChippClient {
    http: reqwest::Client,
    config: ChippConfig,
}

impl ChippClient {
    pub fn new(config: ChippConfig) -> Self;
    
    pub async fn chat(
        &self,
        session: &mut ChippSession,
        messages: &[ChippMessage],
    ) -> Result<String, ChippClientError>;
    
    pub async fn chat_stream(
        &self,
        session: &mut ChippSession,
        messages: &[ChippMessage],
    ) -> Result<(), ChippClientError>;  // TODO: Not implemented
}
```

### 4.3 Error Types
```rust
pub enum ChippClientError {
    HttpError(reqwest::Error),
    InvalidResponse(String),
    ApiError { status: u16, message: String },
    StreamingNotImplemented,
}
```

### 4.4 Implementation Status
- ✅ Non-streaming chat completion
- ✅ Session management
- ✅ Error handling
- ✅ Correlation ID tracking
- ✅ Structured logging with `tracing`
- ❌ Streaming (SSE) - marked as TODO
- ❌ Retry logic - not implemented
- ❌ Timeout configuration - hardcoded to 30s

---

## 5. Configuration

### 5.1 Location
`crates/cec-config/src/lib.rs:81-95`

```rust
pub struct ChippConfig {
    pub api_key: Option<String>,      // Optional - local-only mode if None
    pub base_url: String,             // Default: https://app.chipp.ai/api/v1
    pub model: Option<String>,        // appNameId, required when using API
}
```

### 5.2 Configuration File
Expected location: `/etc/cec/cec.toml`

```toml
[chipp]
api_key = "sk-..."              # Or use env var CHIPP_API_KEY
base_url = "https://app.chipp.ai/api/v1"
model = "myapp-123"
```

---

## 6. Orchestrator Integration

### 6.1 Location
`crates/cec-orchestrator/src/orchestrator.rs:474-492`

### 6.2 Usage Pattern
```rust
async fn call_chipp_api(&mut self, text: &str) -> Result<String, OrchestratorError> {
    let (client, session) = match (&self.chipp_client, &mut self.chipp_session) {
        (Some(client), Some(session)) => (client, session),
        _ => {
            warn!("Chipp API not configured, using local LLM");
            return Err(OrchestratorError::ProcessingError(
                "Chipp API not configured".to_string(),
            ));
        }
    };

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: text.to_string(),
    }];

    client.chat(session, &messages).await
        .map_err(|e| OrchestratorError::ProcessingError(e.to_string()))
}
```

---

## 7. Gaps and Missing Features

### 7.1 Streaming (SSE)
**Status**: Marked as TODO in `crates/cec-chipp-client/src/lib.rs:254-264`

**Implementation needed**:
- Parse SSE `data:` lines
- Handle `chat.completion.chunk` format
- Stop on `[DONE]` marker
- Return async stream of chunks

### 7.2 Retry Logic
**Status**: Not implemented

**Requirements** (from `.augment/rules/chipp-api-integration.md:52`):
- Retry transient failures (5xx, network errors)
- Exponential backoff
- Fall back to offline mode on persistent failures

### 7.3 Health Endpoint
**Status**: Not implemented

**Requirements** (from `.augment/rules/chipp-api-integration.md:70`):
- Ping Chipp API health endpoint before routing
- Cache last-known connectivity status
- Use for offline-first routing decisions

### 7.4 Testing
**Status**: Only unit tests for session management

**Missing**:
- Integration tests with real API (or mock server)
- Error path testing (network failures, API errors)
- Session continuity testing across multiple requests

### 7.5 CLI Demo Command
**Status**: Not implemented

**Suggestion**: Add `cec demo chipp` command similar to `cec demo tts` and `cec demo asr`

---

## 8. Next Steps

To complete Chipp API integration:

1. **Get API token** (user is doing this now)
2. **Test existing implementation** with real API
3. **Add CLI demo command**: `cec demo chipp "Hello, Chipp!"`
4. **Implement streaming** if needed for better UX
5. **Add retry logic** for production reliability
6. **Add integration tests** with real API calls
7. **Document appNameId setup** (how to get "myapp-123" from Chipp dashboard)

---

## 9. References

### 9.1 Code Files
- Client: `crates/cec-chipp-client/src/lib.rs`
- Config: `crates/cec-config/src/lib.rs`
- Orchestrator: `crates/cec-orchestrator/src/orchestrator.rs`
- Rules: `.augment/rules/chipp-api-integration.md`

### 9.2 Documentation
- Architecture: `chipp_edge_companion_docs/cloud_architecture.md`
- Software: `chipp_edge_companion_docs/software_architecture.md`
- Official API: https://chipp.ai/docs/api/reference

### 9.3 Dependencies
- `reqwest` - HTTP client
- `serde` / `serde_json` - JSON serialization
- `anyhow` / `thiserror` - Error handling
- `uuid` - Correlation IDs
- `tracing` - Structured logging

