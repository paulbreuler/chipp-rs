//! Streaming response types and parsing for the Chipp API.
//!
//! The Chipp API uses Server-Sent Events (SSE) with custom JSON event types:
//!
//! - `text-delta`: Content chunks with `delta` field
//! - `message-metadata`: Contains `persistedMessageId` for session tracking
//! - `finish`: Stream completion signal
//!
//! # Example Format
//!
//! ```text
//! data: {"type":"text-delta","id":"...","delta":"Hello "}
//! data: {"type":"text-delta","id":"...","delta":"world!"}
//! data: {"type":"message-metadata","messageMetadata":{"annotations":[{"persistedMessageId":"uuid"}]}}
//! data: [DONE]
//! ```

use crate::error::ChippClientError;
use bytes::Bytes;
use futures::Stream;
use serde::Deserialize;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;

/// A stream event from the Chipp API.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Text content chunk
    TextDelta(String),
    /// Session ID from message metadata
    SessionId(String),
    /// Stream finished
    Done,
}

/// Internal JSON structure for SSE events.
#[derive(Debug, Deserialize)]
struct SseEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<String>,
    #[serde(rename = "messageMetadata")]
    message_metadata: Option<MessageMetadata>,
}

#[derive(Debug, Deserialize)]
struct MessageMetadata {
    annotations: Vec<Annotation>,
}

#[derive(Debug, Deserialize)]
struct Annotation {
    #[serde(rename = "persistedMessageId")]
    persisted_message_id: Option<String>,
}

/// Parse a single SSE line into an event.
pub fn parse_sse_line(line: &str) -> Option<StreamEvent> {
    let data = line.strip_prefix("data: ")?;

    // Handle [DONE] signal
    if data == "[DONE]" {
        return Some(StreamEvent::Done);
    }

    // Parse JSON event
    let event: SseEvent = serde_json::from_str(data).ok()?;

    match event.event_type.as_str() {
        "text-delta" => event.delta.map(StreamEvent::TextDelta),
        "message-metadata" => {
            // Extract persistedMessageId from annotations
            event.message_metadata.and_then(|meta| {
                meta.annotations
                    .into_iter()
                    .find_map(|ann| ann.persisted_message_id.map(StreamEvent::SessionId))
            })
        }
        _ => None,
    }
}

/// Stream of text chunks from Chipp API.
///
/// Implements `Stream<Item = Result<String, ChippClientError>>`.
///
/// Use with `futures::StreamExt` to iterate over chunks:
///
/// ```no_run
/// use futures::StreamExt;
/// # use chipp::{ChippClient, ChippConfig, ChippSession, ChippMessage};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let config = ChippConfig::default();
/// # let client = ChippClient::new(config)?;
/// # let mut session = ChippSession::new();
/// let mut stream = client.chat_stream(&mut session, &[ChippMessage::user("Hi")]).await?;
///
/// while let Some(chunk) = stream.next().await {
///     print!("{}", chunk?);
/// }
/// # Ok(())
/// # }
/// ```
pub struct ChippStream {
    /// Inner byte stream from reqwest
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    /// Buffer for incomplete SSE lines
    buffer: String,
    /// Shared reference to session for updating chatSessionId
    session_id: Arc<Mutex<Option<String>>>,
    /// Whether stream has finished
    finished: bool,
}

impl std::fmt::Debug for ChippStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChippStream")
            .field("finished", &self.finished)
            .finish_non_exhaustive()
    }
}

impl ChippStream {
    /// Create a new stream from a reqwest byte stream.
    pub(crate) fn new(
        inner: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
        session_id: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self {
            inner,
            buffer: String::new(),
            session_id,
            finished: false,
        }
    }

    /// Get the session ID captured during streaming (if available).
    ///
    /// This is set when the API sends `message-metadata` with `persistedMessageId`.
    pub async fn session_id(&self) -> Option<String> {
        self.session_id.lock().await.clone()
    }

    /// Process buffered data and extract next text chunk.
    fn process_buffer(&mut self) -> Option<Result<String, ChippClientError>> {
        // Process complete lines from buffer
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].trim().to_string();
            self.buffer = self.buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Some(event) = parse_sse_line(&line) {
                match event {
                    StreamEvent::TextDelta(text) => {
                        return Some(Ok(text));
                    }
                    StreamEvent::SessionId(id) => {
                        // Update session ID asynchronously
                        // We can't await here, so we use try_lock
                        if let Ok(mut guard) = self.session_id.try_lock() {
                            *guard = Some(id);
                        }
                    }
                    StreamEvent::Done => {
                        self.finished = true;
                        return None;
                    }
                }
            }
        }
        None
    }
}

impl Stream for ChippStream {
    type Item = Result<String, ChippClientError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }

        // First, try to get content from existing buffer
        if let Some(result) = self.process_buffer() {
            return Poll::Ready(Some(result));
        }

        // Poll for more data from the inner stream
        loop {
            match self.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    // Append new data to buffer
                    match String::from_utf8(bytes.to_vec()) {
                        Ok(text) => {
                            self.buffer.push_str(&text);
                            // Try to extract content from buffer
                            if let Some(result) = self.process_buffer() {
                                return Poll::Ready(Some(result));
                            }
                            // No complete line yet, continue polling
                        }
                        Err(e) => {
                            return Poll::Ready(Some(Err(ChippClientError::StreamError(format!(
                                "Invalid UTF-8 in stream: {}",
                                e
                            )))));
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(ChippClientError::HttpError(e))));
                }
                Poll::Ready(None) => {
                    // Stream ended, process any remaining buffer
                    if !self.buffer.is_empty() {
                        if let Some(result) = self.process_buffer() {
                            return Poll::Ready(Some(result));
                        }
                    }
                    self.finished = true;
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}
