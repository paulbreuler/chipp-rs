# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Non-streaming chat completions via `chat()`
- Streaming chat completions via `chat_stream()` with Server-Sent Events
- Automatic session management with `chatSessionId` tracking
- Retry logic with configurable exponential backoff
- Configurable request timeouts
- Comprehensive error types with `ChippClientError`
