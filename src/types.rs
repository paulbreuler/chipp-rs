//! Core types for the Chipp API client.

use serde::{Deserialize, Serialize};

/// Message role in conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User message
    User,
    /// Assistant (AI) response
    Assistant,
    /// System prompt/instructions
    System,
}

/// A message in the conversation.
///
/// # Example
///
/// ```
/// use chipp::{ChippMessage, MessageRole};
///
/// let msg = ChippMessage {
///     role: MessageRole::User,
///     content: "Hello!".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChippMessage {
    /// The role of the message sender
    pub role: MessageRole,
    /// The message content
    pub content: String,
}

impl ChippMessage {
    /// Create a user message.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// Create an assistant message.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }

    /// Create a system message.
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }
}

/// Session state for maintaining conversation continuity.
///
/// The Chipp API uses `chatSessionId` to maintain conversation context.
/// Pass a session to multiple requests to continue a conversation.
///
/// # Example
///
/// ```
/// use chipp::ChippSession;
///
/// let mut session = ChippSession::new();
/// assert!(session.chat_session_id.is_none());
///
/// // After first API call, session.chat_session_id will be populated
/// ```
#[derive(Debug, Clone, Default)]
pub struct ChippSession {
    /// Chipp chatSessionId for conversation continuity
    pub chat_session_id: Option<String>,
}

impl ChippSession {
    /// Create a new session (no existing conversation).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a session with an existing session ID.
    #[must_use]
    pub fn with_id(chat_session_id: impl Into<String>) -> Self {
        Self {
            chat_session_id: Some(chat_session_id.into()),
        }
    }

    /// Reset the session (start new conversation).
    pub fn reset(&mut self) {
        self.chat_session_id = None;
    }
}

// Internal request/response types

/// Request body for Chipp API.
#[derive(Debug, Serialize)]
pub(crate) struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChippMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "chatSessionId")]
    pub chat_session_id: Option<String>,
}

/// Response from Chipp API (non-streaming).
#[derive(Debug, Deserialize)]
pub(crate) struct ChatCompletionResponse {
    #[serde(rename = "chatSessionId")]
    pub chat_session_id: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Choice {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Message {
    pub content: String,
}
