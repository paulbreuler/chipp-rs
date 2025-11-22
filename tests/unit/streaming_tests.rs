//! Unit tests for ChippClient::chat_stream() method
//!
//! These tests verify the streaming functionality including:
//! - Successful streaming with multiple chunks
//! - Error handling for API failures
//! - Custom Chipp streaming format parsing

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

/// Tests that chat_stream() successfully streams multiple chunks
///
/// Arrange: Mock server returns streaming response with multiple chunks
/// Act: Call chat_stream() and collect chunks
/// Assert: All chunks are received in order
#[tokio::test]
async fn test_chat_stream_multiple_chunks() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    // Chipp streaming format: 0:"chunk1"\n0:"chunk2"\ne:{...}\nd:{...}
    let stream_body = r#"0:"Hello! "
0:"How "
0:"can "
0:"I "
0:"help "
0:"you?"
e:{"chatSessionId":"session-123"}
d:{"done":true}
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

    assert_eq!(chunks.len(), 6);
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

    let stream_body = r#"0:"Complete response"
e:{"chatSessionId":"session-456"}
d:{"done":true}
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

/// Tests that chat_stream() skips unknown prefixes
///
/// Arrange: Mock server returns streaming response with unknown prefixes
/// Act: Call chat_stream() and collect chunks
/// Assert: Only 0: prefixed chunks are returned
#[tokio::test]
async fn test_chat_stream_unknown_prefix_skipped() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    let stream_body = r#"f:{"some":"metadata"}
0:"First chunk"
8:{"other":"data"}
0:"Second chunk"
e:{"chatSessionId":"session-789"}
d:{"done":true}
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

    // Only the 0: prefixed chunks should be returned
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0], "First chunk");
    assert_eq!(chunks[1], "Second chunk");
}

/// Tests that chat_stream() handles empty response
///
/// Arrange: Mock server returns streaming response with no chunks
/// Act: Call chat_stream() and collect chunks
/// Assert: No chunks are returned
#[tokio::test]
async fn test_chat_stream_empty_response() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    let stream_body = r#"e:{"chatSessionId":"session-empty"}
d:{"done":true}
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

/// Tests that chat_stream() returns error on invalid chunk JSON
///
/// Arrange: Mock server returns streaming response with invalid JSON chunk
/// Act: Call chat_stream() and collect chunks
/// Assert: Returns StreamError for invalid chunk
#[tokio::test]
async fn test_chat_stream_invalid_chunk_json() {
    // Arrange
    let (client, mock_server) = setup_test_client().await;

    let stream_body = r#"0:"Valid chunk"
0:not valid json
e:{"chatSessionId":"session-bad"}
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

    // First chunk should be valid
    let first = stream.next().await;
    assert!(first.is_some());
    assert_eq!(first.unwrap().unwrap(), "Valid chunk");

    // Second chunk should be an error
    let second = stream.next().await;
    assert!(second.is_some());
    match second.unwrap() {
        Err(ChippClientError::StreamError(msg)) => {
            assert!(msg.contains("Failed to parse content chunk"));
        }
        other => panic!("Expected StreamError, got: {:?}", other),
    }
}
