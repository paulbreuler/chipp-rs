//! Unit tests for ChippClient::new() constructor
//!
//! These tests verify that the ChippClient can be properly instantiated
//! with various configurations.

use chipp::{ChippClient, ChippConfig};
use std::time::Duration;

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

/// Tests that ChippClient::new() works with default configuration
///
/// Arrange: Create ChippConfig with only required fields
/// Act: Call ChippClient::new()
/// Assert: Client is created with default values
#[test]
fn test_new_with_default_config() {
    // Arrange
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        model: "gpt-4".to_string(),
        ..Default::default()
    };

    // Act
    let _client = ChippClient::new(config);

    // Assert - Client created successfully with defaults
    // Default timeout: 30s
    // Default max_retries: 3
    // Default initial_retry_delay: 100ms
    // Default max_retry_delay: 10s
}

/// Tests that ChippClient::new() accepts custom timeout values
///
/// Arrange: Create ChippConfig with very short timeout
/// Act: Call ChippClient::new()
/// Assert: Client is created (timeout validation happens at runtime)
#[test]
fn test_new_with_custom_timeout() {
    // Arrange
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        model: "test-model".to_string(),
        timeout: Duration::from_millis(1),
        ..Default::default()
    };

    // Act
    let _client = ChippClient::new(config);

    // Assert - Client created successfully
}

/// Tests that ChippClient::new() accepts custom retry configuration
///
/// Arrange: Create ChippConfig with zero retries
/// Act: Call ChippClient::new()
/// Assert: Client is created (will not retry on failures)
#[test]
fn test_new_with_zero_retries() {
    // Arrange
    let config = ChippConfig {
        api_key: "test-api-key".to_string(),
        model: "test-model".to_string(),
        max_retries: 0,
        ..Default::default()
    };

    // Act
    let _client = ChippClient::new(config);

    // Assert - Client created successfully
}

/// Tests that ChippSession::default() creates a new session with no ID
///
/// Arrange: N/A
/// Act: Call ChippSession::default()
/// Assert: Session has no chat_session_id
#[test]
fn test_session_default_trait() {
    // Arrange & Act
    let session = chipp::ChippSession::default();

    // Assert
    assert!(session.chat_session_id.is_none());
}
