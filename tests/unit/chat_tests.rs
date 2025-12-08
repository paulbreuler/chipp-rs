//! Unit tests for ChippClient::chat() and chat_detailed() methods
//!
//! These tests verify the chat functionality including:
//! - Successful API calls
//! - Retry logic with exponential backoff
//! - Error handling for various failure modes
//! - Session management
//! - Token usage tracking (chat_detailed)

use chipp::{
    ChatResponse, ChippClient, ChippClientError, ChippConfig, ChippMessage, ChippSession,
    MessageRole, Usage,
};
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create test client with mock server
async fn setup_test_client() -> (ChippClient, MockServer) {
    let mock_server = MockServer::start().await;
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        base_url: mock_server.uri(),
        model: "test-model".to_string(),
        timeout: Duration::from_secs(5),
        max_retries: 3,
        initial_retry_delay: Duration::from_millis(10), // Fast retries for tests
        max_retry_delay: Duration::from_millis(100),
    };
    let client = ChippClient::new(config).expect("Failed to create test client");
    (client, mock_server)
}

/// Helper to create test messages
fn create_test_messages() -> Vec<ChippMessage> {
    vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello".to_string(),
    }]
}

/// Helper to create successful API response with all fields
/// This matches the real Chipp API response structure
fn create_full_response(
    content: &str,
    session_id: &str,
    completion_id: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
) -> serde_json::Value {
    json!({
        "chatSessionId": session_id,
        "id": completion_id,
        "object": "chat.completion",
        "created": 1234567890,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens
        }
    })
}

/// Helper to create successful API response (backward compatible helper)
fn create_success_response(content: &str, session_id: &str) -> serde_json::Value {
    create_full_response(content, session_id, "chatcmpl-test123", 10, 5)
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
        .and(header("Authorization", "Bearer test-api-key"))
        .and(header("Content-Type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(create_success_response(
                "Hello! How can I help you?",
                "session-123",
            )),
        )
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), "Hello! How can I help you?");
    assert_eq!(session.chat_session_id, Some("session-123".to_string()));
}

/// Tests that chat() succeeds after one retry (500 then 200)
///
/// Arrange: Mock server fails once with 500, then succeeds
/// Act: Call chat() with test message
/// Assert: Returns success after retry
#[tokio::test]
async fn test_chat_succeeds_after_one_retry() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // First attempt fails with 500
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Second attempt succeeds
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(create_success_response("Success!", "session-456")),
        )
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok(), "Expected Ok after retry, got: {:?}", result);
    assert_eq!(result.unwrap(), "Success!");
    assert_eq!(session.chat_session_id, Some("session-456".to_string()));
}

/// Tests that chat() succeeds after two retries (500, 500, 200)
///
/// Arrange: Mock server fails twice with 500, then succeeds
/// Act: Call chat() with test message
/// Assert: Returns success after two retries
#[tokio::test]
async fn test_chat_succeeds_after_two_retries() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // First two attempts fail with 500
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    // Third attempt succeeds
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(create_success_response("Finally!", "session-789")),
        )
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat(&mut session, &messages).await;

    // Assert
    assert!(
        result.is_ok(),
        "Expected Ok after two retries, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Finally!");
    assert_eq!(session.chat_session_id, Some("session-789".to_string()));
}

/// Tests that chat() fails when max retries exceeded (all 500s)
///
/// Arrange: Mock server always returns 500
/// Act: Call chat() with test message
/// Assert: Returns MaxRetriesExceeded error
#[tokio::test]
async fn test_chat_max_retries_exceeded() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // All attempts fail with 500
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat(&mut session, &messages).await;

    // Assert
    assert!(result.is_err(), "Expected Err, got: {:?}", result);
    match result.unwrap_err() {
        ChippClientError::MaxRetriesExceeded(max_retries) => {
            assert_eq!(max_retries, 3); // max_retries config value
        }
        other => panic!("Expected MaxRetriesExceeded, got: {:?}", other),
    }
}

/// Tests that chat() returns immediately on non-retryable 4xx error
///
/// Arrange: Mock server returns 400 Bad Request
/// Act: Call chat() with test message
/// Assert: Returns ApiError immediately without retry
#[tokio::test]
async fn test_chat_non_retryable_error_immediate_return() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
        .expect(1) // Should only be called once (no retries)
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat(&mut session, &messages).await;

    // Assert
    assert!(result.is_err(), "Expected Err, got: {:?}", result);
    match result.unwrap_err() {
        ChippClientError::ApiError { status, message } => {
            assert_eq!(status, 400);
            assert_eq!(message, "Bad Request");
        }
        other => panic!("Expected ApiError, got: {:?}", other),
    }
}

/// Tests that chat() updates session ID from API response
///
/// Arrange: Mock server returns response with new session ID
/// Act: Call chat() twice with same session
/// Assert: Session ID is updated after each call
#[tokio::test]
async fn test_chat_updates_session_id() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // First call returns session-1
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(create_success_response("First", "session-1")),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Second call returns session-2
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(create_success_response("Second", "session-2")),
        )
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act & Assert - First call
    let result1 = client.chat(&mut session, &messages).await;
    assert!(result1.is_ok());
    assert_eq!(session.chat_session_id, Some("session-1".to_string()));

    // Act & Assert - Second call
    let result2 = client.chat(&mut session, &messages).await;
    assert!(result2.is_ok());
    assert_eq!(session.chat_session_id, Some("session-2".to_string()));
}

/// Tests that chat() returns error when API returns invalid JSON
///
/// Arrange: Mock server returns 200 with invalid JSON
/// Act: Call chat() with test message
/// Assert: Returns InvalidResponse error
#[tokio::test]
async fn test_chat_invalid_json_returns_error() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat(&mut session, &messages).await;

    // Assert
    assert!(result.is_err(), "Expected Err, got: {:?}", result);
    match result.unwrap_err() {
        ChippClientError::InvalidResponse(msg) => {
            assert!(msg.contains("Failed to parse response"));
        }
        other => panic!("Expected InvalidResponse, got: {:?}", other),
    }
}

/// Tests that chat() returns error when API returns empty choices array
///
/// Arrange: Mock server returns response with no choices
/// Act: Call chat() with test message
/// Assert: Returns InvalidResponse error
#[tokio::test]
async fn test_chat_no_choices_returns_error() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // Response with all required fields but empty choices array
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chatSessionId": "session-123",
            "id": "chatcmpl-empty",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "test-model",
            "choices": [],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 0,
                "total_tokens": 10
            }
        })))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat(&mut session, &messages).await;

    // Assert
    assert!(result.is_err(), "Expected Err, got: {:?}", result);
    match result.unwrap_err() {
        ChippClientError::InvalidResponse(msg) => {
            assert!(
                msg.contains("No choices"),
                "Expected 'No choices' in error, got: {}",
                msg
            );
        }
        other => panic!("Expected InvalidResponse, got: {:?}", other),
    }
}

/// Tests that chat() returns error when API returns missing message content
///
/// Arrange: Mock server returns response with no message content
/// Act: Call chat() with test message
/// Assert: Returns InvalidResponse error
#[tokio::test]
async fn test_chat_missing_content_returns_error() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chatSessionId": "session-123",
            "choices": [{
                "message": {}
            }]
        })))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat(&mut session, &messages).await;

    // Assert
    assert!(result.is_err(), "Expected Err, got: {:?}", result);
    match result.unwrap_err() {
        ChippClientError::InvalidResponse(msg) => {
            // Serde error message contains "missing field `content`"
            assert!(
                msg.contains("missing field") || msg.contains("Failed to parse"),
                "Unexpected error message: {}",
                msg
            );
        }
        other => panic!("Expected InvalidResponse, got: {:?}", other),
    }
}

// =============================================================================
// chat_detailed() Tests - Token Usage and Full Response
// =============================================================================

/// Tests that chat_detailed() returns full response with token usage
///
/// Arrange: Mock server returns successful response with usage data
/// Act: Call chat_detailed() with test message
/// Assert: Returns ChatResponse with all fields populated
#[tokio::test]
async fn test_chat_detailed_returns_full_response() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(create_full_response(
                "Hello! I'm here to help.",
                "session-abc",
                "chatcmpl-xyz789",
                100,
                25,
            )),
        )
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat_detailed(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let response: ChatResponse = result.unwrap();

    // Verify content
    assert_eq!(response.content(), "Hello! I'm here to help.");

    // Verify session management
    assert_eq!(response.session_id(), "session-abc");
    assert_eq!(session.chat_session_id, Some("session-abc".to_string()));

    // Verify token usage
    assert_eq!(response.usage().prompt_tokens, 100);
    assert_eq!(response.usage().completion_tokens, 25);
    assert_eq!(response.usage().total_tokens, 125);

    // Verify metadata
    assert_eq!(response.completion_id(), "chatcmpl-xyz789");
    assert_eq!(response.created_at(), 1234567890);
    assert_eq!(response.finish_reason(), "stop");
    assert_eq!(response.model(), "test-model");
}

/// Tests that chat_detailed() tracks token usage for monitoring
///
/// Arrange: Mock server returns response with specific token counts
/// Act: Call chat_detailed()
/// Assert: Token counts are correctly captured
#[tokio::test]
async fn test_chat_detailed_token_usage_tracking() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(create_full_response(
                "Response text",
                "session-123",
                "chatcmpl-abc",
                8751,
                62,
            )),
        )
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let response = client
        .chat_detailed(&mut session, &messages)
        .await
        .expect("Should succeed");

    // Assert
    let usage: &Usage = response.usage();
    assert_eq!(usage.prompt_tokens, 8751);
    assert_eq!(usage.completion_tokens, 62);
    assert_eq!(usage.total_tokens, 8813);
}

/// Tests that chat_detailed() retries on transient failures
///
/// Arrange: Mock server fails once with 500, then succeeds
/// Act: Call chat_detailed() with test message
/// Assert: Returns success after retry with full response
#[tokio::test]
async fn test_chat_detailed_retries_on_failure() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // First attempt fails with 500
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Second attempt succeeds
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(create_full_response(
                "Success after retry!",
                "session-retry",
                "chatcmpl-retry456",
                50,
                10,
            )),
        )
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat_detailed(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok(), "Expected Ok after retry, got: {:?}", result);
    let response = result.unwrap();
    assert_eq!(response.content(), "Success after retry!");
    assert_eq!(response.usage().total_tokens, 60);
}

/// Tests that chat() still works and returns just content (backward compatibility)
///
/// Arrange: Mock server returns full response
/// Act: Call chat() (not chat_detailed())
/// Assert: Returns just the content string, not full response
#[tokio::test]
async fn test_chat_backward_compatibility() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(create_full_response(
                "Simple response",
                "session-compat",
                "chatcmpl-compat",
                20,
                5,
            )),
        )
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act - using chat() not chat_detailed()
    let result = client.chat(&mut session, &messages).await;

    // Assert - returns String, not ChatResponse
    assert!(result.is_ok());
    let content: String = result.unwrap();
    assert_eq!(content, "Simple response");

    // Session should still be updated
    assert_eq!(session.chat_session_id, Some("session-compat".to_string()));
}
