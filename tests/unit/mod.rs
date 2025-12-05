//! Unit tests for chipp-rs SDK
//!
//! Tests are organized by functionality:
//! - client_new_tests: ChippClient::new() constructor tests
//! - chat_tests: ChippClient::chat() method tests
//! - streaming_tests: ChippClient::chat_stream() method tests
//! - security_tests: Security-critical behavior tests (API key redaction, etc.)

mod chat_tests;
mod client_new_tests;
mod config_tests;
mod security_tests;
mod streaming_tests;
mod types_tests;
