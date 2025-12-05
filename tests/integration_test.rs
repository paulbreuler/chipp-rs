//! Integration tests for Chipp API client
//!
//! These tests call the real Chipp API and require:
//! - CHIPP_API_KEY environment variable
//! - CHIPP_APP_NAME_ID environment variable (your appNameId from Chipp dashboard)
//!
//! Run with: `cargo test --features integration-tests -- --ignored`

use chipp::{ChippClient, ChippConfig, ChippMessage, ChippSession, MessageRole};
use futures::StreamExt;

fn get_test_config() -> Option<ChippConfig> {
    let api_key = std::env::var("CHIPP_API_KEY").ok()?;
    let model = std::env::var("CHIPP_APP_NAME_ID")
        .ok()
        .unwrap_or_else(|| "newapplication-10032142".to_string());

    Some(ChippConfig {
        api_key,
        model,
        ..Default::default()
    })
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_chat_non_streaming() {
    let config = get_test_config().expect("CHIPP_API_KEY and CHIPP_APP_NAME_ID must be set");
    let client = ChippClient::new(config).expect("Failed to create client");
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Say 'Hello from Rust!' and nothing else.".to_string(),
    }];

    let response = client
        .chat(&mut session, &messages)
        .await
        .expect("Chat request failed");

    println!("Response: {}", response);
    assert!(!response.is_empty());
    assert!(session.chat_session_id.is_some());
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_chat_session_continuity() {
    let config = get_test_config().expect("CHIPP_API_KEY and CHIPP_APP_NAME_ID must be set");
    let client = ChippClient::new(config).expect("Failed to create client");
    let mut session = ChippSession::new();

    // First message: Ask to remember something
    let messages1 = vec![ChippMessage {
        role: MessageRole::User,
        content: "Remember this number: 42. Just acknowledge you'll remember it.".to_string(),
    }];

    let response1 = client
        .chat(&mut session, &messages1)
        .await
        .expect("First chat request failed");

    println!("Response 1: {}", response1);
    assert!(!response1.is_empty());
    let session_id = session.chat_session_id.clone();
    assert!(session_id.is_some());

    // Second message: Ask what was remembered (tests session continuity)
    let messages2 = vec![ChippMessage {
        role: MessageRole::User,
        content: "What number did I tell you to remember?".to_string(),
    }];

    let response2 = client
        .chat(&mut session, &messages2)
        .await
        .expect("Second chat request failed");

    println!("Response 2: {}", response2);
    assert!(!response2.is_empty());
    assert_eq!(
        session.chat_session_id, session_id,
        "Session ID should remain the same"
    );

    // Response should mention "42"
    assert!(
        response2.contains("42"),
        "Response should mention the number 42, got: {}",
        response2
    );
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_chat_streaming() {
    let config = get_test_config().expect("CHIPP_API_KEY and CHIPP_APP_NAME_ID must be set");
    let client = ChippClient::new(config).expect("Failed to create client");
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Count from 1 to 5, with each number on a new line.".to_string(),
    }];

    println!("Creating stream...");
    let mut stream = client
        .chat_stream(&mut session, &messages)
        .await
        .expect("Stream request failed");

    println!("Stream created, reading chunks...");
    let mut full_response = String::new();
    let mut chunk_count = 0;

    while let Some(chunk) = stream.next().await {
        println!("Received chunk #{}", chunk_count + 1);
        match chunk {
            Ok(text) => {
                print!("{}", text);
                full_response.push_str(&text);
                chunk_count += 1;
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                panic!("Stream error: {}", e);
            }
        }
    }
    println!("\nStream ended");

    println!("\nReceived {} chunks", chunk_count);
    println!("Full response: {}", full_response);

    assert!(!full_response.is_empty(), "Should receive some response");
    assert!(chunk_count > 0, "Should receive at least one chunk");
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_is_healthy_with_real_api() {
    let config = get_test_config().expect("CHIPP_API_KEY and CHIPP_APP_NAME_ID must be set");
    let client = ChippClient::new(config).expect("Failed to create client");

    let is_healthy = client
        .is_healthy()
        .await
        .expect("Health check request failed");

    println!("API is healthy: {}", is_healthy);
    assert!(is_healthy, "API should be healthy");
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_ping_with_real_api() {
    let config = get_test_config().expect("CHIPP_API_KEY and CHIPP_APP_NAME_ID must be set");
    let client = ChippClient::new(config).expect("Failed to create client");

    let latency = client.ping().await.expect("Ping request failed");

    println!("API latency: {:?}", latency);
    // Latency should be reasonable (< 5 seconds for real API)
    assert!(
        latency < std::time::Duration::from_secs(5),
        "Latency should be less than 5 seconds, got {:?}",
        latency
    );
}
