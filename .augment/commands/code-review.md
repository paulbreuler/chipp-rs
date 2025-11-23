---
description: Multi-perspective code review using PM, Engineer, and QA subagents
argument-hint: [PR number or file path]
---

Perform a comprehensive code review of $ARGUMENTS using three specialized subagents:

1. **First**, use the `code-review-pm` subagent to review from a product perspective
2. **Then**, use the `code-review-engineer` subagent to review architecture and code quality
3. **Finally**, use the `code-review-qa` subagent to review test coverage and quality

After all three reviews are complete, provide an **Executive Summary** in this format:

```markdown
# Code Review Summary: [PR/File Name]

## Overall Recommendation

[APPROVED / NEEDS CHANGES / BLOCKED]

## Critical Issues (Blocking)

- ❌ [Issue from any reviewer with file:line reference]

## Recommended Improvements

- ⚠️ [Non-blocking suggestions from any reviewer]

## Positive Highlights

- ✅ [Good patterns or practices observed]

## Next Steps

[Clear action items based on the reviews]
```

**Instructions for the review**:

- Run the three subagent reviews sequentially (PM → Engineer → QA)
- Each subagent should provide their structured review output
- Synthesize all three perspectives into the executive summary
- Highlight any consensus issues (mentioned by multiple reviewers)
- Flag any conflicting recommendations for discussion
