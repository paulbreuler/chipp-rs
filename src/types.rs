//! Core types for the Chipp API client.
//!
//! This module provides all the types needed to interact with the Chipp API:
//!
//! - [`ChippMessage`] - Messages in a conversation
//! - [`ChippSession`] - Session state for conversation continuity
//! - [`ChatResponse`] - Full response from chat completion (includes token usage)
//! - [`Usage`] - Token usage information for monitoring

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

// =============================================================================
// Public Response Types
// =============================================================================

/// Token usage information from the API response.
///
/// The Chipp API returns token counts for every chat completion request.
/// Use this for rate limiting and monitoring token consumption.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Usage {
    /// Number of tokens in the prompt (input).
    /// Defaults to 0 if the API returns null or is missing.
    #[serde(default, deserialize_with = "deserialize_null_as_zero")]
    pub prompt_tokens: u32,
    /// Number of tokens in the completion (output).
    /// Defaults to 0 if the API returns null or is missing.
    #[serde(default, deserialize_with = "deserialize_null_as_zero")]
    pub completion_tokens: u32,
    /// Total tokens used (prompt + completion).
    /// Defaults to 0 if the API returns null or is missing.
    #[serde(default, deserialize_with = "deserialize_null_as_zero")]
    pub total_tokens: u32,
}

/// Deserialize a u32 that may be null, defaulting null to 0.
fn deserialize_null_as_zero<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<u32> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or(0))
}

/// Response from a chat completion request.
///
/// Contains the AI's response message plus metadata like token usage,
/// completion ID, and timestamps. Use this when you need more than just
/// the response text.
///
/// # When to Use
///
/// Use [`ChippClient::chat_detailed()`](crate::ChippClient::chat_detailed) to get a `ChatResponse`
/// when you need:
/// - Token usage for rate limiting and monitoring
/// - Completion ID for debugging/logging
/// - Finish reason to understand why completion stopped
///
/// For simple use cases where you only need the response text,
/// use [`ChippClient::chat()`](crate::ChippClient::chat) instead.
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
/// println!("Completion ID: {}", response.completion_id());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// The AI's response content
    content: String,
    /// Chat session ID for conversation continuity
    session_id: String,
    /// Token usage information
    usage: Usage,
    /// Completion ID for debugging (e.g., "chatcmpl-79f98a48-...")
    completion_id: String,
    /// Unix timestamp when the response was created
    created_at: i64,
    /// Reason the completion finished ("stop", "length", etc.)
    finish_reason: String,
    /// The model/app ID used for this completion
    model: String,
}

impl ChatResponse {
    /// Get the AI's response content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the chat session ID.
    ///
    /// Use this to continue conversations across requests.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get token usage information.
    ///
    /// Use this for rate limiting and monitoring.
    #[must_use]
    pub fn usage(&self) -> &Usage {
        &self.usage
    }

    /// Get the completion ID.
    ///
    /// Useful for debugging and request tracing.
    #[must_use]
    pub fn completion_id(&self) -> &str {
        &self.completion_id
    }

    /// Get the creation timestamp (Unix time).
    #[must_use]
    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    /// Get the finish reason.
    ///
    /// Common values:
    /// - `"stop"`: Normal completion
    /// - `"length"`: Max tokens reached
    #[must_use]
    pub fn finish_reason(&self) -> &str {
        &self.finish_reason
    }

    /// Get the model/app ID used for this completion.
    ///
    /// Note: This is the Chipp app ID (e.g., "myapp-123"), not the
    /// underlying LLM model (e.g., "gpt-4o"). The Chipp API does not
    /// expose the underlying model information.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }
}

// =============================================================================
// Internal Request/Response Types
// =============================================================================

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
///
/// This is the internal type that matches the raw API response structure.
/// It gets converted to the public `ChatResponse` type.
#[derive(Debug, Deserialize)]
pub(crate) struct ChatCompletionResponse {
    /// Chipp's session ID for conversation continuity
    #[serde(rename = "chatSessionId")]
    pub chat_session_id: String,

    /// Unique ID for this completion (e.g., "chatcmpl-79f98a48-...")
    pub id: String,

    /// Object type (always "chat.completion").
    #[allow(dead_code)]
    pub object: String,

    /// Unix timestamp when the response was created
    pub created: i64,

    /// The model/app ID used (this is the Chipp app ID, not underlying LLM)
    pub model: String,

    /// Array of completion choices
    pub choices: Vec<Choice>,

    /// Token usage information
    pub usage: Usage,
}

/// A single completion choice from the API.
#[derive(Debug, Deserialize)]
pub(crate) struct Choice {
    /// Index of this choice (usually 0 for single completions).
    #[allow(dead_code)]
    pub index: u32,

    /// The message content
    pub message: ResponseMessage,

    /// Why the completion stopped (e.g., "stop", "length")
    pub finish_reason: String,
}

/// Message in the API response (internal type).
///
/// Note: This is separate from `ChippMessage` to avoid confusion.
/// `ChippMessage` is for requests, `ResponseMessage` is for responses.
#[derive(Debug, Deserialize)]
pub(crate) struct ResponseMessage {
    /// Role of the message sender (always "assistant" in responses).
    #[allow(dead_code)]
    pub role: String,

    /// The message content
    pub content: String,
}

// =============================================================================
// Type Conversions
// =============================================================================

impl From<ChatCompletionResponse> for ChatResponse {
    fn from(response: ChatCompletionResponse) -> Self {
        // Get the first choice - API always returns at least one
        let choice = response
            .choices
            .into_iter()
            .next()
            .expect("API response must have at least one choice");

        Self {
            content: choice.message.content,
            session_id: response.chat_session_id,
            usage: response.usage,
            completion_id: response.id,
            created_at: response.created,
            finish_reason: choice.finish_reason,
            model: response.model,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_deserialization() {
        let json = r#"{
            "prompt_tokens": 100,
            "completion_tokens": 50,
            "total_tokens": 150
        }"#;

        let usage: Usage = serde_json::from_str(json).expect("Usage should deserialize");
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    /// Tests that null values in usage fields default to 0.
    /// The Chipp API can return null for completion_tokens in some cases.
    #[test]
    fn test_usage_deserialization_with_null_values() {
        let json = r#"{
            "prompt_tokens": 9240,
            "completion_tokens": null,
            "total_tokens": 9240
        }"#;

        let usage: Usage = serde_json::from_str(json).expect("Usage should deserialize with null");
        assert_eq!(usage.prompt_tokens, 9240);
        assert_eq!(usage.completion_tokens, 0); // null defaults to 0
        assert_eq!(usage.total_tokens, 9240);
    }

    #[test]
    fn test_usage_equality() {
        let usage1 = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        let usage2 = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        let usage3 = Usage {
            prompt_tokens: 200,
            completion_tokens: 50,
            total_tokens: 250,
        };

        assert_eq!(usage1, usage2);
        assert_ne!(usage1, usage3);
    }

    #[test]
    fn test_usage_clone() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        let cloned = usage.clone();
        assert_eq!(usage, cloned);
    }

    #[test]
    fn test_usage_debug() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        let debug_str = format!("{:?}", usage);
        assert!(debug_str.contains("prompt_tokens"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_chat_response_accessors() {
        let response = ChatResponse {
            content: "Hello!".to_string(),
            session_id: "session-123".to_string(),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            completion_id: "chatcmpl-456".to_string(),
            created_at: 1234567890,
            finish_reason: "stop".to_string(),
            model: "myapp-123".to_string(),
        };

        assert_eq!(response.content(), "Hello!");
        assert_eq!(response.session_id(), "session-123");
        assert_eq!(response.usage().total_tokens, 15);
        assert_eq!(response.completion_id(), "chatcmpl-456");
        assert_eq!(response.created_at(), 1234567890);
        assert_eq!(response.finish_reason(), "stop");
        assert_eq!(response.model(), "myapp-123");
    }

    #[test]
    fn test_chat_response_clone() {
        let response = ChatResponse {
            content: "Hello!".to_string(),
            session_id: "session-123".to_string(),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            completion_id: "chatcmpl-456".to_string(),
            created_at: 1234567890,
            finish_reason: "stop".to_string(),
            model: "myapp-123".to_string(),
        };

        let cloned = response.clone();
        assert_eq!(response.content(), cloned.content());
        assert_eq!(response.usage().total_tokens, cloned.usage().total_tokens);
    }

    #[test]
    fn test_chat_response_from_internal() {
        // Simulate what the API returns
        let internal = ChatCompletionResponse {
            chat_session_id: "session-123".to_string(),
            id: "chatcmpl-456".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "myapp-123".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ResponseMessage {
                    role: "assistant".to_string(),
                    content: "Hello!".to_string(),
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        };

        let response: ChatResponse = internal.into();

        assert_eq!(response.content(), "Hello!");
        assert_eq!(response.session_id(), "session-123");
        assert_eq!(response.completion_id(), "chatcmpl-456");
        assert_eq!(response.created_at(), 1234567890);
        assert_eq!(response.model(), "myapp-123");
        assert_eq!(response.finish_reason(), "stop");
        assert_eq!(response.usage().prompt_tokens, 10);
        assert_eq!(response.usage().completion_tokens, 5);
        assert_eq!(response.usage().total_tokens, 15);
    }
}
