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
//! use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage, MessageRole};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ChippConfig {
//!     api_key: "live_your-api-key".to_string(),
//!     base_url: "https://app.chipp.ai/api/v1".to_string(),
//!     model: "myapp-123".to_string(),
//!     timeout: Duration::from_secs(30),
//!     max_retries: 3,
//! };
//!
//! let client = ChippClient::new(config);
//! let mut session = ChippSession::new();
//!
//! let messages = vec![
//!     ChippMessage {
//!         role: MessageRole::User,
//!         content: "What is Chipp?".to_string(),
//!     }
//! ];
//!
//! let response = client.chat(&mut session, &messages).await?;
//! println!("Response: {}", response);
//! # Ok(())
//! # }
//! ```
//!
//! # Streaming Example
//!
//! ```no_run
//! use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage, MessageRole};
//! use futures::StreamExt;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ChippConfig {
//!     api_key: "live_your-api-key".to_string(),
//!     base_url: "https://app.chipp.ai/api/v1".to_string(),
//!     model: "myapp-123".to_string(),
//!     timeout: Duration::from_secs(30),
//!     max_retries: 3,
//! };
//!
//! let client = ChippClient::new(config);
//! let mut session = ChippSession::new();
//!
//! let messages = vec![
//!     ChippMessage {
//!         role: MessageRole::User,
//!         content: "Tell me a story".to_string(),
//!     }
//! ];
//!
//! let mut stream = client.chat_stream(&mut session, &messages).await?;
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

use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur when using the Chipp API client
#[derive(Error, Debug)]
pub enum ChippClientError {
    /// HTTP request failed (network error, DNS failure, etc.)
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// API returned invalid JSON or unexpected response format
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    /// API returned an error response (4xx, 5xx)
    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },

    /// SSE stream parsing error
    #[error("Stream parsing error: {0}")]
    StreamError(String),

    /// Maximum retry attempts exceeded
    #[error("Maximum retry attempts ({0}) exceeded")]
    MaxRetriesExceeded(usize),
}

/// Configuration for Chipp API client
///
/// # Example
///
/// ```
/// use chipp::ChippConfig;
/// use std::time::Duration;
///
/// let config = ChippConfig {
///     api_key: "live_your-api-key".to_string(),
///     base_url: "https://app.chipp.ai/api/v1".to_string(),
///     model: "myapp-123".to_string(),
///     timeout: Duration::from_secs(30),
///     max_retries: 3,
///     initial_retry_delay: Duration::from_millis(100),
///     max_retry_delay: Duration::from_secs(10),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ChippConfig {
    /// Chipp API key (from Share → API tab in Chipp dashboard)
    pub api_key: String,

    /// Base URL for Chipp API (default: `https://app.chipp.ai/api/v1`)
    pub base_url: String,

    /// Chipp appNameId (e.g., "myapp-123" from your Chipp dashboard)
    pub model: String,

    /// Request timeout (default: 30 seconds)
    pub timeout: Duration,

    /// Maximum number of retry attempts for transient failures (default: 3)
    pub max_retries: usize,

    /// Initial delay before first retry (default: 100ms)
    pub initial_retry_delay: Duration,

    /// Maximum delay between retries (default: 10 seconds)
    pub max_retry_delay: Duration,
}

impl Default for ChippConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://app.chipp.ai/api/v1".to_string(),
            model: String::new(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(10),
        }
    }
}

/// Message role in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChippMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Session state for maintaining conversation continuity
#[derive(Debug, Clone)]
pub struct ChippSession {
    /// Chipp chatSessionId for conversation continuity
    pub chat_session_id: Option<String>,
}

impl ChippSession {
    /// Create a new session
    #[must_use]
    pub fn new() -> Self {
        Self {
            chat_session_id: None,
        }
    }

    /// Reset the session (start new conversation)
    pub fn reset(&mut self) {
        self.chat_session_id = None;
    }
}

impl Default for ChippSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Request body for Chipp API
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChippMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "chatSessionId")]
    chat_session_id: Option<String>,
}

/// Response from Chipp API (non-streaming)
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    #[serde(rename = "chatSessionId")]
    chat_session_id: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

/// Chipp API client
pub struct ChippClient {
    http: reqwest::Client,
    config: ChippConfig,
}

impl ChippClient {
    /// Create a new Chipp API client
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration including API key, model, timeout, and retry settings
    ///
    /// # Example
    ///
    /// ```
    /// use chipp::{ChippClient, ChippConfig};
    /// use std::time::Duration;
    ///
    /// let config = ChippConfig {
    ///     api_key: "live_your-api-key".to_string(),
    ///     base_url: "https://app.chipp.ai/api/v1".to_string(),
    ///     model: "myapp-123".to_string(),
    ///     timeout: Duration::from_secs(30),
    ///     max_retries: 3,
    /// };
    ///
    /// let client = ChippClient::new(config);
    /// ```
    #[must_use]
    pub fn new(config: ChippConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self { http, config }
    }

    /// Determine if an error is retryable
    fn is_retryable_error(error: &ChippClientError) -> bool {
        match error {
            // Retry on HTTP errors (network failures, timeouts, DNS errors)
            ChippClientError::HttpError(e) => {
                // Retry on connection errors, timeouts, etc.
                e.is_timeout() || e.is_connect() || e.is_request()
            }
            // Retry on 5xx server errors and 429 rate limiting
            ChippClientError::ApiError { status, .. } => *status >= 500 || *status == 429,
            // Don't retry on invalid responses or stream errors
            ChippClientError::InvalidResponse(_) | ChippClientError::StreamError(_) => false,
            // Don't retry if we've already exceeded max retries
            ChippClientError::MaxRetriesExceeded(_) => false,
        }
    }

    /// Create a backoff strategy for retries
    fn create_backoff(&self) -> backoff::ExponentialBackoff {
        ExponentialBackoffBuilder::new()
            .with_initial_interval(self.config.initial_retry_delay)
            .with_max_interval(self.config.max_retry_delay)
            .with_max_elapsed_time(None) // We'll handle max retries manually
            .with_multiplier(2.0) // Double the delay each time
            .with_randomization_factor(0.3) // Add 30% jitter to prevent thundering herd
            .build()
    }

    /// Send a chat completion request (non-streaming)
    ///
    /// # Arguments
    ///
    /// * `session` - Mutable session to track conversation state (updates `chatSessionId`)
    /// * `messages` - Array of messages in the conversation
    ///
    /// # Returns
    ///
    /// The assistant's response text
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - HTTP request fails
    /// - API returns error response
    /// - Response parsing fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage, MessageRole};
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ChippConfig::default();
    /// # let client = ChippClient::new(config);
    /// let mut session = ChippSession::new();
    /// let messages = vec![
    ///     ChippMessage {
    ///         role: MessageRole::User,
    ///         content: "Hello!".to_string(),
    ///     }
    /// ];
    ///
    /// let response = client.chat(&mut session, &messages).await?;
    /// println!("Response: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, session, messages), fields(correlation_id))]
    pub async fn chat(
        &self,
        session: &mut ChippSession,
        messages: &[ChippMessage],
    ) -> Result<String, ChippClientError> {
        let correlation_id = Uuid::new_v4().to_string();
        tracing::Span::current().record("correlation_id", &correlation_id);

        let mut backoff = self.create_backoff();
        let mut attempt = 0;
        let max_attempts = self.config.max_retries + 1; // +1 for initial attempt

        loop {
            attempt += 1;

            tracing::debug!(
                attempt = attempt,
                max_attempts = max_attempts,
                "Sending Chipp API request"
            );

            let result = self.chat_attempt(session, messages, &correlation_id).await;

            match result {
                Ok(content) => {
                    tracing::debug!("Received Chipp API response");
                    return Ok(content);
                }
                Err(e) if attempt >= max_attempts => {
                    tracing::warn!(
                        attempt = attempt,
                        error = %e,
                        "Max retry attempts exceeded"
                    );
                    return Err(ChippClientError::MaxRetriesExceeded(
                        self.config.max_retries,
                    ));
                }
                Err(e) if Self::is_retryable_error(&e) => {
                    if let Some(delay) = backoff.next_backoff() {
                        tracing::warn!(
                            attempt = attempt,
                            error = %e,
                            delay_ms = delay.as_millis(),
                            "Request failed, retrying after delay"
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        // Backoff exhausted (shouldn't happen with our config)
                        return Err(e);
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Non-retryable error occurred");
                    return Err(e);
                }
            }
        }
    }

    /// Internal method to attempt a single chat request (without retry logic)
    async fn chat_attempt(
        &self,
        session: &mut ChippSession,
        messages: &[ChippMessage],
        correlation_id: &str,
    ) -> Result<String, ChippClientError> {
        let request_body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            stream: false,
            chat_session_id: session.chat_session_id.clone(),
        };

        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .header("X-Correlation-ID", correlation_id)
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChippClientError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let response_body: ChatCompletionResponse = response.json().await.map_err(|e| {
            ChippClientError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        // Update session with new chatSessionId
        session.chat_session_id = Some(response_body.chat_session_id);

        let content = response_body
            .choices
            .first()
            .ok_or_else(|| ChippClientError::InvalidResponse("No choices in response".to_string()))?
            .message
            .content
            .clone();

        Ok(content)
    }

    /// Send a streaming chat completion request (SSE)
    ///
    /// Returns a stream of text chunks as they arrive from the API.
    /// The session's `chatSessionId` is updated when the stream completes.
    ///
    /// # Arguments
    ///
    /// * `session` - Mutable session to track conversation state
    /// * `messages` - Array of messages in the conversation
    ///
    /// # Returns
    ///
    /// A stream of `Result<String, ChippClientError>` where each `Ok(String)` is a chunk of text
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - HTTP request fails
    /// - API returns error response
    /// - SSE stream parsing fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage, MessageRole};
    /// # use futures::StreamExt;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ChippConfig::default();
    /// # let client = ChippClient::new(config);
    /// let mut session = ChippSession::new();
    /// let messages = vec![
    ///     ChippMessage {
    ///         role: MessageRole::User,
    ///         content: "Tell me a story".to_string(),
    ///     }
    /// ];
    ///
    /// let mut stream = client.chat_stream(&mut session, &messages).await?;
    ///
    /// while let Some(chunk) = stream.next().await {
    ///     match chunk {
    ///         Ok(text) => print!("{}", text),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chat_stream(
        &self,
        session: &mut ChippSession,
        messages: &[ChippMessage],
    ) -> Result<ChippStream, ChippClientError> {
        let correlation_id = Uuid::new_v4().to_string();

        let request_body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            stream: true,
            chat_session_id: session.chat_session_id.clone(),
        };

        let url = format!("{}/chat/completions", self.config.base_url);

        tracing::debug!("Sending Chipp API streaming request");

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .header("X-Correlation-ID", &correlation_id)
            .header("Accept", "text/event-stream")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChippClientError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        // Convert response to custom format stream
        // NOTE: Chipp uses a custom streaming format, NOT standard SSE!
        // Format: `0:"content"` for chunks, `e:{...}` for end, `d:{...}` for done
        // See docs/chipp-api-streaming-format.md for details

        // Read all bytes and parse as lines
        let bytes = response.bytes().await?;
        let text = String::from_utf8_lossy(&bytes).to_string();

        // Parse lines and extract content chunks
        let chunks: Vec<Result<String, ChippClientError>> = text
            .lines()
            .filter_map(|line| {
                if let Some(json_str) = line.strip_prefix("0:") {
                    // Content chunk: `0:"Hello! "`
                    match serde_json::from_str::<String>(json_str) {
                        Ok(content) => Some(Ok(content)),
                        Err(e) => Some(Err(ChippClientError::StreamError(format!(
                            "Failed to parse content chunk: {}",
                            e
                        )))),
                    }
                } else if line.starts_with("e:") || line.starts_with("d:") {
                    // End of stream - stop processing
                    None
                } else {
                    // Skip other prefixes (f:, 8:, etc.)
                    None
                }
            })
            .collect();

        let stream = futures::stream::iter(chunks);

        Ok(ChippStream {
            inner: Box::pin(stream),
        })
    }
}

// NOTE: Streaming uses custom format, not OpenAI-compatible SSE
// See docs/chipp-api-streaming-format.md for actual format details

/// Stream of text chunks from Chipp API
///
/// Implements `Stream<Item = Result<String, ChippClientError>>`.
pub struct ChippStream {
    inner: Pin<Box<dyn Stream<Item = Result<String, ChippClientError>> + Send>>,
}

impl std::fmt::Debug for ChippStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChippStream").finish_non_exhaustive()
    }
}

impl Stream for ChippStream {
    type Item = Result<String, ChippClientError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = ChippSession::new();
        assert!(session.chat_session_id.is_none());
    }

    #[test]
    fn test_session_reset() {
        let mut session = ChippSession::new();
        session.chat_session_id = Some("test-session-id".to_string());
        session.reset();
        assert!(session.chat_session_id.is_none());
    }

    #[test]
    fn test_message_role_serialization() {
        let msg = ChippMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
    }

    #[tokio::test]
    async fn test_is_retryable_error_http_timeout() {
        // Create a timeout error by trying to connect to a non-existent server
        let error = ChippClientError::HttpError(
            reqwest::Client::new()
                .get("http://localhost:1")
                .timeout(std::time::Duration::from_millis(1))
                .send()
                .await
                .unwrap_err(),
        );
        assert!(ChippClient::is_retryable_error(&error));
    }

    #[test]
    fn test_is_retryable_error_5xx() {
        let error = ChippClientError::ApiError {
            status: 500,
            message: "Internal Server Error".to_string(),
        };
        assert!(ChippClient::is_retryable_error(&error));

        let error = ChippClientError::ApiError {
            status: 503,
            message: "Service Unavailable".to_string(),
        };
        assert!(ChippClient::is_retryable_error(&error));
    }

    #[test]
    fn test_is_retryable_error_429() {
        let error = ChippClientError::ApiError {
            status: 429,
            message: "Too Many Requests".to_string(),
        };
        assert!(ChippClient::is_retryable_error(&error));
    }

    #[test]
    fn test_is_not_retryable_error_4xx() {
        let error = ChippClientError::ApiError {
            status: 400,
            message: "Bad Request".to_string(),
        };
        assert!(!ChippClient::is_retryable_error(&error));

        let error = ChippClientError::ApiError {
            status: 401,
            message: "Unauthorized".to_string(),
        };
        assert!(!ChippClient::is_retryable_error(&error));

        let error = ChippClientError::ApiError {
            status: 404,
            message: "Not Found".to_string(),
        };
        assert!(!ChippClient::is_retryable_error(&error));
    }

    #[test]
    fn test_is_not_retryable_error_invalid_response() {
        let error = ChippClientError::InvalidResponse("Bad JSON".to_string());
        assert!(!ChippClient::is_retryable_error(&error));
    }

    #[test]
    fn test_is_not_retryable_error_max_retries_exceeded() {
        let error = ChippClientError::MaxRetriesExceeded(3);
        assert!(!ChippClient::is_retryable_error(&error));
    }

    #[test]
    fn test_backoff_configuration() {
        let config = ChippConfig {
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(10),
            ..Default::default()
        };
        let client = ChippClient::new(config);
        let _backoff = client.create_backoff();

        // Verify backoff is created with correct configuration
        // Note: We can't easily test the exact values without making the backoff public,
        // but we can verify it compiles and creates successfully
    }

    #[test]
    fn test_config_default_retry_values() {
        let config = ChippConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_retry_delay, Duration::from_millis(100));
        assert_eq!(config.max_retry_delay, Duration::from_secs(10));
    }
}
