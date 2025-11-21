---
type: "agent_requested"
description: "Documentation cleanup policy - ephemeral vs permanent files, aggressive cleanup of temporary documentation"
---

# Documentation Cleanup and Ephemeral Files

## Rule

**Agents must aggressively clean up temporary documentation while preserving intentional, long-lived docs.**

Use this rule together with `60-architecture-decisions.md` (ADRs) and `00-sdk-core.md` (SDK core principles).

## Classification

### Ephemeral (delete after task completion)

These are **working artifacts** created to support a specific task:

- Files in `./analyses/` (e.g., `./analyses/*.md`) unless the user explicitly marks them to keep
- Scratch/working `.md` files created during a single prompt or command run
- Intermediate documents generated only to unblock a specific implementation or investigation step
- Temporary test output files, benchmark results, profiling data

Assume ephemeral unless:

- The user explicitly says "keep this" or "treat this as permanent", or
- The file is moved into one of the permanent locations listed below.

### Permanent (keep and maintain)

These are **intentional documentation artifacts**:

- `README.md` (installation, quick start, examples)
- `CHANGELOG.md` (version history)
- ADRs in `docs/adr/` (immutable once accepted; see ADR rules)
- Inline code documentation (doc comments in Rust code)
- Test fixtures in `tests/fixtures/` (test data, configuration examples)

If content belongs in one of these locations, **move or rewrite it there** instead of leaving stray `.md` files elsewhere.

## Agent Behavior

### Before creating ANY `.md` file

- Ask: "Is this an ephemeral working note, or permanent documentation?"
- Prefer **ephemeral** unless the user explicitly wants a permanent doc.
- If permanent:
  - ADR → follow `60-architecture-decisions.md` and place under `docs/adr/`.
  - Setup / getting started / examples → `README.md`.
  - Version history → `CHANGELOG.md`.
  - Implementation details → inline code docs.

If none of these apply, strongly prefer **not** creating a new `.md` file.

### During a task

- Keep track (mentally) of any `.md` files you create or materially change, especially under `./analyses/`.
- Prefer reusing or updating existing permanent docs over creating new ones.

### After completing ANY task

- Identify ephemeral candidates:
  - New files in `./analyses/` created during the task.
  - Scratch `.md` files that are not `README.md`, `CHANGELOG.md`, or under `docs/adr/`.
  - Temporary test output, benchmark results, profiling data.
- Offer cleanup:
  - Summarize the list: "The following temporary files were created: ...".
  - Ask the user: "Delete these temporary files now? (Y/n)" with **default = delete**.
  - If the user agrees or accepts default, delete them (or, if deletion is unavailable, overwrite with a short stub indicating they were intentionally cleaned up).
- When in doubt about a specific file, **ask the user** before deleting.

## Alignment with ADRs

For architectural decisions:

- **Before major architectural decisions**: scan `docs/adr/` for existing ADRs so you don't contradict previous decisions.
- **After landing impactful features**: create an ADR _if_ the feature involves a significant architectural decision.

ADRs capture the stable "how" and "why". Code + inline docs implement it. README.md explains "what" and "how to use".

## Principle

"**Leave the codebase cleaner than you found it.**"

- Don't leave behind scratch `.md` files.
- Consolidate duplicated or stale docs into the correct permanent location.
- Prefer small, intentional documentation changes that make it easier for the next engineer (or agent) to understand the system.

## Examples

### Ephemeral Files (delete after task)

```text
./analyses/streaming-format-investigation.md
./analyses/retry-strategy-comparison.md
./benchmark-results-2024-11-20.md
./profiling-output.txt
./scratch-notes.md
```

### Permanent Files (keep and maintain)

```text
README.md
CHANGELOG.md
docs/adr/001-reqwest-for-http-client.md
src/lib.rs  (with doc comments)
tests/fixtures/test-response.json
```

## Cleanup Workflow

After completing a task:

1. **Identify**: List all `.md` files created during the task
2. **Classify**: Ephemeral vs permanent
3. **Consolidate**: Move permanent content to correct location
4. **Offer cleanup**: Ask user to confirm deletion of ephemeral files
5. **Delete**: Remove ephemeral files if user confirms (default: yes)

Example:

```text
✅ Task complete: Implemented retry strategy with exponential backoff

The following temporary files were created:
- ./analyses/retry-strategy-investigation.md
- ./benchmark-retry-timings.md

Delete these temporary files now? (Y/n): _
```
