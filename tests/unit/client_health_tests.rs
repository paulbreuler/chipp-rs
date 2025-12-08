//! Tests for ChippClient ping method.

use chipp::{ChippClient, ChippClientError, ChippConfig};
use std::time::Duration;

// ============================================================================
// ping() Tests
// ============================================================================

#[tokio::test]
async fn test_ping_returns_duration_for_successful_request() {
    // ARRANGE
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("HEAD", "/chat/completions")
        .with_status(200)
        .create_async()
        .await;

    let config = ChippConfig::builder()
        .api_key("test-key")
        .model("test-model")
        .base_url(server.url())
        .build()
        .unwrap();

    let client = ChippClient::new(config).unwrap();

    // ACT
    let result = client.ping().await;

    // ASSERT
    assert!(result.is_ok());
    let latency = result.unwrap();
    // Latency should be reasonable (< 5 seconds for local mock server)
    assert!(latency < Duration::from_secs(5));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_ping_returns_err_for_network_failure() {
    // ARRANGE - use invalid URL to simulate network failure
    let config = ChippConfig::builder()
        .api_key("test-key")
        .model("test-model")
        .base_url("http://invalid-domain-that-does-not-exist-12345.com")
        .timeout(Duration::from_millis(100))
        .build()
        .unwrap();

    let client = ChippClient::new(config).unwrap();

    // ACT
    let result = client.ping().await;

    // ASSERT
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ChippClientError::HttpError(_)
    ));
}
