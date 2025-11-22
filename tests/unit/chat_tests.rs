//! Unit tests for ChippClient::chat() method
//!
//! These tests verify the chat functionality including:
//! - Successful API calls
//! - Retry logic with exponential backoff
//! - Error handling for various failure modes
//! - Session management

use chipp::{ChippClient, ChippClientError, ChippConfig, ChippMessage, ChippSession, MessageRole};
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

/// Helper to create successful API response
fn create_success_response(content: &str, session_id: &str) -> serde_json::Value {
    json!({
        "chatSessionId": session_id,
        "choices": [{
            "message": {
                "content": content
            }
        }]
    })
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

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chatSessionId": "session-123",
            "choices": []
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
            assert!(msg.contains("No choices"));
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
