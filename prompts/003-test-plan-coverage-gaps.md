# Test Plan: Coverage Gap Analysis for chipp-rs

## Current Coverage: 53.66% (132/246 lines)

Generated from: `cargo llvm-cov --all-features --workspace --html`

## Existing Test Coverage (11 unit tests)

### ✅ Already Covered

1. **Session Management** (3 tests)
   - `test_session_creation()` - Session starts with no ID
   - `test_session_reset()` - Reset clears session ID
   - `test_message_role_serialization()` - MessageRole serializes correctly

2. **Retry Logic Classification** (6 tests)
   - `test_is_retryable_error_http_timeout()` - HTTP timeout is retryable
   - `test_is_retryable_error_5xx()` - 5xx errors are retryable
   - `test_is_retryable_error_429()` - 429 rate limit is retryable
   - `test_is_not_retryable_error_4xx()` - 4xx errors are NOT retryable
   - `test_is_not_retryable_error_invalid_response()` - Invalid response is NOT retryable
   - `test_is_not_retryable_error_max_retries_exceeded()` - Max retries error is NOT retryable

3. **Configuration** (2 tests)
   - `test_backoff_configuration()` - Backoff uses correct config values
   - `test_config_default_retry_values()` - Default config has correct retry values

### ❌ NOT Covered (Major Gaps)

## Gap Analysis by Function

### 1. `ChippClient::new()` - 0% coverage
**Lines**: 281-288 (8 lines)
**Current tests**: NONE

**Missing test cases:**
- ✅ Valid configuration creates client successfully
- ✅ HTTP client is built with correct timeout
- ⚠️ Edge case: Very short timeout (1ms)
- ⚠️ Edge case: Very long timeout (1 hour)

**Priority**: HIGH (constructor is critical)

---

### 2. `ChippClient::chat()` - ~20% coverage
**Lines**: 358-416 (59 lines)
**Current tests**: Partial (only error classification tested)

**Missing test cases:**
- ✅ Successful chat on first attempt
- ✅ Successful chat after 1 retry (5xx then success)
- ✅ Successful chat after 2 retries (5xx, 5xx, success)
- ✅ Max retries exceeded (all attempts fail with 5xx)
- ✅ Non-retryable error returns immediately (4xx)
- ✅ Backoff delay is applied between retries
- ⚠️ Edge case: max_retries = 0 (no retries)
- ⚠️ Edge case: Backoff exhausted (shouldn't happen with our config)

**Priority**: CRITICAL (core functionality)

---

### 3. `ChippClient::chat_attempt()` - 0% coverage
**Lines**: 419-470 (52 lines)
**Current tests**: NONE (private method, tested via chat())

**Missing test cases:**
- ✅ Successful request with valid response
- ✅ Session ID is updated from response
- ✅ API error (non-2xx status) returns ApiError
- ✅ Invalid JSON response returns InvalidResponse
- ✅ Response with no choices returns InvalidResponse
- ✅ Authorization header is set correctly
- ✅ Content-Type header is set correctly
- ✅ X-Correlation-ID header is set correctly
- ✅ Request body includes model, messages, stream=false
- ✅ Request body includes chat_session_id if present

**Priority**: CRITICAL (core functionality)

---

### 4. `ChippClient::chat_stream()` - 0% coverage
**Lines**: 521-597 (77 lines)
**Current tests**: NONE

**Missing test cases:**
- ✅ Successful streaming with multiple chunks
- ✅ Streaming with single chunk
- ✅ Streaming with empty response
- ✅ Stream parsing: `0:"content"` format
- ✅ Stream parsing: `e:{...}` end marker
- ✅ Stream parsing: `d:{...}` done marker
- ✅ Stream parsing: Skip unknown prefixes (f:, 8:, etc.)
- ✅ API error (non-2xx status) returns ApiError
- ✅ Invalid chunk JSON returns StreamError
- ✅ Authorization header is set correctly
- ✅ Accept: text/event-stream header is set
- ✅ Request body includes stream=true

**Priority**: HIGH (important feature)

---

### 5. `ChippConfig::default()` - 100% coverage ✅
**Lines**: 165-174 (10 lines)
**Current tests**: Covered by `test_config_default_retry_values()`

---

### 6. `ChippSession::default()` - 100% coverage ✅
**Lines**: 217-220 (4 lines)
**Current tests**: Covered by `test_session_creation()`

---

### 7. Error Type Conversions - 0% coverage
**Lines**: Various (From implementations)
**Current tests**: NONE

**Missing test cases:**
- ✅ `From<reqwest::Error>` for `ChippClientError`
- ✅ `Display` for `ChippClientError` (all variants)

**Priority**: MEDIUM

---

## Test Implementation Plan

### Phase 1: Critical Path (Target: 70% coverage)

**Estimated tests needed**: 15-20

1. **ChippClient::new() tests** (2 tests)
   - Valid configuration
   - HTTP client timeout configuration

2. **ChippClient::chat() integration tests** (8 tests)
   - Success on first attempt
   - Success after 1 retry
   - Success after 2 retries
   - Max retries exceeded
   - Non-retryable error immediate return
   - Backoff delay verification
   - max_retries = 0
   - Session ID updated correctly

3. **ChippClient::chat_attempt() tests** (via mocking) (5 tests)
   - Successful request
   - API error handling
   - Invalid JSON response
   - No choices in response
   - Headers set correctly

### Phase 2: Streaming (Target: 80% coverage)

**Estimated tests needed**: 8-10

4. **ChippClient::chat_stream() tests** (8 tests)
   - Multiple chunks
   - Single chunk
   - Empty response
   - End marker handling
   - Done marker handling
   - Unknown prefix skipping
   - API error
   - Invalid chunk JSON

### Phase 3: Edge Cases (Target: 85%+ coverage)

**Estimated tests needed**: 3-5

5. **Error conversion tests** (2 tests)
   - reqwest::Error conversion
   - Display formatting

6. **Edge cases** (3 tests)
   - Very short timeout
   - Very long timeout
   - Backoff exhausted

---

## Testing Strategy

### Mock HTTP Responses

Use `wiremock` for HTTP mocking:

```bash
cargo add --dev wiremock
cargo add --dev tokio-test
```

### Test File Organization

```
src/
└── lib.rs (existing unit tests stay here)

tests/
├── integration_test.rs (existing, keep as-is)
└── unit/
    ├── mod.rs
    ├── client_new_tests.rs       # ChippClient::new()
    ├── chat_tests.rs              # chat() method
    ├── chat_attempt_tests.rs      # chat_attempt() via mocking
    ├── streaming_tests.rs         # chat_stream()
    └── error_tests.rs             # Error conversions
```

### AAA Pattern Template

```rust
/// Tests that [BEHAVIOR]
/// 
/// Arrange: [SETUP]
/// Act: [ACTION]
/// Assert: [EXPECTED OUTCOME]
#[tokio::test]
async fn test_descriptive_name() {
    // Arrange
    let mock_server = MockServer::start().await;
    let config = ChippConfig {
        base_url: mock_server.uri(),
        api_key: "test-key".to_string(),
        model: "test-model".to_string(),
        ..Default::default()
    };
    let client = ChippClient::new(config);
    
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chatSessionId": "session-123",
            "choices": [{
                "message": {
                    "content": "Hello!"
                }
            }]
        })))
        .mount(&mock_server)
        .await;
    
    // Act
    let mut session = ChippSession::new();
    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Hi".to_string(),
    }];
    let result = client.chat(&mut session, &messages).await;
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello!");
    assert_eq!(session.chat_session_id, Some("session-123".to_string()));
}
```

---

## Success Criteria

- [ ] Coverage reaches 80% or higher
- [ ] All critical paths tested (chat, chat_stream, new)
- [ ] All tests follow AAA pattern
- [ ] All tests have clear documentation
- [ ] `just quality` passes
- [ ] `just coverage-check` passes

---

## Estimated Effort

- **Phase 1 (Critical)**: 15-20 tests → ~70% coverage → 3-4 hours
- **Phase 2 (Streaming)**: 8-10 tests → ~80% coverage → 2-3 hours
- **Phase 3 (Edge Cases)**: 3-5 tests → ~85% coverage → 1-2 hours

**Total**: 26-35 tests, 6-9 hours of work

