//! Streaming chat example
//!
//! Run with:
//! ```bash
//! export CHIPP_API_KEY="your-api-key"
//! export CHIPP_APP_NAME_ID="your-app-name-id"
//! cargo run --example streaming
//! ```

use chipp::{ChippClient, ChippConfig, ChippMessage, ChippSession, MessageRole};
use futures::StreamExt;

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
        model: app_name_id,
        ..Default::default()
    };

    // Create client and session
    let client = ChippClient::new(config)?;
    let mut session = ChippSession::new();

    // Send a message with streaming
    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Tell me a short story about a robot learning to code.".to_string(),
    }];

    println!("Sending message to Chipp API (streaming)...\n");
    println!("✅ Response:");

    let mut stream = client.chat_stream(&mut session, &messages).await?;
    let mut full_response = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => {
                print!("{}", text);
                full_response.push_str(&text);
            }
            Err(e) => {
                eprintln!("\n❌ Stream error: {}", e);
                return Err(e.into());
            }
        }
    }

    println!("\n\n📝 Session ID: {:?}", session.chat_session_id);
    println!("📊 Total characters: {}", full_response.len());

    Ok(())
}
