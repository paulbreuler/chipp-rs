---
description: Architecture Decision Records (ADRs) format, documentation structure, and decision-making process for chipp-rs
behavior: auto
---

# Architecture Decision Records (ADRs)

## Rule

**Before making architectural decisions, ALWAYS review existing ADRs in `docs/adr/` to ensure consistency and avoid contradicting previous decisions.**

## ADR Format

All architectural decisions MUST be documented using the ADR format:

```markdown
# ADR-{number}: {Title}

**Status**: Proposed | Accepted | Deprecated | Superseded
**Date**: YYYY-MM-DD
**Deciders**: {Names}
**Technical Story**: {Brief description}

## Context and Problem Statement

{Describe the context and problem}

## Decision Drivers

- {Driver 1}
- {Driver 2}

## Considered Options

1. **{Chosen option}**
2. ~~{Rejected option}~~ - Rejected: {reason}

## Decision Outcome

**Chosen option**: **"{Option}"**
**Rationale**: {Why this option was chosen}

### Consequences

**Positive**:

- ✅ {Benefit}

**Negative**:

- ⚠️ {Drawback}

## References

- {Link to authoritative documentation}
```

## When to Create an ADR

Create an ADR for decisions about:

- **API Design**: Public API surface, method signatures, type design
- **Architecture**: Module boundaries, trait design, abstraction layers
- **Technology Stack**: HTTP client choice, async runtime, serialization library
- **Streaming**: SSE parsing strategy, Stream trait implementation
- **Error Handling**: Error type hierarchy, retry logic, timeout strategy
- **Session Management**: Session state tracking, ID persistence
- **Testing**: Integration test strategy, mocking approach
- **Dependencies**: Adding/removing dependencies, version constraints

## ADR Principles

- **One decision per ADR**: Keep ADRs focused on a single architectural decision
- **Concise**: ADRs should be brief (< 100 lines) - link to authoritative docs instead of explaining
- **Authoritative sources**: Always link to official documentation for technical claims
- **Immutable**: Once accepted, ADRs are not edited - create new ADR to supersede
- **Sequential numbering**: Use next available number (001, 002, 003...)
- **Kebab-case naming**: `{number}-{short-title}.md`

## Location

All ADRs are stored in: `docs/adr/`

Index file: `docs/adr/README.md`

## Documentation Structure Policy

**chipp-rs maintains a strict documentation structure to prevent documentation sprawl.**

### Allowed Documentation Locations

#### 1. ADRs (`docs/adr/`)

- **Purpose**: Architectural decisions (immutable, accepted decisions)
- **Characteristics**: Concise, focused, immutable once accepted
- **Examples**: HTTP client choice, streaming strategy, error type design
- **Status**: Proposed → Accepted → (optionally) Deprecated/Superseded

#### 2. README.md

- **Purpose**: Getting started, installation, quick start, examples
- **Characteristics**: User-facing, practical, maintained
- **Examples**: Installation instructions, basic usage, links to docs

#### 3. Inline Code Documentation

- **Purpose**: API documentation, implementation details
- **Format**: Doc comments (`///` in Rust)
- **Characteristics**: Lives with the code, explains WHY not just WHAT
- **Examples**: Function purpose, design decisions, usage examples

#### 4. CHANGELOG.md

- **Purpose**: Version history, changes per release
- **Format**: Keep a Changelog format
- **Characteristics**: Chronological, semver-aligned

### Prohibited Documentation

**DO NOT create these types of files:**

❌ **Architecture guides** (e.g., `ARCHITECTURE.md`, `DESIGN.md`)

- **Why**: Should be in ADRs
- **Alternative**: Create ADR for architectural decisions

❌ **Implementation guides** (e.g., `STREAMING.md`, `TESTING.md`)

- **Why**: Should be in ADRs or inline code docs
- **Alternative**: Create ADR for patterns, doc comments for specifics

❌ **API guides** (e.g., `API.md`, `USAGE.md`)

- **Why**: Should be in README.md and inline docs
- **Alternative**: Add to README.md or crate-level docs in `src/lib.rs`

❌ **Checklist/status files** (e.g., `FEATURE_CHECKLIST.md`, `TODO.md`)

- **Why**: Temporary tracking documents, not permanent documentation
- **Alternative**: Use GitHub Issues or Projects

### Decision Flow

```text
Idea/Research
    ↓
ADR (decision made, documented, immutable)
    ↓
Code + Inline Docs (implementation with doc comments)
    ↓
README.md (user-facing guide)
```

### When to Use Each Location

| Document Type          | Location         | Mutable?           | Example                           |
| ---------------------- | ---------------- | ------------------ | --------------------------------- |
| Architectural decision | `docs/adr/`      | No (once accepted) | ADR-001: reqwest for HTTP client  |
| Setup instructions     | `README.md`      | Yes                | Installation, quick start         |
| API documentation      | Inline code docs | Yes                | Function doc comments             |
| Implementation details | Inline code docs | Yes                | Design rationale in comments      |
| Version history        | `CHANGELOG.md`   | Yes                | Changes per release               |

### Enforcement

**Before creating ANY `.md` file:**

1. Ask: "Is this an architectural decision?" → ADR
2. Ask: "Is this setup/getting started?" → README.md
3. Ask: "Is this API documentation?" → Inline code docs
4. Ask: "Is this version history?" → CHANGELOG.md
5. If none of the above, **don't create the file**

**When you find policy violations:**

- Remove files that duplicate ADR content
- Consolidate setup info into README.md
- Convert implementation guides to inline code docs or ADRs
