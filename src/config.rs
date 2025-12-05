//! Configuration for the Chipp API client.

use crate::error::ChippClientError;
use std::time::Duration;

/// Configuration for Chipp API client.
///
/// Use [`ChippConfigBuilder`] for ergonomic construction, or create directly.
///
/// # Example (Direct)
///
/// ```
/// use chipp::ChippConfig;
/// use std::time::Duration;
///
/// let config = ChippConfig {
///     api_key: "YOUR_API_KEY_HERE".to_string(),
///     model: "myapp-123".to_string(),
///     ..Default::default()
/// };
/// ```
///
/// # Example (Builder)
///
/// ```
/// use chipp::ChippConfig;
///
/// let config = ChippConfig::builder()
///     .api_key("YOUR_API_KEY_HERE")
///     .model("myapp-123")
///     .build()
///     .expect("Invalid config");
/// ```
#[derive(Clone)]
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

// SECURITY: Custom Debug implementation to prevent API key exposure in logs
impl std::fmt::Debug for ChippConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChippConfig")
            .field("api_key", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .field("initial_retry_delay", &self.initial_retry_delay)
            .field("max_retry_delay", &self.max_retry_delay)
            .finish()
    }
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

impl ChippConfig {
    /// Create a builder for `ChippConfig`.
    #[must_use]
    pub fn builder() -> ChippConfigBuilder {
        ChippConfigBuilder::default()
    }
}

/// Builder for [`ChippConfig`].
///
/// # Example
///
/// ```
/// use chipp::ChippConfig;
/// use std::time::Duration;
///
/// let config = ChippConfig::builder()
///     .api_key("YOUR_API_KEY_HERE")
///     .model("myapp-123")
///     .timeout(Duration::from_secs(60))
///     .build()
///     .expect("Invalid config");
/// ```
#[derive(Default)]
pub struct ChippConfigBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<usize>,
    initial_retry_delay: Option<Duration>,
    max_retry_delay: Option<Duration>,
}

// SECURITY: Custom Debug implementation to prevent API key exposure in logs
impl std::fmt::Debug for ChippConfigBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChippConfigBuilder")
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .field("initial_retry_delay", &self.initial_retry_delay)
            .field("max_retry_delay", &self.max_retry_delay)
            .finish()
    }
}

impl ChippConfigBuilder {
    /// Set the API key (required).
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the model/app name ID (required).
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the base URL (default: `https://app.chipp.ai/api/v1`).
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the request timeout (default: 30 seconds).
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum retry attempts (default: 3).
    #[must_use]
    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Set the initial retry delay (default: 100ms).
    #[must_use]
    pub fn initial_retry_delay(mut self, delay: Duration) -> Self {
        self.initial_retry_delay = Some(delay);
        self
    }

    /// Set the maximum retry delay (default: 10 seconds).
    #[must_use]
    pub fn max_retry_delay(mut self, delay: Duration) -> Self {
        self.max_retry_delay = Some(delay);
        self
    }

    /// Build the configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if required fields (`api_key`, `model`) are missing.
    pub fn build(self) -> Result<ChippConfig, ChippClientError> {
        let api_key = self
            .api_key
            .ok_or_else(|| ChippClientError::ConfigError("api_key is required".to_string()))?;
        let model = self
            .model
            .ok_or_else(|| ChippClientError::ConfigError("model is required".to_string()))?;

        let defaults = ChippConfig::default();

        Ok(ChippConfig {
            api_key,
            model,
            base_url: self.base_url.unwrap_or(defaults.base_url),
            timeout: self.timeout.unwrap_or(defaults.timeout),
            max_retries: self.max_retries.unwrap_or(defaults.max_retries),
            initial_retry_delay: self
                .initial_retry_delay
                .unwrap_or(defaults.initial_retry_delay),
            max_retry_delay: self.max_retry_delay.unwrap_or(defaults.max_retry_delay),
        })
    }
}
