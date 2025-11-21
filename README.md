# chipp

[![Crates.io](https://img.shields.io/crates/v/chipp.svg)](https://crates.io/crates/chipp)
[![Documentation](https://docs.rs/chipp/badge.svg)](https://docs.rs/chipp)
[![License](https://img.shields.io/crates/l/chipp.svg)](https://github.com/paulbreuler/chipp-rs#license)
[![Build Status](https://github.com/paulbreuler/chipp-rs/workflows/CI/badge.svg)](https://github.com/paulbreuler/chipp-rs/actions)

Rust client for the [Chipp.ai](https://chipp.ai) API - OpenAI-compatible chat completions with streaming support.

## Features

- ✅ **Non-streaming chat**: Simple request/response with `chat()`
- ✅ **Streaming chat**: Real-time text streaming with `chat_stream()`
- ✅ **Session management**: Automatic `chatSessionId` tracking for conversation continuity
- ✅ **Configurable timeouts**: Per-request timeout configuration
- ✅ **Correlation IDs**: Automatic UUID generation for request tracing
- ✅ **Comprehensive error handling**: Typed errors with context
- ✅ **Full async/await**: Built on `tokio` and `reqwest`
- ✅ **Production-ready**: Comprehensive tests and documentation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
chipp = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use chipp::{ChippClient, ChippConfig, ChippMessage, ChippSession, MessageRole};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ChippConfig {
        api_key: std::env::var("CHIPP_API_KEY")?,
        base_url: "https://app.chipp.ai/api/v1".to_string(),
        model: "your-app-name-id".to_string(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };

    let client = ChippClient::new(config);
    let mut session = ChippSession::new();

    let messages = vec![ChippMessage {
        role: MessageRole::User,
        content: "Hello!".to_string(),
    }];

    let response = client.chat(&mut session, &messages).await?;
    println!("Response: {}", response);

    Ok(())
}
```

## Examples

### Non-Streaming Chat

```rust
use chipp::{ChippClient, ChippConfig, ChippMessage, ChippSession, MessageRole};

let client = ChippClient::new(config);
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
let messages1 = vec![ChippMessage {
    role: MessageRole::User,
    content: "Remember this number: 42".to_string(),
}];
client.chat(&mut session, &messages1).await?;

// Second message (remembers context)
let messages2 = vec![ChippMessage {
    role: MessageRole::User,
    content: "What number did I tell you?".to_string(),
}];
let response = client.chat(&mut session, &messages2).await?;
// Response will mention "42"
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
```

## Configuration

```rust
pub struct ChippConfig {
    pub api_key: String,        // Your Chipp API key
    pub base_url: String,       // Default: "https://app.chipp.ai/api/v1"
    pub model: String,          // Your appNameId from Chipp dashboard
    pub timeout: Duration,      // Request timeout (default: 30s)
    pub max_retries: usize,     // Max retry attempts (default: 3)
}
```

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

## Important Notes

### Streaming Format

⚠️ **The actual Chipp API streaming format differs from the official documentation!**

- **Documentation shows**: Standard SSE with `data:` prefix and OpenAI-compatible JSON chunks
- **Actual API returns**: Custom format with prefixes like `0:`, `e:`, `d:`, `f:`, `8:`

This client handles the actual format automatically - you don't need to worry about the format differences.

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

## Acknowledgments

- Built for the [Chipp.ai](https://chipp.ai) platform
- Originally developed as part of the [Chipp Edge Companion](https://github.com/paulbreuler/chipp-edge-companion) project

## Links

- [Chipp.ai](https://chipp.ai) - Official website
- [Chipp API Documentation](https://chipp.ai/docs/api) - API docs
- [crates.io](https://crates.io/crates/chipp) - Crate registry
- [docs.rs](https://docs.rs/chipp) - Documentation
- [GitHub](https://github.com/paulbreuler/chipp-rs) - Source code
