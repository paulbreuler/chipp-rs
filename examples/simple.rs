//! Simple non-streaming chat example
//!
//! Run with:
//! ```bash
//! export CHIPP_API_KEY="your-api-key"
//! export CHIPP_APP_NAME_ID="your-app-name-id"
//! cargo run --example simple
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

    // Send a message
    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello! Can you tell me a short joke?".to_string(),
    }];

    println!("Sending message to Chipp API...");
    let response = client.chat(&mut session, &messages).await?;

    println!("\n✅ Response:");
    println!("{}", response);
    println!("\n📝 Session ID: {:?}", session.chat_session_id);

    Ok(())
}
