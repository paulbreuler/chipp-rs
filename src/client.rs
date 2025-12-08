//! Chipp API client implementation.

use crate::config::ChippConfig;
use crate::error::ChippClientError;
use crate::stream::ChippStream;
use crate::types::{
    ChatCompletionRequest, ChatCompletionResponse, ChatResponse, ChippMessage, ChippSession,
};

use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Chipp API client.
///
/// # Example
///
/// ```no_run
/// use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ChippConfig::builder()
///     .api_key("YOUR_API_KEY_HERE")
///     .model("myapp-123")
///     .build()?;
///
/// let client = ChippClient::new(config)?;
/// let mut session = ChippSession::new();
///
/// let response = client.chat(&mut session, &[ChippMessage::user("Hello!")]).await?;
/// println!("Response: {}", response);
/// # Ok(())
/// # }
/// ```
pub struct ChippClient {
    http: reqwest::Client,
    config: ChippConfig,
}

impl ChippClient {
    /// Create a new Chipp API client.
    ///
    /// # Errors
    ///
    /// Returns `ChippClientError::HttpError` if the underlying HTTP client fails to build.
    pub fn new(config: ChippConfig) -> Result<Self, ChippClientError> {
        let http = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self { http, config })
    }

    /// Determine if an error is retryable.
    fn is_retryable_error(error: &ChippClientError) -> bool {
        match error {
            ChippClientError::HttpError(e) => e.is_timeout() || e.is_connect() || e.is_request(),
            ChippClientError::ApiError { status, .. } => *status >= 500 || *status == 429,
            _ => false,
        }
    }

    /// Create a backoff strategy for retries.
    fn create_backoff(&self) -> backoff::ExponentialBackoff {
        ExponentialBackoffBuilder::new()
            .with_initial_interval(self.config.initial_retry_delay)
            .with_max_interval(self.config.max_retry_delay)
            .with_max_elapsed_time(None)
            .with_multiplier(2.0)
            .with_randomization_factor(0.3)
            .build()
    }

    /// Send a chat completion request (non-streaming).
    ///
    /// This is a convenience method that returns just the response content as a string.
    /// For access to token usage and other metadata, use [`chat_detailed()`](Self::chat_detailed).
    ///
    /// # Arguments
    ///
    /// * `session` - Session to track conversation state (updates `chatSessionId`)
    /// * `messages` - Messages in the conversation
    ///
    /// # Returns
    ///
    /// The assistant's response text.
    ///
    /// # Errors
    ///
    /// Returns error if HTTP request fails, API returns error, or response parsing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ChippConfig::default();
    /// # let client = ChippClient::new(config)?;
    /// let mut session = ChippSession::new();
    /// let response = client.chat(&mut session, &[ChippMessage::user("Hello!")]).await?;
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
        let response = self.chat_detailed(session, messages).await?;
        Ok(response.content().to_string())
    }

    /// Send a chat completion request and return the full response with metadata.
    ///
    /// This method returns a [`ChatResponse`] containing:
    /// - The AI's response content
    /// - Token usage information for rate limiting and monitoring
    /// - Completion ID for debugging
    /// - Timestamps and finish reason
    ///
    /// For simple use cases where you only need the response text,
    /// use [`chat()`](Self::chat) instead.
    ///
    /// # Arguments
    ///
    /// * `session` - Session to track conversation state (updates `chatSessionId`)
    /// * `messages` - Messages in the conversation
    ///
    /// # Returns
    ///
    /// A [`ChatResponse`] containing the response and metadata.
    ///
    /// # Errors
    ///
    /// Returns error if HTTP request fails, API returns error, or response parsing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ChippConfig::default();
    /// # let client = ChippClient::new(config)?;
    /// let mut session = ChippSession::new();
    /// let response = client.chat_detailed(&mut session, &[ChippMessage::user("Hello!")]).await?;
    ///
    /// println!("Response: {}", response.content());
    /// println!("Tokens used: {}", response.usage().total_tokens);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, session, messages), fields(correlation_id))]
    pub async fn chat_detailed(
        &self,
        session: &mut ChippSession,
        messages: &[ChippMessage],
    ) -> Result<ChatResponse, ChippClientError> {
        let correlation_id = Uuid::new_v4().to_string();
        tracing::Span::current().record("correlation_id", &correlation_id);

        let mut backoff = self.create_backoff();
        let mut attempt = 0;
        let max_attempts = self.config.max_retries + 1;

        loop {
            attempt += 1;
            let result = self.chat_attempt(session, messages, &correlation_id).await;

            match result {
                Ok(response) => return Ok(response),
                Err(e) if attempt >= max_attempts => {
                    tracing::warn!(attempt, error = %e, "Max retry attempts exceeded");
                    return Err(ChippClientError::MaxRetriesExceeded(
                        self.config.max_retries,
                    ));
                }
                Err(e) if Self::is_retryable_error(&e) => {
                    if let Some(delay) = backoff.next_backoff() {
                        tracing::warn!(attempt, error = %e, delay_ms = delay.as_millis(), "Retrying");
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(e);
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Non-retryable error");
                    return Err(e);
                }
            }
        }
    }

    /// Internal method for a single chat attempt.
    ///
    /// Returns a `ChatResponse` with all metadata from the API.
    async fn chat_attempt(
        &self,
        session: &mut ChippSession,
        messages: &[ChippMessage],
        correlation_id: &str,
    ) -> Result<ChatResponse, ChippClientError> {
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

        // Validate we have at least one choice before converting
        if response_body.choices.is_empty() {
            return Err(ChippClientError::InvalidResponse(
                "No choices in response".to_string(),
            ));
        }

        // Update session with the new session ID
        session.chat_session_id = Some(response_body.chat_session_id.clone());

        // Convert internal response to public type
        Ok(response_body.into())
    }

    /// Send a streaming chat completion request (SSE).
    ///
    /// Returns a stream of text chunks as they arrive from the API.
    /// The session's `chatSessionId` is updated when the stream receives metadata.
    ///
    /// # Arguments
    ///
    /// * `session` - Session to track conversation state
    /// * `messages` - Messages in the conversation
    ///
    /// # Returns
    ///
    /// A stream of `Result<String, ChippClientError>` where each `Ok(String)` is a text chunk.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage};
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ChippConfig::default();
    /// # let client = ChippClient::new(config)?;
    /// let mut session = ChippSession::new();
    /// let mut stream = client.chat_stream(&mut session, &[ChippMessage::user("Hello")]).await?;
    ///
    /// while let Some(chunk) = stream.next().await {
    ///     print!("{}", chunk?);
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

        // Create shared session ID that stream will update
        let session_id = Arc::new(Mutex::new(None::<String>));

        // Get the byte stream for true streaming (not buffered!)
        let byte_stream = response.bytes_stream();

        // Create the stream
        let stream = ChippStream::new(Box::pin(byte_stream), session_id);

        Ok(stream)
    }

    /// Send a streaming chat completion and collect the full response.
    ///
    /// This is a convenience method that consumes the entire stream and
    /// updates the session with the captured session ID.
    ///
    /// For true streaming where you process chunks as they arrive,
    /// use [`chat_stream`](Self::chat_stream) instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ChippConfig::default();
    /// # let client = ChippClient::new(config)?;
    /// let mut session = ChippSession::new();
    /// let response = client.chat_stream_collect(&mut session, &[ChippMessage::user("Hello")]).await?;
    /// println!("Response: {}", response);
    /// println!("Session ID: {:?}", session.chat_session_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chat_stream_collect(
        &self,
        session: &mut ChippSession,
        messages: &[ChippMessage],
    ) -> Result<String, ChippClientError> {
        let mut stream = self.chat_stream(session, messages).await?;
        let mut full_response = String::new();

        while let Some(chunk) = stream.next().await {
            full_response.push_str(&chunk?);
        }

        // Update session with captured ID after stream completes
        if let Some(id) = stream.session_id().await {
            session.chat_session_id = Some(id);
        }

        Ok(full_response)
    }

    /// Measure the round-trip latency to the Chipp API.
    ///
    /// This method performs a lightweight HEAD request to the chat completions endpoint
    /// and measures the time taken for the request to complete.
    ///
    /// # Returns
    ///
    /// - `Ok(Duration)` containing the round-trip latency if successful
    /// - `Err(ChippClientError::HttpError)` if a network error occurs
    ///
    /// # Use Case
    ///
    /// This is useful for monitoring API performance and deciding whether to route
    /// requests to the Chipp API or fall back to a local LLM based on latency.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use chipp::{ChippClient, ChippConfig};
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ChippConfig::builder()
    ///     .api_key("YOUR_API_KEY_HERE")
    ///     .model("myapp-123")
    ///     .build()?;
    ///
    /// let client = ChippClient::new(config)?;
    ///
    /// // Measure API latency
    /// let latency = client.ping().await?;
    /// println!("API latency: {:?}", latency);
    ///
    /// if latency < Duration::from_secs(2) {
    ///     println!("Low latency, using Chipp API");
    /// } else {
    ///     println!("High latency, falling back to local LLM");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ChippClientError::HttpError` if the network request fails due to
    /// timeout, DNS resolution failure, or other connectivity issues.
    pub async fn ping(&self) -> Result<std::time::Duration, ChippClientError> {
        let url = format!("{}/chat/completions", self.config.base_url);

        // Start timer
        let start = std::time::Instant::now();

        // Use HEAD request for minimal overhead
        let _response = self.http.head(&url).send().await?;

        // Calculate elapsed time
        let latency = start.elapsed();

        Ok(latency)
    }
}
