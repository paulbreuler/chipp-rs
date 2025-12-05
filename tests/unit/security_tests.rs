//! Security tests for chipp-rs SDK
//!
//! These tests verify security-critical behaviors:
//! - API key redaction in Debug output
//! - No accidental credential exposure in logs/errors

use chipp::ChippConfig;

/// Tests that ChippConfig's Debug implementation redacts the API key
///
/// SECURITY: API keys must NEVER appear in Debug output to prevent
/// accidental exposure in logs, error messages, or debug prints.
///
/// Arrange: Create config with a known API key
/// Act: Format the config using Debug trait
/// Assert: The API key value is NOT present, "[REDACTED]" is shown instead
#[test]
fn test_config_debug_redacts_api_key() {
    // ARRANGE
    let secret_api_key = "live_super_secret_key_12345";
    let config = ChippConfig {
        api_key: secret_api_key.to_string(),
        model: "test-model".to_string(),
        ..Default::default()
    };

    // ACT
    let debug_output = format!("{:?}", config);

    // ASSERT
    // The actual API key must NOT appear in debug output
    assert!(
        !debug_output.contains(secret_api_key),
        "SECURITY VIOLATION: API key '{}' was exposed in Debug output: {}",
        secret_api_key,
        debug_output
    );

    // The debug output should show redaction
    assert!(
        debug_output.contains("[REDACTED]"),
        "Debug output should show [REDACTED] for api_key, got: {}",
        debug_output
    );
}

/// Tests that ChippConfig's Debug output still shows non-sensitive fields
///
/// Arrange: Create config with known values
/// Act: Format the config using Debug trait
/// Assert: Non-sensitive fields (model, base_url, timeout) are visible
#[test]
fn test_config_debug_shows_non_sensitive_fields() {
    // ARRANGE
    let config = ChippConfig {
        api_key: "secret-key".to_string(),
        model: "my-app-123".to_string(),
        base_url: "https://custom.api.example.com".to_string(),
        ..Default::default()
    };

    // ACT
    let debug_output = format!("{:?}", config);

    // ASSERT - non-sensitive fields should be visible
    assert!(
        debug_output.contains("my-app-123"),
        "Debug output should show model name, got: {}",
        debug_output
    );
    assert!(
        debug_output.contains("https://custom.api.example.com"),
        "Debug output should show base_url, got: {}",
        debug_output
    );
    assert!(
        debug_output.contains("ChippConfig"),
        "Debug output should show struct name, got: {}",
        debug_output
    );
}

/// Tests that ChippConfigBuilder's Debug implementation also redacts API key
///
/// Arrange: Create a builder with API key set
/// Act: Format the builder using Debug trait
/// Assert: The API key value is NOT present
#[test]
fn test_config_builder_debug_redacts_api_key() {
    // ARRANGE
    let secret_api_key = "live_builder_secret_key_67890";
    let builder = ChippConfig::builder().api_key(secret_api_key);

    // ACT
    let debug_output = format!("{:?}", builder);

    // ASSERT
    assert!(
        !debug_output.contains(secret_api_key),
        "SECURITY VIOLATION: API key '{}' was exposed in Builder Debug output: {}",
        secret_api_key,
        debug_output
    );
}
