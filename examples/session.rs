//! Session continuity example
//!
//! Demonstrates how the client maintains conversation context across multiple messages.
//!
//! Run with:
//! ```bash
//! export CHIPP_API_KEY="your-api-key"
//! export CHIPP_APP_NAME_ID="your-app-name-id"
//! cargo run --example session
//! ```

use chipp::{ChippClient, ChippConfig, ChippMessage, ChippSession, MessageRole};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get API credentials from environment
    let api_key =
        std::env::var("CHIPP_API_KEY").expect("CHIPP_API_KEY environment variable not set");
    let app_name_id =
        std::env::var("CHIPP_APP_NAME_ID").expect("CHIPP_APP_NAME_ID environment variable not set");

    // Create client configuration
    let config = ChippConfig {
        api_key,
        base_url: "https://app.chipp.ai/api/v1".to_string(),
        model: app_name_id,
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };

    // Create client and session
    let client = ChippClient::new(config);
    let mut session = ChippSession::new();

    println!("=== Session Continuity Example ===\n");

    // First message: Ask to remember something
    println!("👤 User: Remember this number: 42");
    let messages1 = vec![ChippMessage {
        role: MessageRole::User,
        content: "Remember this number: 42".to_string(),
    }];

    let response1 = client.chat(&mut session, &messages1).await?;
    println!("🤖 Assistant: {}\n", response1);
    println!("📝 Session ID: {:?}\n", session.chat_session_id);

    // Second message: Ask what was remembered (tests session continuity)
    println!("👤 User: What number did I tell you to remember?");
    let messages2 = vec![ChippMessage {
        role: MessageRole::User,
        content: "What number did I tell you to remember?".to_string(),
    }];

    let response2 = client.chat(&mut session, &messages2).await?;
    println!("🤖 Assistant: {}\n", response2);
    println!("📝 Session ID: {:?}\n", session.chat_session_id);

    // Third message: Continue the conversation
    println!("👤 User: What's that number multiplied by 2?");
    let messages3 = vec![ChippMessage {
        role: MessageRole::User,
        content: "What's that number multiplied by 2?".to_string(),
    }];

    let response3 = client.chat(&mut session, &messages3).await?;
    println!("🤖 Assistant: {}\n", response3);

    println!("✅ Session continuity working! The assistant remembered context across 3 messages.");

    Ok(())
}
