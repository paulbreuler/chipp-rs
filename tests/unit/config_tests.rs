//! Tests for ChippConfig and ChippConfigBuilder.

use chipp::{ChippClientError, ChippConfig};
use std::time::Duration;

// ============================================================================
// ChippConfigBuilder Tests
// ============================================================================

#[test]
fn test_builder_with_required_fields_only() {
    let config = ChippConfig::builder()
        .api_key("test-key")
        .model("my-app")
        .build()
        .unwrap();

    assert_eq!(config.api_key, "test-key");
    assert_eq!(config.model, "my-app");
    // Defaults should be applied
    assert_eq!(config.base_url, "https://app.chipp.ai/api/v1");
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.max_retries, 3);
}

#[test]
fn test_builder_missing_api_key_returns_error() {
    let result = ChippConfig::builder().model("my-app").build();

    assert!(result.is_err());
    match result.unwrap_err() {
        ChippClientError::ConfigError(msg) => {
            assert!(msg.contains("api_key is required"));
        }
        _ => panic!("Expected ConfigError"),
    }
}

#[test]
fn test_builder_missing_model_returns_error() {
    let result = ChippConfig::builder().api_key("test-key").build();

    assert!(result.is_err());
    match result.unwrap_err() {
        ChippClientError::ConfigError(msg) => {
            assert!(msg.contains("model is required"));
        }
        _ => panic!("Expected ConfigError"),
    }
}

#[test]
fn test_builder_with_custom_base_url() {
    let config = ChippConfig::builder()
        .api_key("key")
        .model("app")
        .base_url("https://custom.api.com")
        .build()
        .unwrap();

    assert_eq!(config.base_url, "https://custom.api.com");
}

#[test]
fn test_builder_with_custom_timeout() {
    let config = ChippConfig::builder()
        .api_key("key")
        .model("app")
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();

    assert_eq!(config.timeout, Duration::from_secs(60));
}

#[test]
fn test_builder_with_custom_max_retries() {
    let config = ChippConfig::builder()
        .api_key("key")
        .model("app")
        .max_retries(5)
        .build()
        .unwrap();

    assert_eq!(config.max_retries, 5);
}

#[test]
fn test_builder_with_custom_initial_retry_delay() {
    let config = ChippConfig::builder()
        .api_key("key")
        .model("app")
        .initial_retry_delay(Duration::from_millis(500))
        .build()
        .unwrap();

    assert_eq!(config.initial_retry_delay, Duration::from_millis(500));
}

#[test]
fn test_builder_with_custom_max_retry_delay() {
    let config = ChippConfig::builder()
        .api_key("key")
        .model("app")
        .max_retry_delay(Duration::from_secs(30))
        .build()
        .unwrap();

    assert_eq!(config.max_retry_delay, Duration::from_secs(30));
}

#[test]
fn test_builder_with_all_options() {
    let config = ChippConfig::builder()
        .api_key("full-key")
        .model("full-app")
        .base_url("https://full.api.com")
        .timeout(Duration::from_secs(120))
        .max_retries(10)
        .initial_retry_delay(Duration::from_millis(200))
        .max_retry_delay(Duration::from_secs(60))
        .build()
        .unwrap();

    assert_eq!(config.api_key, "full-key");
    assert_eq!(config.model, "full-app");
    assert_eq!(config.base_url, "https://full.api.com");
    assert_eq!(config.timeout, Duration::from_secs(120));
    assert_eq!(config.max_retries, 10);
    assert_eq!(config.initial_retry_delay, Duration::from_millis(200));
    assert_eq!(config.max_retry_delay, Duration::from_secs(60));
}

// ============================================================================
// ChippConfig Tests
// ============================================================================

#[test]
fn test_config_default_values() {
    let config = ChippConfig::default();

    assert_eq!(config.api_key, "");
    assert_eq!(config.model, "");
    assert_eq!(config.base_url, "https://app.chipp.ai/api/v1");
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_retry_delay, Duration::from_millis(100));
    assert_eq!(config.max_retry_delay, Duration::from_secs(10));
}

#[test]
fn test_config_clone() {
    let config = ChippConfig::builder()
        .api_key("clone-key")
        .model("clone-app")
        .build()
        .unwrap();

    let cloned = config.clone();

    assert_eq!(config.api_key, cloned.api_key);
    assert_eq!(config.model, cloned.model);
}
