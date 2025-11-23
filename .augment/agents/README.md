# Augment Subagents for chipp-rs

This directory contains specialized Augment subagents for code review and other tasks.

## Code Review Subagents

### PM Reviewer (`code-review-pm`)

**Focus**: Requirements validation, user impact, scope

- Validates acceptance criteria are met
- Checks user-facing changes are documented
- Verifies scope is appropriate
- Reviews error messages for user-friendliness

**Invoke**:

```
> Use the code-review-pm subagent to review this PR from a product perspective
```

### Engineer Reviewer (`code-review-engineer`)

**Focus**: SDK API design, code quality, security, Rust best practices

- Checks SDK API consistency and usability
- Reviews error handling (Result/Option, no unwrap in library code)
- Validates security (input validation, credential leaks, safe defaults)
- Verifies Rust idioms (ownership, borrowing, lifetimes)
- Checks dependencies are justified and MSRV-compatible

**Invoke**:

```
> Use the code-review-engineer subagent to review the security of this code
```

### QA Reviewer (`code-review-qa`)

**Focus**: Test coverage, edge cases, regression risks

- Verifies unit tests for business logic
- Checks integration tests for API endpoints
- Identifies missing edge case coverage
- Assesses test quality (arrange-act-assert)
- Flags regression risks

**Invoke**:

```
> Use the code-review-qa subagent to check test coverage for this feature
```

## Multi-Agent Review Workflow

**Sequential (recommended for comprehensive reviews)**:

```
> First use code-review-pm to validate requirements, then code-review-engineer to check architecture, then code-review-qa to verify tests
```

**Parallel (faster, independent reviews)**:

```
> Use code-review-pm, code-review-engineer, and code-review-qa subagents in parallel to review PR #42
```

## Output Format

Each subagent returns structured markdown:

```markdown
# [Role] Review: [PR Title]

## Summary

[Approved/Needs Changes/Blocked - 2-3 sentences]

## Blocking Issues

- ❌ [Critical issue with specific reference]

## Recommended Changes

- ⚠️ [Important but not blocking]

## Positive Observations

- ✅ [What was done well]

## Questions

- ❓ [Clarification needed]
```

## How Subagents Work

- **Separate context**: Each subagent has its own context window
- **Specialized prompts**: Tailored system prompts for each role
- **Tool access**: Can use Read, Grep, Glob, GitHub, Bash
- **Automatic invocation**: Augment can invoke them automatically when appropriate
- **Explicit invocation**: You can request specific subagents

## Adding New Subagents

1. Create a new `.md` file in this directory
2. Follow the format:

```markdown
---
name: your-subagent-name
description: When this subagent should be used
tools: Read, Grep, Glob
model: sonnet
---

# Your Subagent Title

System prompt and instructions...
```

3. Test with: `> Use the your-subagent-name subagent to...`

## References

- [Augment Subagents Docs](https://docs.augmentcode.com/cli/subagents)
- [chipp-rs SDK Rules](.augment/rules/)
