//! Production-ready Chipp API client
//!
//! Provides async HTTP client for interacting with the Chipp API (<https://chipp.ai>).
//! Supports both non-streaming and streaming (SSE) responses with automatic retry logic.
//!
//! # Features
//!
//! - **Non-streaming chat**: Simple request/response with `chat()`
//! - **Streaming chat**: Server-Sent Events (SSE) with `chat_stream()`
//! - **Session management**: Automatic `chatSessionId` tracking for conversation continuity
//! - **Retry logic**: Exponential backoff for transient failures (5xx, network errors)
//! - **Configurable timeouts**: Per-request timeout configuration
//! - **Correlation IDs**: Automatic UUID generation for request tracing
//!
//! # API Reference
//!
//! See: <https://chipp.ai/docs/api/reference>
//!
//! # Non-Streaming Example
//!
//! ```no_run
//! use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ChippConfig::builder()
//!     .api_key("YOUR_API_KEY_HERE")
//!     .model("myapp-123")
//!     .build()?;
//!
//! let client = ChippClient::new(config)?;
//! let mut session = ChippSession::new();
//!
//! let response = client.chat(&mut session, &[ChippMessage::user("What is Chipp?")]).await?;
//! println!("Response: {}", response);
//! # Ok(())
//! # }
//! ```
//!
//! # Streaming Example
//!
//! ```no_run
//! use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage};
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ChippConfig::builder()
//!     .api_key("YOUR_API_KEY_HERE")
//!     .model("myapp-123")
//!     .build()?;
//!
//! let client = ChippClient::new(config)?;
//! let mut session = ChippSession::new();
//!
//! let mut stream = client.chat_stream(&mut session, &[ChippMessage::user("Tell me a story")]).await?;
//!
//! while let Some(chunk) = stream.next().await {
//!     match chunk {
//!         Ok(text) => print!("{}", text),
//!         Err(e) => eprintln!("Stream error: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod config;
mod error;
mod stream;
mod types;

// Re-export public API
pub use client::ChippClient;
pub use config::{ChippConfig, ChippConfigBuilder};
pub use error::{ChippClientError, Result};
pub use stream::ChippStream;
pub use types::{ChatResponse, ChippMessage, ChippSession, MessageRole, Usage};
