---
description: Heal and update core documentation (README, recent ADRs, architecture docs) to prevent staleness and enforce project policies
argument-hint: [readme|adrs|architecture|ADR-number]
model: sonnet4.5
allowed-tools: [Read, Write, Glob, WebFetch, AskUserQuestion]
---

<objective>
Perform recurring maintenance on chipp-rs core documentation so it stays current, lean, and aligned with SDK documentation policies.

This command focuses on **root documentation** (README, CHANGELOG) and **recent, Proposed ADRs**. It never edits Accepted/Deprecated/Superseded ADRs.
</objective>

<input>
Use `$ARGUMENTS` to control which documentation to heal:

- No arguments: heal **all** documentation types in scope (README, CHANGELOG, recent Proposed ADRs).
- `readme`: heal only the root `README.md`.
- `changelog`: heal only `CHANGELOG.md`.
- `adrs`: heal only **recent, Proposed** ADRs.
- `<ADR-number>` (e.g., `001`): heal only that ADR **if** it is `Proposed` and **recent** (see <recent_adrs_definition>).

Treat arguments as case-insensitive. If an ADR number is provided, normalize to three digits (e.g., `1` → `001`) and look for `docs/adr/001-*.md`.
</input>

<scope>

- In-scope:
  - Root `README.md`.
  - `CHANGELOG.md`.
  - ADRs under `docs/adr/*.md` with **Status = Proposed**.
- Out-of-scope:
  - Source code, tests, and inline documentation.
  - `.augment/rules/*.md` (healed by `/heal-rules`).
  - Prompt templates under `./prompts/` (managed separately).
- Any temporary analysis files created under `./analyses/` are **ephemeral** and governed by `.augment/rules/90-documentation-cleanup.md` and can be cleaned up by chaining `/cleanup-temporary-analyses`.
</scope>

<recent_adrs_definition>
An ADR is considered **recent** if:

- Its `**Status**` line is `Proposed`, **and**
- Its `**Last Updated**:` date (if present) is within the last 7 days (UTC), **or**
- If `Last Updated` is missing, its `**Date**:` is within the last 7 days (UTC).

Use the current UTC date when computing this 7-day window.
</recent_adrs_definition>

<process>

<step1_determine_targets>
Determine which documentation artifacts to process based on `$ARGUMENTS`.

1. Start with an empty set of targets.
2. If `$ARGUMENTS` is empty:
   - Include `README.md`, `CHANGELOG.md`, and recent Proposed ADRs.
3. If `$ARGUMENTS` is `readme`:
   - Target only `README.md`.
4. If `$ARGUMENTS` is `changelog`:
   - Target only `CHANGELOG.md`.
5. If `$ARGUMENTS` is `adrs`:
   - Target only recent Proposed ADRs.
6. If `$ARGUMENTS` parses as an ADR number:
   - Normalize to `NNN` (three digits).
   - Use Glob on `docs/adr/NNN-*.md` to find the corresponding ADR file.
   - If not found, report that the ADR does not exist and stop.
   - If found, only target that ADR, **and only if** it is `Proposed` and `recent` per <recent_adrs_definition>.
7. If multiple tokens are provided (e.g., `readme adrs`), treat them as a set and union the corresponding targets.

If no valid targets are found (e.g., requested ADR is not recent or not Proposed), return a clear message and do not modify any files.
</step1_determine_targets>

<step2_readme_analysis_and_healing>
If `README.md` is in the target set:

1. Use Read to load `README.md`.
2. Assess its structure against SDK best practices:
   - **Project description & value**: Confirm the intro clearly states what chipp-rs is and why it exists.
   - **Installation**: Confirm cargo installation instructions are present and accurate.
   - **Quick start / examples**: Confirm basic usage examples are present and compile.
   - **Pointers to deeper docs**: Confirm links to ADRs (`docs/adr/`) and docs.rs are present.
   - **Appropriate length**: Identify any sections that belong in other locations per `.augment/rules/60-architecture-decisions.md`:
     - Detailed architecture → ADRs.
     - API documentation → inline docs and docs.rs.
3. Detect content to extract:
   - Look for headings like `Architecture`, `Design`, or long implementation details.
   - For each such section, decide whether it should:
     - Stay in README (if it's core to getting started), or
     - Be shortened in-place and extracted into an ADR.
4. Internal link check:
   - For each Markdown link whose target does **not** start with `http`, treat it as a local path.
   - Use Read or Glob to verify that the referenced file exists.
   - Record any broken local links.
5. External link sanity check (optional but preferred):
   - For a small sample of external links, use WebFetch to confirm they return a valid page (2xx).
   - Record any clear 404s.
6. Apply **minimal edits** to `README.md`:
   - Fix obviously broken local links by correcting paths when the intended file can be inferred.
   - If a section is clearly better suited for an ADR:
     - Add a brief pointer (e.g., "See docs/adr/001-streaming-strategy.md for details") and trim excessive detail, or
     - Leave full text but record a suggestion to extract it in the summary if automated trimming would be too destructive.
   - Do **not** move or create `.md` files directly from this step; keep extraction suggestions in the final summary and/or ephemeral analysis notes.
7. Write back `README.md` only with safe, targeted edits (link fixes, small wording clarifications, brief pointers).
</step2_readme_analysis_and_healing>

<step3_select_recent_proposed_adrs>
If ADRs are in scope (`adrs` argument, ADR number, or default no-argument mode):

1. Use Glob on `docs/adr/*.md` to list ADR files (excluding `README.md`).
2. For each ADR file:
   - Use Read to load the file.
   - Parse the metadata lines:
     - `**Status**: ...`
     - `**Date**: YYYY-MM-DD`
     - `**Last Updated**: YYYY-MM-DD` (if present).
   - If `Status` is not `Proposed`, **skip** this ADR (immutable per `.augment/rules/60-architecture-decisions.md`).
   - For `Proposed` ADRs, compute whether they are **recent** using <recent_adrs_definition>.
3. Collect all ADRs that are both `Proposed` and recent.
4. If a specific ADR number was requested, intersect this set with the requested ADR; if it's not in the set, report that it is either not `Proposed` or not recent and skip healing.
</step3_select_recent_proposed_adrs>

<step4_heal_recent_proposed_adrs>
For each ADR selected in <step3_select_recent_proposed_adrs>:

1. **Update Last Updated**:
   - Ensure a `**Last Updated**: YYYY-MM-DD` line exists directly after the `**Date**:` line.
   - Set `Last Updated` to the current UTC date in `YYYY-MM-DD` format.
2. **Verify structure matches template** (`docs/adr/README.md`):
   - Confirm presence and ordering of sections:
     - `## Context and Problem Statement`
     - `## Decision Drivers`
     - `## Considered Options`
     - `## Decision Outcome`
     - `### Consequences`
     - `## References`
   - If a required section is missing, insert a minimal placeholder with a TODO-style comment (e.g., "TBD – fill in context before accepting this ADR").
   - Keep ADRs concise; avoid adding large new paragraphs.
3. **Ensure ADR is indexed**:
   - Read `docs/adr/README.md`.
   - Confirm there is a table row for this ADR number (e.g., `| [001](./001-gemini-api-for-asr-tts.md) | ... |`).
   - If missing, add a new row with status `Proposed` and the ADR's date.
4. **Validate references and links**:
   - For local links (e.g., `./002-something.md`, other ADRs), confirm those files exist.
   - For external documentation links, optionally use WebFetch to verify they resolve (2xx) and are still authoritative.
   - Record any broken links and, where clear, update them to current URLs.
5. Write back each ADR with minimal edits (Last Updated, added placeholders, fixed links, index row).

Never change the `Status` of an ADR from within this command.
</step4_heal_recent_proposed_adrs>

<step5_changelog>
If `CHANGELOG.md` is in scope:

1. Use Read to load `CHANGELOG.md`.
2. Check for:
   - Proper Keep a Changelog format.
   - Unreleased section at the top.
   - Broken links to GitHub issues, PRs, or commits.
   - Missing version entries for recent releases.
3. Apply minimal edits:
   - Fix broken links.
   - Ensure Unreleased section exists.
   - Verify format consistency.
4. Write back only if changes are needed.
</step5_changelog>

<step6_ephemeral_analysis_and_cleanup>
If you create any temporary analysis notes under `./analyses/` while performing link checks or outlining suggestions (e.g., `./analyses/heal-docs-report.md`):

1. Treat these files as **ephemeral** per `.augment/rules/90-documentation-cleanup.md`.
2. At the end of the command, use Glob to list any `./analyses/heal-docs-*.md` files created during this run.
3. Use AskUserQuestion:
   - header: "Cleanup heal-docs analysis files"
   - question: "Delete temporary heal-docs analysis files now? (Y/n)"
   - options:
     - "Y - Delete them" (default)
     - "n - Keep them for now"
4. If the user confirms deletion or accepts the default:
   - Delete each listed file, or overwrite with a short stub indicating it was intentionally cleaned up as ephemeral.
5. Reflect the cleanup decision in the final output.
</step6_ephemeral_analysis_and_cleanup>

<step7_summary_output>
Produce a consolidated Markdown summary of what was analyzed, changed, and suggested.

For each file type in scope, include:

- **README.md**:
  - Status: `unchanged` | `healed`.
  - List of sections updated (e.g., link fixes, new pointers, trimmed content).
  - Any broken links found and how they were resolved.
- **ADRs**:
  - For each healed ADR, show:
    - ADR file name and number (e.g., `001-gemini-api-for-asr-tts.md`).
    - Previous and new `Last Updated` date.
    - Whether any structural placeholders were added.
    - Link fixes applied.
  - Explicitly confirm that only `Proposed` ADRs were modified.
- **Architecture docs**:
  - For each healed doc, show:
    - File name (e.g., `software_architecture.md`).
    - Changes applied (link fixes, spec updates, ADR references added).

Also include:

- A list of **suggested new docs** that were not auto-created.
- A list of any **broken links** that could not be automatically fixed, so a human can address them.
- Confirmation that ADR timestamps (`Last Updated`) were updated with the current UTC date.
- A short note about whether temporary `heal-docs` analysis files were deleted or retained.
</step7_summary_output>

</process>

<success_criteria>

- Command supports all argument modes:
  - No arguments → heal README, CHANGELOG, and recent Proposed ADRs.
  - `readme` → only README.
  - `changelog` → only CHANGELOG.
  - `adrs` → only recent Proposed ADRs (or a specific ADR if a number is given).
  - `NNN` → only ADR `NNN` if `Proposed` and recent.
- Only `Proposed` ADRs are modified; Accepted/Deprecated/Superseded ADRs remain immutable.
- All healed ADRs have an accurate `Last Updated` field set to today's UTC date and are indexed in `docs/adr/README.md`.
- README has valid internal links and clear pointers to deeper documentation, without becoming bloated.
- CHANGELOG follows Keep a Changelog format with valid links.
- No new permanent `.md` files are created without explicit user confirmation.
- Any temporary analysis files created for this command are treated as ephemeral and either deleted or explicitly retained per `.augment/rules/90-documentation-cleanup.md`.

</success_criteria>
