//! Unit tests for ChippClient::chat_stream() method
//!
//! These tests verify the streaming functionality including:
//! - Successful streaming with multiple chunks
//! - Error handling for API failures
//! - Chipp SSE streaming format parsing (data: JSON events)

use chipp::{ChippClient, ChippClientError, ChippConfig, ChippMessage, ChippSession, MessageRole};
use futures::StreamExt;
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
        initial_retry_delay: Duration::from_millis(10),
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

/// Tests that chat_stream() successfully streams multiple chunks
///
/// Arrange: Mock server returns SSE streaming response with text-delta events
/// Act: Call chat_stream() and collect chunks
/// Assert: All chunks are received in order
#[tokio::test]
async fn test_chat_stream_multiple_chunks() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // Chipp SSE streaming format: data: JSON events with type field
    let stream_body = r#"data: {"type":"start","messageId":"msg123"}

data: {"type":"text-delta","id":"msg123","delta":"Hello! "}

data: {"type":"text-delta","id":"msg123","delta":"How "}

data: {"type":"text-delta","id":"msg123","delta":"can "}

data: {"type":"text-delta","id":"msg123","delta":"I "}

data: {"type":"text-delta","id":"msg123","delta":"help "}

data: {"type":"text-delta","id":"msg123","delta":"you?"}

data: {"type":"message-metadata","messageMetadata":{"annotations":[{"persistedMessageId":"session-123"}]}}

data: [DONE]
"#;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("Authorization", "Bearer test-api-key"))
        .and(header("Accept", "text/event-stream"))
        .respond_with(ResponseTemplate::new(200).set_body_string(stream_body))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat_stream(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let mut stream = result.unwrap();

    let mut chunks = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        assert!(
            chunk_result.is_ok(),
            "Expected Ok chunk, got: {:?}",
            chunk_result
        );
        chunks.push(chunk_result.unwrap());
    }

    assert_eq!(chunks.len(), 6, "Expected 6 chunks, got {:?}", chunks);
    assert_eq!(chunks[0], "Hello! ");
    assert_eq!(chunks[1], "How ");
    assert_eq!(chunks[2], "can ");
    assert_eq!(chunks[3], "I ");
    assert_eq!(chunks[4], "help ");
    assert_eq!(chunks[5], "you?");
}

/// Tests that chat_stream() handles single chunk correctly
///
/// Arrange: Mock server returns streaming response with one chunk
/// Act: Call chat_stream() and collect chunks
/// Assert: Single chunk is received
#[tokio::test]
async fn test_chat_stream_single_chunk() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    let stream_body = r#"data: {"type":"text-delta","id":"msg456","delta":"Complete response"}

data: {"type":"message-metadata","messageMetadata":{"annotations":[{"persistedMessageId":"session-456"}]}}

data: [DONE]
"#;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(stream_body))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat_stream(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok());
    let mut stream = result.unwrap();

    let mut chunks = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        chunks.push(chunk_result.unwrap());
    }

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], "Complete response");
}

/// Tests that chat_stream() returns error on API failure
///
/// Arrange: Mock server returns 500 error
/// Act: Call chat_stream()
/// Assert: Returns ApiError
#[tokio::test]
async fn test_chat_stream_api_error() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat_stream(&mut session, &messages).await;

    // Assert
    assert!(result.is_err(), "Expected Err, got: {:?}", result);
    match result.unwrap_err() {
        ChippClientError::ApiError { status, message } => {
            assert_eq!(status, 500);
            assert_eq!(message, "Internal Server Error");
        }
        other => panic!("Expected ApiError, got: {:?}", other),
    }
}

/// Tests that chat_stream() skips non-text-delta events
///
/// Arrange: Mock server returns streaming response with various event types
/// Act: Call chat_stream() and collect chunks
/// Assert: Only text-delta events are returned
#[tokio::test]
async fn test_chat_stream_unknown_prefix_skipped() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    let stream_body = r#"data: {"type":"start","messageId":"msg789"}

data: {"type":"start-step"}

data: {"type":"text-delta","id":"msg789","delta":"First chunk"}

data: {"type":"text-end","id":"msg789"}

data: {"type":"text-delta","id":"msg789","delta":"Second chunk"}

data: {"type":"finish","finishReason":"stop"}

data: [DONE]
"#;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(stream_body))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat_stream(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok());
    let mut stream = result.unwrap();

    let mut chunks = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        chunks.push(chunk_result.unwrap());
    }

    // Only text-delta events should be returned
    assert_eq!(chunks.len(), 2, "Expected 2 chunks, got {:?}", chunks);
    assert_eq!(chunks[0], "First chunk");
    assert_eq!(chunks[1], "Second chunk");
}

/// Tests that chat_stream() handles empty response
///
/// Arrange: Mock server returns streaming response with no text-delta events
/// Act: Call chat_stream() and collect chunks
/// Assert: No chunks are returned
#[tokio::test]
async fn test_chat_stream_empty_response() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    let stream_body = r#"data: {"type":"start","messageId":"empty"}

data: {"type":"finish","finishReason":"stop"}

data: [DONE]
"#;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(stream_body))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat_stream(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok());
    let mut stream = result.unwrap();

    let mut chunks = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        chunks.push(chunk_result.unwrap());
    }

    assert_eq!(chunks.len(), 0);
}

/// Tests that chat_stream() handles malformed JSON gracefully
///
/// Arrange: Mock server returns streaming response with malformed JSON
/// Act: Call chat_stream() and collect chunks
/// Assert: Malformed lines are skipped (not errors)
#[tokio::test]
async fn test_chat_stream_invalid_chunk_json() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // Malformed JSON lines are skipped by the parser
    let stream_body = r#"data: {"type":"text-delta","id":"msg","delta":"Valid chunk"}

data: not valid json at all

data: {"type":"text-delta","id":"msg","delta":"Another valid"}

data: [DONE]
"#;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(stream_body))
        .mount(&mock_server)
        .await;

    let mut session = ChippSession::new();
    let messages = create_test_messages();

    // Act
    let result = client.chat_stream(&mut session, &messages).await;

    // Assert
    assert!(result.is_ok());
    let mut stream = result.unwrap();

    let mut chunks = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        // All chunks should be Ok (malformed JSON is skipped)
        chunks.push(chunk_result.unwrap());
    }

    // Both valid chunks should be returned
    assert_eq!(chunks.len(), 2, "Expected 2 chunks, got {:?}", chunks);
    assert_eq!(chunks[0], "Valid chunk");
    assert_eq!(chunks[1], "Another valid");
}
