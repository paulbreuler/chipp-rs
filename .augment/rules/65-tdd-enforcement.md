---
description: Enforce TDD workflow for all code changes
globs: ["src/**/*.rs", "tests/**/*.rs", "examples/**/*.rs"]
---

# TDD Enforcement

## MANDATORY WORKFLOW

For ANY code change that modifies behavior:

### 1. RED Phase (Write Failing Test First)

- Write a test that describes the expected behavior
- Run `just test` - tests MUST fail (or be new)
- Commit: `test: add failing test for <feature>`

### 2. GREEN Phase (Minimal Implementation)

- Write the MINIMUM code to make the test pass
- No extra features, no premature optimization
- Run `just test` - tests MUST pass
- Commit: `feat: implement <feature>` or `fix: resolve <bug>`

### 3. REFACTOR Phase (Improve with Safety Net)

- Clean up code while tests stay green
- Apply DRY, extract functions, improve naming
- Run `just test` after each change
- Commit: `refactor: improve <component>`

## Verification Commands

```bash
just test          # Run unit tests (fast feedback)
just test-all      # Run all tests including doc tests
just coverage-check # Verify ≥80% coverage threshold
```

## Test Location Conventions

| Code Location | Test Location |
|---------------|---------------|
| `src/lib.rs` (public API) | `tests/unit/*.rs` or inline `#[cfg(test)]` |
| `src/*.rs` (internal) | Inline `#[cfg(test)]` modules |
| Integration scenarios | `tests/integration_test.rs` |
| Example usage | `examples/*.rs` (verified by `cargo test --examples`) |

## Test Naming Convention

```rust
#[test]
fn test_<function>_<scenario>_<expected_result>() {
    // ARRANGE - set up test data
    // ACT - call the function
    // ASSERT - verify the result
}

// Examples:
fn test_new_valid_config_returns_ok() { ... }
fn test_chat_empty_messages_returns_error() { ... }
fn test_parse_stream_chunk_valid_data_extracts_content() { ... }
```

## What Requires TDD

✅ **MUST use TDD**:
- New public API methods
- Bug fixes (write test that reproduces bug first)
- Behavior changes to existing code
- New error types or error conditions

## Exceptions (No TDD Required)

❌ **Skip TDD for**:
- Documentation-only changes (`*.md`, rustdoc comments)
- Configuration files (`Cargo.toml`, `justfile`, `.github/`)
- Refactoring with existing test coverage
- Adding new dependencies (but test their usage)

## Example TDD Flow

### Bug Fix Example

```bash
# 1. RED: Write test that reproduces the bug
git add tests/unit/client_tests.rs
git commit -m "test: add failing test for panic on invalid timeout"

# 2. GREEN: Fix the bug
git add src/lib.rs
git commit -m "fix: handle invalid timeout gracefully"

# 3. REFACTOR: Clean up if needed
git add src/lib.rs
git commit -m "refactor: extract timeout validation to separate function"
```

### New Feature Example

```bash
# 1. RED: Write test for new behavior
git add tests/unit/streaming_tests.rs
git commit -m "test: add test for retry on connection reset"

# 2. GREEN: Implement the feature
git add src/streaming.rs
git commit -m "feat(streaming): add retry on connection reset"

# 3. REFACTOR: Improve implementation
git add src/streaming.rs
git commit -m "refactor(streaming): extract retry logic to helper"
```

## Enforcement

When assisting with code changes:

1. **Before writing implementation**: Ask "Where is the failing test?"
2. **If no test exists**: Write the test first, verify it fails
3. **After implementation**: Run `just test` to verify green
4. **Before PR**: Run `just coverage-check` to verify ≥80% coverage

