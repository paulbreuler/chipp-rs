---
description: Heal and update .augment/rules/ files to align with latest Augment Code official docs and chipp-rs SDK best practices
argument-hint: [optional: specific rule file to heal, e.g., "audio-processing"]
model: sonnet4.5
allowed-tools: [Read, Write, WebSearch, WebFetch]
---

<objective>
Perform recurring maintenance on all `.augment/rules/*.md` files so they stay synchronized with the latest Augment Code / Auggie CLI documentation, release notes, and chipp-rs SDK best practices.

This command only updates rule files under `.augment/rules/` and never touches source code or other documentation locations directly.
</objective>

<input>
Use `$ARGUMENTS` to control which rule files to heal:

- No arguments: heal **all** rule files under `.augment/rules/`:
  - `00-sdk-core.md`
  - `10-library-publishing.md`
  - `60-architecture-decisions.md`
  - `70-teaching-mode.md`
  - `80-code-quality-assessment.md`
  - `90-documentation-cleanup.md`
  - `chipp-api-integration.md`
- Single argument (e.g. `audio-processing`, `audio-processing.md`, or substrings like `audio`): heal only that rule file.

When a specific rule is requested, accept both forms **with** and **without** the `.md` extension and normalize to the exact filename under `.augment/rules/`.
</input>

<scope>

- In-scope: `.augment/rules/*.md` only.
- Out-of-scope:
  - `.augment/commands/*.md` (commands are healed separately via dedicated commands).
  - `./prompts/*.md` (managed separately).
  - ADRs, README.md, CHANGELOG.md, and any non-rule documentation.
- Any temporary analysis documents created as part of the research phase (e.g., `./analyses/augment-docs-latest.md`) are **ephemeral** and governed by `.augment/rules/90-documentation-cleanup.md`.
</scope>

<process>

<step1_select_rule_targets>
Determine which `.augment/rules/*.md` files to process.

1. If `$ARGUMENTS` is empty:
   - Target the full, explicit list of rule files enumerated in the Requirements.
2. If `$ARGUMENTS` contains a single token (e.g. `audio-processing` or `audio-processing.md`):
   - Strip any trailing `.md` extension.
   - Append `.md` and look for an exact filename match under `.augment/rules/`.
3. If the requested file does not exist, report an error and show the list of known rules, then stop.
4. For the final target set, verify each file is readable before proceeding.
</step1_select_rule_targets>

<step2_research_phase>
Before changing any rules, gather the latest official information about Augment Code and Auggie CLI.

1. Use WebSearch to find current official documentation pages under `docs.augmentcode.com`, focusing on:
   - Custom commands and command frontmatter.
   - Rules configuration (including `behavior:` and `description:` schemas).
   - Tool / allowed-tools naming and usage.
2. Use WebSearch again to locate **recent release notes / changelog entries for 2025** related to Augment Code and Auggie CLI.
3. For each high-signal result (documentation page or changelog entry):
   - Use WebFetch to retrieve its content.
   - Extract key points relevant to rule-writing, such as:
     - Supported command locations and any deprecated ones.
     - Updated rule frontmatter schemas or new rule types.
     - New tools or capabilities that can or should be referenced in rules.
     - Any breaking changes that affect how rules and commands should be written.
4. Summarize findings into a **temporary analysis file** `./analyses/augment-docs-latest.md` with, at minimum:
   - Date of research.
   - URLs of the documentation pages and release notes consulted.
   - A concise bullet list of **current best practices** for:
     - Rules frontmatter (`behavior:`, `description:`, other supported fields).
     - Command locations and naming (`.augment/commands/`, compatibility with `.claude/commands/`, etc.).
     - Any new or deprecated tools / allowed-tools.
   - Explicit note that this file is **ephemeral** and should be cleaned up per `.augment/rules/90-documentation-cleanup.md` (for example by running `/cleanup-temporary-analyses`) once rule healing is complete.
</step2_research_phase>

<step3_analyze_each_rule>
For each selected rule file:

1. Read the full file contents.
2. Compare its guidance against the current Augment docs summarized in `./analyses/augment-docs-latest.md`:
   - Detect **outdated patterns**, such as:
     - References to deprecated command directories (e.g., `~/.claude/commands/`) when official docs prefer `~/.augment/commands/` and `./.augment/commands/`.
     - Old or incorrect frontmatter formats for rules.
     - Superseded tool names or behaviors.
   - Identify **missing features** that should be mentioned in context, for example:
     - New allowed tools that are relevant to that rule.
     - New rule types or frontmatter options introduced in 2025.
   - Flag **incorrect assumptions** about Augment behavior (e.g., wrong precedence ordering, wrong default command lookup) that contradict current official docs.
   - Note any **stale external links** to documentation that have changed or moved.
3. Additionally, for SDK-specific rules:
   - Verify Rust library patterns are current (e.g., tokio async patterns, thiserror usage, reqwest).
   - Check that API design patterns align with Rust API Guidelines.
   - Ensure Chipp API integration patterns match current API documentation.
4. Record, per file, which of the above issues are present so they can be summarized in the output.
</step3_analyze_each_rule>

<step4_heal_each_rule>
Apply targeted edits to each rule file to bring it in line with current documentation and best practices.

For each rule file:

1. **Update frontmatter (if present)**:
   - Ensure it uses the current official schema for rules, including a correct `behavior:` value (e.g., `always` or `auto`) and a clear `description:`.
   - Remove or fix any deprecated or unsupported frontmatter fields.
2. **Fix outdated command / rule patterns**:
   - Replace references to legacy command directories (e.g., `~/.claude/commands/`) with the current recommended ones (`~/.augment/commands/`, `./.augment/commands/`), while still acknowledging compatibility layers if official docs say they remain supported.
   - Ensure references to `.augment/rules/` and any rule-discovery behavior match current precedence and search rules.
3. **Incorporate relevant new features**:
   - Where appropriate, mention new command or rule features introduced in 2025 (for example, improved frontmatter options, new allowed tools, or updated rule evaluation logic).
   - Only add features that are clearly documented and relevant to that specific rule file's scope.
4. **Correct inaccurate statements**:
   - Amend or remove any claims about Augment or Auggie behavior that conflict with the latest official docs.
   - Where behavior has changed, briefly note the updated behavior instead of preserving stale guidance.
5. **Refresh external links**:
   - Update any URLs that no longer match official documentation locations (e.g., old paths that now 404 or have been reorganized).
   - Prefer linking to canonical docs pages under `docs.augmentcode.com` and any officially maintained changelog sources.
6. **Update SDK-specific content**:
   - Verify Rust library patterns align with current best practices (tokio 1.x, thiserror 1.x, reqwest).
   - Update API design patterns to match Rust API Guidelines.
   - Refresh API documentation links for Chipp API.
7. **Honor documentation structure policies**:
   - Ensure each rule continues to respect the separation between ADRs, README.md, CHANGELOG.md, and inline code docs as described in `60-architecture-decisions.md` and `90-documentation-cleanup.md`.
   - Do not introduce new documentation locations that contradict that structure.

Apply edits using the Write/Save tools with minimal necessary changes to avoid rewriting rule narratives wholesale.
</step4_heal_each_rule>

<step5_cleanup_analysis>
Once all rule healing edits are complete:

1. Treat `./analyses/augment-docs-latest.md` as an **ephemeral** analysis artifact.
2. Offer the user a cleanup choice consistent with `.augment/rules/90-documentation-cleanup.md`:
   - Default behavior: delete or truncate the file after healing is successfully completed.
   - If the user explicitly wants to keep it, leave it intact or suggest creating an ADR if it represents durable architectural decisions rather than scratch analysis.
3. Reflect the cleanup decision in the final output summary.
</step5_cleanup_analysis>

<step6_summarize_changes>
Produce a consolidated summary describing what was healed.

For each processed rule file, report:

- Rule file name (e.g., `audio-processing.md`).
- Status: `unchanged` | `healed`.
- Specific categories of changes applied, such as:
  - `~ Updated frontmatter to current rules schema (behavior/description).`
  - `~ Replaced deprecated command path references with .augment/commands guidance.`
  - `+ Documented new Augment feature: [feature name / summary].`
  - `~ Corrected external documentation links.`
  - `~ Fixed inaccurate description of rule or command behavior.`
  - `~ Updated Rust patterns to current best practices.`
  - `~ Refreshed API design patterns.`

Also include in the final summary:

- The date/time of the research phase.
- The key documentation URLs and release notes versions used as the source of truth.
- A short statement confirming that, to the best of the research performed, all processed `.augment/rules/*.md` files now align with the latest available Augment Code documentation and chipp-rs SDK best practices.
</step6_summarize_changes>

</process>

<output>
Return a Markdown report that includes:

- A per-file summary as described in <step6_summarize_changes>.
- A section titled "Augment docs reference" that lists:
  - The primary docs.augmentcode.com URLs consulted.
  - Any Auggie CLI changelog or release notes URLs.
  - The date of the documentation snapshot.
- A short note indicating whether `./analyses/augment-docs-latest.md` was deleted or retained, and why.
</output>

<success_criteria>

- Command supports both modes:
  - No arguments → heals all rule files in `.augment/rules/`.
  - Single rule name → heals only the specified rule file, accepting both `audio-processing` and `audio-processing.md` forms.
- All processed rule files:
  - Reference **current** Augment Code patterns for command locations and rules.
  - Use correct and up-to-date frontmatter schemas where applicable.
  - Do not rely on deprecated features or incorrect descriptions of Augment behavior.
  - Have external documentation links that point to valid, current official docs.
  - Reflect current chipp-rs SDK Rust best practices and API design patterns.
- The final report clearly describes:
  - What was changed in each rule file.
  - Which outdated patterns were replaced.
  - Which new Augment features (if any) were incorporated.
  - Which version/date of official documentation was used as the reference.

</success_criteria>

