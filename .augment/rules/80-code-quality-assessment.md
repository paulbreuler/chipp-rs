---
description: Honest code quality assessment - avoid marketing language, state facts about implementation status
behavior: always
---

# Code Quality Assessment: Honest and Direct

## Rule

**NEVER use marketing language to describe code quality. State facts about what works, what doesn't, and what's missing.**

## Forbidden Phrases

❌ **NEVER say**:
- "production-ready"
- "ready for handoff"
- "polished product"
- "enterprise-grade"
- "battle-tested"
- "robust implementation"
- "comprehensive solution"
- "fully-featured"
- "complete implementation"

✅ **Instead, state facts**:
- "All tests pass (35 tests, 0 failures)"
- "Builds without warnings on Rust 1.75"
- "Implements streaming and session management with error handling"
- "Missing: unit tests for SessionManager, integration tests for error paths"
- "Known limitation: requires network connectivity (documented)"
- "Tested with feature gates, integration tests behind `integration-tests` feature"

## Assessment Framework

### What to Report

**1. Build Status**
```
✅ Builds: cargo build --release (15.7s, 0 warnings)
❌ Builds: cargo build fails with 3 type errors in demo.rs
⚠️  Builds: cargo build succeeds with 2 clippy warnings
```

**2. Test Status**
```
✅ Tests: 35 passed, 0 failed (cargo test --workspace)
❌ Tests: 3 failed in integration tests
⚠️  Tests: All pass, but missing tests for error paths
```

**3. Implementation Completeness**
```
Implemented:
- TTS demo with execution logging
- ASR demo with file validation
- API key resolution from CLI/env/keychain

Missing:
- Unit tests for ExecutionLog methods
- Progress indicators for long API calls
- Retry logic for network failures
```

**4. Known Issues**
```
Known issues:
1. Playback fails on headless systems (documented as non-fatal)
2. No timeout for API calls (could hang indefinitely)
3. Large audio files (>10MB) not tested
```

**5. Documentation Status**
```
Documentation:
✅ All public APIs have doc comments
✅ Examples in doc comments compile
⚠️  Missing: troubleshooting guide for common errors
❌ Missing: ADR documenting Gemini API choice
```

## When Assessing Code Quality

### Be Specific, Not Vague

❌ **Vague**:
```
The implementation is solid and well-structured. The code is clean and follows
best practices. This is ready for production use.
```

✅ **Specific**:
```
Implementation status:
- Builds: ✅ No warnings (cargo build --release)
- Tests: ✅ 35 tests pass, 0 failures
- Docs: ✅ All public APIs documented
- Error handling: ✅ Uses Result<T> with context
- Input validation: ✅ File extensions validated
- Missing: Unit tests for ExecutionLog, progress indicators
```

### State Gaps Explicitly

❌ **Hiding gaps**:
```
The demo commands are complete and ready to use.
```

✅ **Stating gaps**:
```
Demo commands implement core functionality:
- TTS: text → audio file (works)
- ASR: audio file → transcript (works)

Gaps:
- No unit tests for SessionManager struct
- No integration tests for network failures
- No timeout handling for long API calls (>30s)
- Integration tests not yet run in CI
```

### Use Measurable Indicators

❌ **Unmeasurable**:
```
The code is fast and efficient.
```

✅ **Measurable**:
```
Performance (measured on macOS M1):
- Streaming first token: 180ms avg
- Complete response: 1.2s for 50-token response
- Session creation: 15ms

Not yet measured:
- Performance with large conversation histories (>100 messages)
- Memory usage under load
- Latency with slow network (simulated)
```

## Handling Uncertainty

### When You Don't Know

❌ **Guessing**:
```
This should work fine in production.
```

✅ **Stating uncertainty**:
```
This works in testing (unit tests, doc tests). Unknown:
- Behavior with very large conversation histories
- Performance under high concurrency
- Stability over long-running sessions

Recommend: Add integration tests and load testing before 1.0 release.
```

### When There Are Trade-offs

❌ **Hiding trade-offs**:
```
The implementation uses the best approach.
```

✅ **Explaining trade-offs**:
```
Implementation uses reqwest for HTTP client:

Advantages:
- Async/await support with tokio
- Built-in streaming support
- Well-maintained, widely used

Trade-offs:
- Adds tokio as required dependency
- Larger binary size than minimal HTTP clients
- Requires async runtime

Alternative: ureq (blocking, smaller binary, no async runtime)
```

## Summary Format

### End-of-Task Summary

```markdown
## Summary

**What was done**:
- Added doc comments to SessionManager struct and methods
- Implemented streaming response handling
- Added retry logic with exponential backoff
- Added session ID tracking for conversation continuity

**Verification**:
- ✅ Builds: cargo build --release (15.7s, 0 warnings)
- ✅ Tests: 35 passed, 0 failed
- ✅ Diagnostics: No compiler errors

**What works**:
- Streaming chat completions with proper SSE parsing
- Session management with conversation history
- API key configuration
- Error messages guide users to fix issues

**Known limitations**:
- No timeout handling for long-running requests
- No progress indicators for API calls >30s
- Integration tests not yet run in CI

**What's missing**:
- Unit tests for SessionManager methods
- Integration tests for network error paths
- Examples in documentation

**Next steps** (if continuing):
1. Add unit tests for SessionManager (2-3 hours)
2. Add integration tests to CI (1 hour)
3. Add timeout handling for long requests (1 hour)
```

## Remember

- **State facts, not opinions** - "All tests pass" vs "high quality"
- **Be explicit about gaps** - Don't hide missing tests or documentation
- **Use measurable indicators** - Build status, test counts, performance numbers
- **Acknowledge uncertainty** - Say "not tested" instead of "should work"
- **Explain trade-offs** - Every design choice has pros and cons
- **Never claim completeness** - There are always improvements possible

## When to Use Quality Indicators

✅ **Good uses**:
- "Builds without warnings"
- "All 35 tests pass"
- "Implements error handling with Result<T>"
- "Documented with examples"

❌ **Bad uses**:
- "Production-ready" (vague, unmeasurable)
- "Enterprise-grade" (marketing language)
- "Fully tested" (impossible to prove)
- "Complete implementation" (always more to do)

