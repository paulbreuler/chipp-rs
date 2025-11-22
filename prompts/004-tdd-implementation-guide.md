# TDD Implementation Guide: Step-by-Step

## Overview

This guide walks through implementing tests using TDD (Test-Driven Development) with RGR (Red-Green-Refactor) cycle and AAA (Arrange-Act-Assert) pattern.

## Prerequisites

```bash
# Add test dependencies
cargo add --dev wiremock
cargo add --dev tokio-test
cargo add --dev serde_json
```

## Step 1: Clean Up Existing Tests

### Current State

Existing tests in `src/lib.rs` (lines 620-740) are well-structured but could benefit from:
1. More explicit AAA comments
2. Better documentation
3. Grouping related tests

### Action: Refactor Existing Tests

**Goal**: Make existing tests exemplary for new tests to follow

**Changes**:
1. Add AAA comments to each test
2. Add comprehensive doc comments
3. Keep all existing tests (they're good!)
4. Run `cargo test` to ensure nothing breaks

**Example refactor** (for reference, don't change yet):

```rust
/// Tests that ChippSession starts with no chat session ID
/// 
/// Arrange: Create new session
/// Act: Check chat_session_id field
/// Assert: Field is None
#[test]
fn test_session_creation() {
    // Arrange & Act
    let session = ChippSession::new();
    
    // Assert
    assert!(session.chat_session_id.is_none());
}
```

---

## Step 2: Set Up Test Infrastructure

### Create Test Module Structure

```bash
mkdir -p tests/unit
touch tests/unit/mod.rs
touch tests/unit/client_new_tests.rs
touch tests/unit/chat_tests.rs
```

### tests/unit/mod.rs

```rust
//! Unit tests for chipp-rs SDK
//! 
//! Tests are organized by functionality:
//! - client_new_tests: ChippClient::new() constructor tests
//! - chat_tests: ChippClient::chat() method tests
//! - streaming_tests: ChippClient::chat_stream() method tests

mod client_new_tests;
mod chat_tests;
```

---

## Step 3: First TDD Cycle - ChippClient::new()

### Test 1: Valid Configuration Creates Client

**RED**: Write failing test

```rust
// tests/unit/client_new_tests.rs

use chipp::{ChippClient, ChippConfig};
use std::time::Duration;

/// Tests that ChippClient::new() successfully creates a client with valid configuration
/// 
/// Arrange: Create valid ChippConfig
/// Act: Call ChippClient::new()
/// Assert: Client is created (no panic)
#[test]
fn test_new_with_valid_config_creates_client() {
    // Arrange
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        base_url: "https://test.example.com/api/v1".to_string(),
        model: "test-model".to_string(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
        initial_retry_delay: Duration::from_millis(100),
        max_retry_delay: Duration::from_secs(10),
    };
    
    // Act
    let client = ChippClient::new(config);
    
    // Assert
    // If we get here without panic, test passes
    // We can't inspect private fields, but we can verify the client exists
    assert_eq!(client.config.api_key, "test-api-key");
    assert_eq!(client.config.model, "test-model");
}
```

**Run test**:
```bash
cargo test test_new_with_valid_config_creates_client
```

**Expected**: ❌ FAIL (client.config is private)

**GREEN**: Fix the test

```rust
#[test]
fn test_new_with_valid_config_creates_client() {
    // Arrange
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        base_url: "https://test.example.com/api/v1".to_string(),
        model: "test-model".to_string(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
        initial_retry_delay: Duration::from_millis(100),
        max_retry_delay: Duration::from_secs(10),
    };
    
    // Act
    let _client = ChippClient::new(config);
    
    // Assert
    // Client created successfully (no panic)
}
```

**Run test**:
```bash
cargo test test_new_with_valid_config_creates_client
```

**Expected**: ✅ PASS

**REFACTOR**: Simplify using Default

```rust
/// Tests that ChippClient::new() successfully creates a client with valid configuration
/// 
/// Arrange: Create valid ChippConfig using defaults
/// Act: Call ChippClient::new()
/// Assert: Client is created without panic
#[test]
fn test_new_with_valid_config_creates_client() {
    // Arrange
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        model: "test-model".to_string(),
        ..Default::default()
    };
    
    // Act
    let _client = ChippClient::new(config);
    
    // Assert - Client created successfully (no panic)
}
```

**Run test**:
```bash
cargo test test_new_with_valid_config_creates_client
```

**Expected**: ✅ PASS

---

## Step 4: Second TDD Cycle - ChippClient::chat() Success

### Test 2: Successful Chat on First Attempt

**RED**: Write test with mock server

```rust
// tests/unit/chat_tests.rs

use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage, MessageRole};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header, body_json};
use serde_json::json;

/// Tests that chat() succeeds on first attempt with valid API response
/// 
/// Arrange: Mock server returns successful response, create client and session
/// Act: Call chat() with test message
/// Assert: Returns expected content and updates session ID
#[tokio::test]
async fn test_chat_succeeds_on_first_attempt() {
    // Arrange
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("Authorization", "Bearer test-api-key"))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chatSessionId": "session-123",
            "choices": [{
                "message": {
                    "content": "Hello! How can I help you?"
                }
            }]
        })))
        .mount(&mock_server)
        .await;
    
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        base_url: mock_server.uri(),
        model: "test-model".to_string(),
        ..Default::default()
    };
    let client = ChippClient::new(config);
    let mut session = ChippSession::new();
    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello".to_string(),
    }];
    
    // Act
    let result = client.chat(&mut session, &messages).await;
    
    // Assert
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let content = result.unwrap();
    assert_eq!(content, "Hello! How can I help you?");
    assert_eq!(session.chat_session_id, Some("session-123".to_string()));
}
```

**Run test**:
```bash
cargo test test_chat_succeeds_on_first_attempt
```

**Expected**: ✅ PASS (code already works!)

**REFACTOR**: Extract common setup

```rust
// tests/unit/chat_tests.rs

use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage, MessageRole};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};
use serde_json::json;

/// Helper to create test client with mock server
async fn setup_test_client() -> (ChippClient, MockServer) {
    let mock_server = MockServer::start().await;
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        base_url: mock_server.uri(),
        model: "test-model".to_string(),
        ..Default::default()
    };
    let client = ChippClient::new(config);
    (client, mock_server)
}

/// Helper to create test messages
fn create_test_messages() -> Vec<ChippMessage> {
    vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello".to_string(),
    }]
}

/// Tests that chat() succeeds on first attempt with valid API response
/// 
/// Arrange: Mock server returns successful response
/// Act: Call chat() with test message
/// Assert: Returns expected content and updates session ID
#[tokio::test]
async fn test_chat_succeeds_on_first_attempt() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;
    
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chatSessionId": "session-123",
            "choices": [{
                "message": {
                    "content": "Hello! How can I help you?"
                }
            }]
        })))
        .mount(&mock_server)
        .await;
    
    let mut session = ChippSession::new();
    let messages = create_test_messages();
    
    // Act
    let result = client.chat(&mut session, &messages).await;
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello! How can I help you?");
    assert_eq!(session.chat_session_id, Some("session-123".to_string()));
}
```

---

## Step 5: Continue TDD Cycles

### Recommended Order

1. ✅ `test_new_with_valid_config_creates_client`
2. ✅ `test_chat_succeeds_on_first_attempt`
3. ⬜ `test_chat_succeeds_after_one_retry`
4. ⬜ `test_chat_max_retries_exceeded`
5. ⬜ `test_chat_non_retryable_error_immediate_return`
6. ⬜ `test_chat_updates_session_id`
7. ⬜ `test_chat_api_error_returns_error`
8. ⬜ `test_chat_invalid_json_returns_error`
9. ⬜ `test_chat_no_choices_returns_error`
10. ⬜ `test_chat_stream_multiple_chunks`

### After Each Test

```bash
# Run the specific test
cargo test test_name

# Check coverage
just coverage

# Run all tests
cargo test

# Run quality checks
just quality
```

---

## Tips for Success

1. **One test at a time** - Don't write multiple tests before running them
2. **Run tests frequently** - After every change
3. **Keep tests simple** - Test one behavior per test
4. **Use descriptive names** - Test name should describe the behavior
5. **Document with AAA** - Make it clear what's being tested
6. **Refactor ruthlessly** - Extract common setup, remove duplication
7. **Check coverage** - Use `just coverage` to see what's covered

---

## Common Patterns

### Testing Retry Logic

```rust
#[tokio::test]
async fn test_chat_succeeds_after_one_retry() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;
    
    // First attempt fails with 500
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;
    
    // Second attempt succeeds
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chatSessionId": "session-123",
            "choices": [{"message": {"content": "Success!"}}]
        })))
        .mount(&mock_server)
        .await;
    
    // Act & Assert
    let mut session = ChippSession::new();
    let result = client.chat(&mut session, &create_test_messages()).await;
    assert!(result.is_ok());
}
```

### Testing Error Cases

```rust
#[tokio::test]
async fn test_chat_api_error_returns_error() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;
    
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
        .mount(&mock_server)
        .await;
    
    // Act
    let mut session = ChippSession::new();
    let result = client.chat(&mut session, &create_test_messages()).await;
    
    // Assert
    assert!(result.is_err());
    match result.unwrap_err() {
        ChippClientError::ApiError { status, message } => {
            assert_eq!(status, 400);
            assert_eq!(message, "Bad Request");
        }
        _ => panic!("Expected ApiError"),
    }
}
```

---

## Next Steps

1. Review this guide
2. Set up test infrastructure (Step 2)
3. Implement first test (Step 3)
4. Continue with remaining tests (Step 4-5)
5. Monitor coverage with `just coverage`
6. Celebrate when you hit 80%! 🎉

