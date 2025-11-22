# TDD Coverage Improvement for chipp-rs

## Objective

Increase test coverage from **53.66%** to **80%** using Test-Driven Development (TDD) principles with RGR (Red-Green-Refactor) style and AAA (Arrange-Act-Assert) pattern.

## Current State Analysis

### Coverage Report (53.66% - 246 lines total, 114 missed)

Run this command to see detailed coverage:
```bash
just coverage
# Opens target/llvm-cov/html/index.html in browser
```

### Files to Analyze

1. **`src/lib.rs`** - Main SDK implementation
   - `ChippClient::new()` - Constructor and validation
   - `chat()` - Main chat method with retry logic
   - `chat_attempt()` - Single attempt helper
   - `chat_stream()` - Streaming functionality (likely uncovered)
   - `is_retryable_error()` - Error classification
   - `create_backoff()` - Backoff configuration
   - Error types and conversions

2. **`tests/integration_test.rs`** - Current integration tests
   - Review existing test coverage
   - Identify gaps

3. **`examples/`** - Example code (not counted in coverage)
   - Can inform test scenarios

## Phase 1: Cleanup and Analysis

### Tasks

1. **Generate and review current coverage report**
   ```bash
   just coverage
   ```
   - Open `target/llvm-cov/html/index.html`
   - Identify all uncovered lines in `src/lib.rs`
   - Document uncovered code paths

2. **Review existing tests**
   - Analyze `tests/integration_test.rs`
   - Analyze unit tests in `src/lib.rs`
   - Identify redundant or poorly structured tests
   - Document what's already covered

3. **Clean up existing tests**
   - Refactor tests to follow AAA pattern strictly
   - Add clear documentation comments
   - Remove any redundant tests
   - Ensure test names clearly describe behavior

4. **Create test plan document**
   - List all uncovered code paths
   - Prioritize by risk/importance
   - Group related test cases
   - Estimate number of tests needed to reach 80%

## Phase 2: TDD Test Writing (RGR Style)

### RGR (Red-Green-Refactor) Process

For each uncovered code path:

1. **RED**: Write a failing test first
   - Test should fail because functionality isn't covered
   - Test should be clear about expected behavior
   - Run `cargo test` to confirm it fails (or passes if code already works)

2. **GREEN**: Verify test passes
   - If test already passes, great! Coverage increased
   - If test fails, fix the implementation
   - Run `cargo test` to confirm it passes

3. **REFACTOR**: Clean up
   - Improve test clarity
   - Remove duplication
   - Ensure AAA pattern is clear
   - Run `cargo test` to ensure still passing

### AAA (Arrange-Act-Assert) Pattern

Every test must follow this structure:

```rust
#[test]
fn test_descriptive_name_of_behavior() {
    // ARRANGE: Set up test data and dependencies
    let input = "test data";
    let expected = "expected result";
    
    // ACT: Execute the behavior being tested
    let actual = function_under_test(input);
    
    // ASSERT: Verify the outcome
    assert_eq!(actual, expected);
}
```

**Documentation style:**
```rust
/// Tests that ChippClient::new() validates API key is not empty
/// 
/// Arrange: Create config with empty API key
/// Act: Call ChippClient::new()
/// Assert: Returns error indicating invalid API key
#[test]
fn test_new_rejects_empty_api_key() {
    // Arrange
    let config = ChippConfig {
        api_key: String::new(),  // Empty API key
        ..Default::default()
    };
    
    // Act
    let result = ChippClient::new(config);
    
    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API key"));
}
```

## Phase 3: Priority Test Cases

### High Priority (Core Functionality)

1. **ChippClient::new() validation**
   - Empty API key
   - Empty model
   - Empty base URL
   - Valid configuration

2. **chat_stream() method**
   - Successful streaming
   - Stream error handling
   - Partial SSE events
   - Connection failures

3. **Error handling paths**
   - Network timeouts
   - DNS failures
   - Connection refused
   - Invalid JSON responses
   - Malformed SSE streams

4. **Session management**
   - Empty session
   - Session with history
   - Session message limits

### Medium Priority (Edge Cases)

5. **Retry logic edge cases**
   - Max retries = 0
   - Backoff delay edge cases
   - Retry on exactly 429 status
   - Retry on exactly 500 status
   - No retry on 400, 401, 403, 404

6. **Response parsing**
   - Missing fields in API response
   - Extra fields in API response
   - Empty content in response

### Low Priority (Nice to Have)

7. **Configuration edge cases**
   - Very short timeout (1ms)
   - Very long timeout (hours)
   - Custom base URLs

## Testing Strategy

### Use Mock HTTP Responses

Since we can't make real API calls in tests, use one of:

**Option 1: mockito** (HTTP mocking)
```bash
cargo add --dev mockito
```

**Option 2: wiremock** (More powerful HTTP mocking)
```bash
cargo add --dev wiremock
```

**Option 3: Manual mocking** (Create test doubles)
- More work but no dependencies
- Good for learning

### Test Organization

```
tests/
├── integration_test.rs          # Existing integration tests (keep)
├── unit/
│   ├── mod.rs
│   ├── client_tests.rs          # ChippClient::new() tests
│   ├── chat_tests.rs            # chat() method tests
│   ├── streaming_tests.rs       # chat_stream() tests
│   ├── retry_tests.rs           # Retry logic tests
│   ├── error_tests.rs           # Error handling tests
│   └── session_tests.rs         # Session management tests
```

## Success Criteria

- [ ] Coverage reaches 80% or higher
- [ ] All tests follow AAA pattern
- [ ] All tests have clear documentation comments
- [ ] Test names clearly describe behavior
- [ ] No redundant tests
- [ ] `just quality` passes with zero warnings
- [ ] `just coverage-check` passes

## Deliverables

1. **Coverage report** showing 80%+ coverage
2. **Test plan document** listing all test cases
3. **Clean, well-documented tests** following AAA pattern
4. **Updated CONTRIBUTING.md** with testing guidelines (if needed)

## Commands Reference

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Generate coverage report
just coverage

# Check coverage threshold
just coverage-check

# Run all quality checks
just quality
```

## Notes

- Focus on **behavior**, not implementation details
- Test **one thing** per test
- Use **descriptive test names** that read like documentation
- **Arrange** should be minimal - only what's needed for the test
- **Act** should be a single operation (or small sequence)
- **Assert** should verify one logical outcome (can have multiple assertions for same outcome)
- Add comments to explain **why** if the test setup is complex
- Avoid testing private implementation details - test public API behavior

