# chipp

[![Crates.io](https://img.shields.io/crates/v/chipp.svg)](https://crates.io/crates/chipp)
[![docs.rs](https://img.shields.io/docsrs/chipp)](https://docs.rs/chipp)
[![License](https://img.shields.io/crates/l/chipp.svg)](https://github.com/paulbreuler/chipp-rs#license)
[![CI](https://github.com/paulbreuler/chipp-rs/workflows/CI/badge.svg)](https://github.com/paulbreuler/chipp-rs/actions)
[![codecov](https://codecov.io/gh/paulbreuler/chipp-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/paulbreuler/chipp-rs)
[![MSRV](https://img.shields.io/badge/MSRV-1.83-blue)](https://github.com/paulbreuler/chipp-rs)
[![Downloads](https://img.shields.io/crates/d/chipp.svg)](https://crates.io/crates/chipp)

Rust client for the [Chipp.ai](https://chipp.ai) API - OpenAI-compatible chat completions with streaming support.

## Features

- ✅ **Non-streaming chat**: Simple request/response with `chat()`
- ✅ **Streaming chat**: Real-time text streaming with `chat_stream()`
- ✅ **Health checks**: API connectivity testing with `is_healthy()` and `ping()`
- ✅ **Session management**: Automatic `chatSessionId` tracking for conversation continuity
- ✅ **Automatic retries**: Exponential backoff for transient failures
- ✅ **Configurable timeouts**: Per-request timeout configuration
- ✅ **Correlation IDs**: Automatic UUID generation for request tracing
- ✅ **Comprehensive error handling**: Typed errors with context
- ✅ **Full async/await**: Built on `tokio` and `reqwest`
- ✅ **Security-first**: API keys redacted from Debug output

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
chipp = "0.1.1" # x-release-please-version
tokio = { version = "1", features = ["full"] }
```

Or install via cargo:

```bash
cargo add chipp tokio --features tokio/full
```

## Quick Start

```rust
use chipp::{ChippClient, ChippConfig, ChippMessage, ChippSession};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the builder pattern for ergonomic configuration
    let config = ChippConfig::builder()
        .api_key(std::env::var("CHIPP_API_KEY")?)
        .model(std::env::var("CHIPP_APP_NAME_ID")?)
        .build()?;

    let client = ChippClient::new(config)?;
    let mut session = ChippSession::new();

    let response = client
        .chat(&mut session, &[ChippMessage::user("Hello!")])
        .await?;

    println!("Response: {}", response);

    Ok(())
}
```

## Examples

### Non-Streaming Chat

```rust
use chipp::{ChippClient, ChippConfig, ChippMessage, ChippSession, MessageRole};

let client = ChippClient::new(config)?;
let mut session = ChippSession::new();

let messages = vec![ChippMessage {
    role: MessageRole::User,
    content: "Tell me a joke".to_string(),
}];

let response = client.chat(&mut session, &messages).await?;
println!("{}", response);
```

### Streaming Chat

```rust
use chipp::{ChippClient, ChippMessage, MessageRole};
use futures::StreamExt;

let mut stream = client.chat_stream(&mut session, &messages).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(text) => print!("{}", text),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Session Continuity

The client automatically manages `chatSessionId` for conversation continuity:

```rust
// First message
let messages1 = vec![ChippMessage::user("Remember this number: 42")];
client.chat(&mut session, &messages1).await?;

// Second message (remembers context)
let messages2 = vec![ChippMessage::user("What number did I tell you?")];
let response = client.chat(&mut session, &messages2).await?;
// Response will mention "42"
```

### Health Checks (Offline-First Apps)

Check API connectivity before routing requests:

```rust
use std::time::Duration;

// Check if API is reachable
if client.is_healthy().await? {
    // Route to Chipp API
    let response = client.chat(&mut session, &messages).await?;
} else {
    // Fall back to local LLM
}

// Measure API latency
let latency = client.ping().await?;
if latency < Duration::from_secs(2) {
    println!("Low latency: {:?}", latency);
}
```

## Running Examples

Set your API credentials:

```bash
export CHIPP_API_KEY="your-api-key"
export CHIPP_APP_NAME_ID="your-app-name-id"
```

Run the examples:

```bash
# Simple non-streaming example
cargo run --example simple

# Streaming example
cargo run --example streaming

# Session continuity example
cargo run --example session

# Error handling example (demonstrates retry logic, fallback strategies, etc.)
cargo run --example error_handling
```

## Configuration

### Using the Builder Pattern (Recommended)

```rust
use chipp::ChippConfig;

let config = ChippConfig::builder()
    .api_key("YOUR_API_KEY_HERE")
    .model("your-app-name-id")
    .timeout(std::time::Duration::from_secs(60))  // Optional: default is 30s
    .max_retries(5)                                // Optional: default is 3
    .build()?;
```

### Direct Struct Initialization

```rust
use chipp::ChippConfig;
use std::time::Duration;

let config = ChippConfig {
    api_key: "YOUR_API_KEY_HERE".to_string(),
    model: "your-app-name-id".to_string(),
    base_url: "https://app.chipp.ai/api/v1".to_string(),  // Default
    timeout: Duration::from_secs(30),                      // Default
    max_retries: 3,                                        // Default
    initial_retry_delay: Duration::from_millis(100),       // Default
    max_retry_delay: Duration::from_secs(10),              // Default
};
```

**Configuration Options:**

- `api_key` (required): Your Chipp API key from the Share → API tab
- `model` (required): Your appNameId from the Chipp dashboard
- `base_url`: API endpoint (default: `https://app.chipp.ai/api/v1`)
- `timeout`: Request timeout (default: 30 seconds)
- `max_retries`: Maximum retry attempts for transient failures (default: 3)
- `initial_retry_delay`: Initial backoff delay (default: 100ms)
- `max_retry_delay`: Maximum backoff delay (default: 10 seconds)

## Error Handling

```rust
use chipp::ChippClientError;

match client.chat(&mut session, &messages).await {
    Ok(response) => println!("Success: {}", response),
    Err(ChippClientError::ApiError { status, message }) => {
        eprintln!("API error {}: {}", status, message);
    }
    Err(ChippClientError::HttpError(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Security Best Practices

This SDK is designed with security in mind. Follow these best practices to protect your API credentials:

### Never Hardcode API Keys

```rust
// ❌ BAD: Hardcoded API key
let config = ChippConfig::builder()
    .api_key("live_abc123...")  // NEVER do this!
    .build()?;

// ✅ GOOD: Load from environment variable
let config = ChippConfig::builder()
    .api_key(std::env::var("CHIPP_API_KEY")?)
    .build()?;
```

### Use Environment Variables

Store credentials in environment variables, not in source code or config files:

```bash
# Set in your shell or .env file (which is gitignored)
export CHIPP_API_KEY="your-api-key"
export CHIPP_APP_NAME_ID="your-app-id"
```

### Avoid Logging Configuration Objects

The `ChippConfig` struct implements a custom `Debug` trait that **redacts the API key**:

```rust
let config = ChippConfig::builder()
    .api_key("secret-key")
    .model("my-app")
    .build()?;

// Safe to log - API key is redacted
println!("{:?}", config);
// Output: ChippConfig { api_key: "[REDACTED]", base_url: "...", model: "my-app", ... }
```

However, avoid logging raw API key strings directly:

```rust
// ❌ BAD: Logging the raw API key
let api_key = std::env::var("CHIPP_API_KEY")?;
tracing::debug!("Using API key: {}", api_key);  // NEVER do this!

// ✅ GOOD: Log without exposing secrets
tracing::info!("Initializing Chipp client");
```

### Production Recommendations

- **Use a secrets manager** (AWS Secrets Manager, HashiCorp Vault, etc.) for production deployments
- **Rotate API keys regularly** and update your environment variables
- **Audit logs** to ensure no sensitive data is accidentally logged
- **Use `.gitignore`** to exclude `.env` files from version control

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests (requires API key)

```bash
export CHIPP_API_KEY="your-api-key"
export CHIPP_APP_NAME_ID="your-app-name-id"
cargo test --features integration-tests -- --ignored
```

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/chipp).

Build documentation locally:

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/paulbreuler/chipp-rs.git
   cd chipp-rs
   ```

2. Run tests:
   ```bash
   cargo test
   ```

3. Run examples:
   ```bash
   export CHIPP_API_KEY="your-api-key"
   export CHIPP_APP_NAME_ID="your-app-name-id"
   cargo run --example simple
   ```

### Code Quality

Before submitting a PR, please ensure:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Run tests
cargo test

# Build docs
cargo doc --no-deps
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:

- Setting up pre-commit hooks
- Code quality standards
- Testing requirements
- Pull request process

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Links

- [Chipp.ai](https://chipp.ai) - Official website
- [API Documentation (docs.rs)](https://docs.rs/chipp) - Full Rust API reference
- [Crates.io](https://crates.io/crates/chipp) - Package registry
- [GitHub Repository](https://github.com/paulbreuler/chipp-rs) - Source code and issues
