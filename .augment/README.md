# Augment Configuration for chipp-rs

This directory contains Augment rules that guide AI-assisted development for the chipp-rs Rust SDK.

## Structure

Rules are organized in `.augment/rules/` with YAML frontmatter defining their behavior:

```yaml
---
description: Brief description of when this rule applies
behavior: always | auto | manual
---
```

### Always Rules

Applied to every Agent and Chat interaction. Use for core principles and non-negotiable patterns.

- `00-sdk-core.md` - SDK core principles, library design standards, code quality
- `10-library-publishing.md` - Publishing standards for crates.io, semver, changelog
- `chipp-api-integration.md` - Chipp API implementation patterns and session management

### Auto Rules

Automatically attached when Augment detects relevant context based on the `description` field.

- `60-architecture-decisions.md` - ADR format and documentation structure policy
- `70-teaching-mode.md` - Explain-as-you-build educational approach
- `80-code-quality-assessment.md` - Honest code quality assessment standards
- `90-documentation-cleanup.md` - Ephemeral vs permanent documentation management

### Manual Rules

Invoked explicitly via @-mention in Chat. Use for specialized contexts or optional guidance.

(None currently defined)

## Rule Types

- **`behavior: always`**: Automatically included in every Augment prompt
- **`behavior: auto`**: Augment detects and attaches based on description field
- **`behavior: manual`**: Must be manually @-mentioned in chat

## Usage

### In Augment Code

Rules are automatically loaded from this directory. No manual import needed.

- **Always rules** are included in every prompt
- **Auto rules** are attached when context matches the description
- **Manual rules** can be @-mentioned in Chat

### Editing Rules

1. Edit the `.md` files directly in `.augment/rules/`
2. Augment will pick up changes automatically
3. Keep rules actionable, specific, and tied to project goals

### Adding New Rules

1. Create a new `.md` file in `.augment/rules/`
2. Add YAML frontmatter with `description` and `behavior`
3. Write clear, concise guidelines
4. Update this README if adding a new category

## Limits

- **User Guidelines**: 24,576 characters max
- **Workspace Rules**: 49,512 characters max (combined)
- Rules are applied in order: manual → always → auto

## Grounding Documents

All rules are grounded in the following documents:

- `README.md` - Installation, quick start, examples
- `CHANGELOG.md` - Version history and release notes
- `docs/adr/` - Architecture Decision Records (immutable decisions)
- Inline code documentation in `src/`

## Commands

chipp-rs includes custom commands in `.augment/commands/` for common workflows:

### Prompt Management

- **`/create-prompt [task description]`** - Expert prompt engineer that creates optimized, XML-structured prompts for Rust SDK development
  - Adaptive requirements gathering with contextual questions
  - SDK-specific templates (coding, API design, testing, documentation)
  - Automatic numbering and organization in `./prompts/`

- **`/run-prompt [number(s)] [--parallel|--sequential]`** - Execute one or more prompts as delegated sub-tasks
  - Single prompt execution (default: most recent)
  - Parallel execution for independent tasks
  - Sequential execution for dependent tasks
  - Automatic archiving to `./prompts/completed/`

### Documentation Maintenance

- **`/heal-docs [readme|adrs|ADR-number]`** - Heal and update core documentation
  - README link checking and structure validation
  - Recent Proposed ADRs (within 7 days)
  - Never modifies Accepted/Deprecated/Superseded ADRs

- **`/heal-rules [rule-name]`** - Heal and update `.augment/rules/` files
  - Sync with latest Augment Code documentation
  - Update Rust SDK best practices
  - Refresh API specifications and library design patterns
  - Verify frontmatter schemas

### Cleanup

- **`/cleanup-temporary-analyses [--dry-run] [--keep-pattern <glob>]`** - Clean up ephemeral analysis files
  - Scans `./analyses/*.md` for temporary files
  - Dry-run mode for preview
  - Pattern-based exclusions
  - Aligned with `90-documentation-cleanup.md` policy

### Usage Examples

```bash
# Create a prompt for implementing retry logic
/create-prompt implement exponential backoff retry logic for API calls

# Run the most recent prompt
/run-prompt

# Run multiple prompts in parallel
/run-prompt 005 006 007 --parallel

# Heal all documentation
/heal-docs

# Heal only recent Proposed ADRs
/heal-docs adrs

# Clean up temporary analysis files
/cleanup-temporary-analyses
```

## Migration from Alder

This configuration was adapted from the Alder project's `.augment/` structure, tailored for Rust SDK development. Key adaptations:

- Replaced application patterns with library design patterns
- Focused on API design, error handling, async patterns, and streaming
- Added publishing standards for crates.io
- Maintained core principles: extreme ownership, ruthless simplicity, clean architecture
- Adapted commands for SDK workflows (API design, testing, documentation, publishing)

## References

- [Augment Rules Documentation](https://docs.augmentcode.com/setup-augment/guidelines)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Chipp API Reference](https://chipp.ai/docs/api/reference)
