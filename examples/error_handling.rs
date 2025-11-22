//! Comprehensive error handling example
//!
//! This example demonstrates how to handle various error scenarios when using the Chipp SDK:
//! - Network failures (timeouts, connection errors)
//! - API errors (4xx, 5xx status codes)
//! - Invalid API keys (401 Unauthorized)
//! - Rate limiting (429 Too Many Requests)
//! - Timeout errors
//! - Invalid responses
//! - Error recovery strategies (retry, fallback, graceful degradation)
//!
//! Run with:
//! ```bash
//! export CHIPP_API_KEY="your-api-key"
//! export CHIPP_APP_NAME_ID="your-app-name-id"
//! cargo run --example error_handling
//! ```

use chipp::{ChippClient, ChippClientError, ChippConfig, ChippMessage, ChippSession, MessageRole};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing to see retry attempts and errors
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔍 Chipp SDK Error Handling Examples\n");
    println!("This example demonstrates various error scenarios and recovery strategies.\n");

    // Example 1: Invalid API Key (401 Unauthorized)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 1: Invalid API Key");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    handle_invalid_api_key().await;

    // Example 2: Network Timeout
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 2: Network Timeout");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    handle_timeout().await;

    // Example 3: Successful Request with Retry Logic
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 3: Successful Request (with automatic retry on transient errors)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    handle_successful_request().await;

    // Example 4: Error Recovery with Fallback
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 4: Error Recovery with Fallback");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    handle_with_fallback().await;

    // Example 5: Proper Error Propagation with ? Operator
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 5: Error Propagation with ? Operator");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    if let Err(e) = propagate_errors().await {
        println!("❌ Error propagated to main: {}", e);
        println!("   Error type: {:?}", classify_error(&e));
    }

    println!("\n✅ All error handling examples completed!\n");

    Ok(())
}

/// Example 1: Handle invalid API key (401 Unauthorized)
///
/// This demonstrates:
/// - How 401 errors are NOT retried (client errors are not retryable)
/// - How to detect authentication errors
/// - How to provide helpful error messages to users
async fn handle_invalid_api_key() {
    let config = ChippConfig {
        api_key: "invalid-api-key".to_string(),
        model: "test-app".to_string(),
        max_retries: 2, // Won't retry 401 errors
        ..Default::default()
    };

    let client = ChippClient::new(config);
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello!".to_string(),
    }];

    match client.chat(&mut session, &messages).await {
        Ok(_) => println!("✅ Unexpected success!"),
        Err(e) => {
            println!("❌ Error: {}", e);
            match &e {
                ChippClientError::ApiError { status, message } if *status == 401 => {
                    println!("   → This is an authentication error (401 Unauthorized)");
                    println!("   → The SDK does NOT retry authentication errors");
                    println!("   → Action: Check your CHIPP_API_KEY environment variable");
                    println!("   → Message from API: {}", message);
                }
                _ => println!("   → Unexpected error type"),
            }
        }
    }
}

/// Example 2: Handle network timeout
///
/// This demonstrates:
/// - How timeout errors ARE retried automatically
/// - How to configure timeout duration
/// - How to detect when max retries are exceeded
async fn handle_timeout() {
    let config = ChippConfig {
        api_key: "test-key".to_string(),
        model: "test-app".to_string(),
        timeout: Duration::from_millis(1), // Extremely short timeout to force failure
        max_retries: 2,
        initial_retry_delay: Duration::from_millis(10),
        ..Default::default()
    };

    let client = ChippClient::new(config);
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello!".to_string(),
    }];

    println!("⏱️  Attempting request with 1ms timeout (will fail)...");
    match client.chat(&mut session, &messages).await {
        Ok(_) => println!("✅ Unexpected success!"),
        Err(e) => {
            println!("❌ Error: {}", e);
            match &e {
                ChippClientError::MaxRetriesExceeded(retries) => {
                    println!("   → Max retries ({}) exceeded", retries);
                    println!("   → The SDK automatically retried timeout errors");
                    println!("   → Action: Increase timeout or check network connectivity");
                }
                ChippClientError::HttpError(http_err) if http_err.is_timeout() => {
                    println!("   → This is a timeout error");
                    println!("   → The SDK will retry timeout errors automatically");
                }
                _ => println!("   → Error type: {:?}", classify_error(&e)),
            }
        }
    }
}

/// Example 3: Successful request (demonstrates that retry logic is transparent)
///
/// This demonstrates:
/// - Normal successful requests work seamlessly
/// - Retry logic is transparent when not needed
/// - How to use the ? operator for error propagation
async fn handle_successful_request() {
    // Get real API credentials from environment (if available)
    let api_key = std::env::var("CHIPP_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  CHIPP_API_KEY not set - skipping successful request example");
        String::new()
    });

    if api_key.is_empty() {
        return;
    }

    let app_name_id = std::env::var("CHIPP_APP_NAME_ID").unwrap_or_else(|_| {
        println!("⚠️  CHIPP_APP_NAME_ID not set - skipping successful request example");
        String::new()
    });

    if app_name_id.is_empty() {
        return;
    }

    let config = ChippConfig {
        api_key,
        model: app_name_id,
        max_retries: 3,
        ..Default::default()
    };

    let client = ChippClient::new(config);
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Say 'Hello!' in one word.".to_string(),
    }];

    println!("📤 Sending request...");
    match client.chat(&mut session, &messages).await {
        Ok(response) => {
            println!("✅ Success!");
            println!("   Response: {}", response);
            println!("   Session ID: {:?}", session.chat_session_id);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("   Error type: {:?}", classify_error(&e));
        }
    }
}

/// Example 4: Error recovery with fallback
///
/// This demonstrates:
/// - How to implement fallback strategies
/// - How to provide default responses when API fails
/// - Graceful degradation patterns
async fn handle_with_fallback() {
    let config = ChippConfig {
        api_key: "invalid-key".to_string(),
        model: "test-app".to_string(),
        max_retries: 1,
        ..Default::default()
    };

    let client = ChippClient::new(config);
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "What's the weather?".to_string(),
    }];

    println!("📤 Attempting API request...");
    let response = client
        .chat(&mut session, &messages)
        .await
        .unwrap_or_else(|e| {
            println!("⚠️  API request failed: {}", e);
            println!("   → Using fallback response");
            "I'm sorry, I'm currently unable to process your request. Please try again later."
                .to_string()
        });

    println!("💬 Final response: {}", response);
}

/// Example 5: Proper error propagation with ? operator
///
/// This demonstrates:
/// - How to use the ? operator for clean error handling
/// - How errors propagate up the call stack
/// - How to handle errors at the appropriate level
async fn propagate_errors() -> Result<String, ChippClientError> {
    let config = ChippConfig {
        api_key: "invalid-key".to_string(),
        model: "test-app".to_string(),
        ..Default::default()
    };

    let client = ChippClient::new(config);
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello!".to_string(),
    }];

    println!("📤 Sending request (will propagate error with ?)...");

    // The ? operator will return early if there's an error
    let response = client.chat(&mut session, &messages).await?;

    // This line won't be reached if there's an error
    println!("✅ Success: {}", response);

    Ok(response)
}

/// Helper function to classify errors for debugging
fn classify_error(error: &ChippClientError) -> &'static str {
    match error {
        ChippClientError::HttpError(e) if e.is_timeout() => "Network Timeout (retryable)",
        ChippClientError::HttpError(e) if e.is_connect() => "Connection Error (retryable)",
        ChippClientError::HttpError(_) => "HTTP Error (retryable)",
        ChippClientError::ApiError { status, .. } if *status >= 500 => {
            "Server Error 5xx (retryable)"
        }
        ChippClientError::ApiError { status, .. } if *status == 429 => "Rate Limit 429 (retryable)",
        ChippClientError::ApiError { status, .. } if *status >= 400 => {
            "Client Error 4xx (NOT retryable)"
        }
        ChippClientError::ApiError { .. } => "API Error",
        ChippClientError::InvalidResponse(_) => "Invalid Response (NOT retryable)",
        ChippClientError::StreamError(_) => "Stream Error (NOT retryable)",
        ChippClientError::MaxRetriesExceeded(_) => "Max Retries Exceeded",
    }
}
