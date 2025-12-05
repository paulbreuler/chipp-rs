//! Error types for the Chipp API client.

use thiserror::Error;

/// Errors that can occur when using the Chipp API client.
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
    ApiError {
        /// HTTP status code
        status: u16,
        /// Error message from API
        message: String,
    },

    /// SSE stream parsing error
    #[error("Stream parsing error: {0}")]
    StreamError(String),

    /// Maximum retry attempts exceeded
    #[error("Maximum retry attempts ({0}) exceeded")]
    MaxRetriesExceeded(usize),

    /// Configuration validation error
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Result type alias for Chipp operations.
pub type Result<T> = std::result::Result<T, ChippClientError>;
