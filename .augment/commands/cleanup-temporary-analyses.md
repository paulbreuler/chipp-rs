---
description: Clean up temporary analysis files and other ephemeral working docs after commands complete
argument-hint: [--dry-run] [--keep-pattern <glob>] [--path <relative-path>]
model: sonnet4.5
allowed-tools: [Read, Write, Glob, AskUserQuestion]
---

<objective>
Remove or neutralize temporary analysis files (especially under `./analyses/`) and other scratch `.md` files
that are not intended as permanent documentation, in line with `.augment/rules/90-documentation-cleanup.md`.
</objective>

<context>
Many commands (e.g., `/heal-docs`, `/heal-rules`, `/run-prompt`) create temporary analysis files under
`./analyses/` or other scratch `.md` files to support planning, research, or comparison.

Per `.augment/rules/90-documentation-cleanup.md`:

- Files in `./analyses/` are **ephemeral** unless explicitly promoted.
- Scratch `.md` files created during a single task should be cleaned up.
- Permanent docs must live in ADRs, README.md, CHANGELOG.md, or inline code docs.

This command centralizes cleanup so other commands can simply chain `/cleanup-temporary-analyses`
instead of re-implementing cleanup logic.
</context>

<input>
Use `$ARGUMENTS` to control cleanup behavior:

- No arguments: scan `./analyses/*.md` and propose deleting all of them.
- `--dry-run`: show what would be deleted/neutralized without changing anything.
- `--keep-pattern <glob>`: keep files matching the glob (e.g., `--keep-pattern "*baseline*"`).
- `--path <relative-path>`: additionally scan a specific directory for scratch `.md` files.
</input>

<process>

<step1_discover_candidates>

1. Use Glob to list `./analyses/*.md` (if directory exists).
2. If `--path` is provided, use Glob to list `--path/*.md` as additional candidates.
3. Apply `--keep-pattern` (if provided) to exclude any matching files from deletion.
4. Build a candidate list with:
   - Path
   - Size
   - Last modified time
</step1_discover_candidates>

<step2_present_plan>

1. If no candidates are found, print:
   - "Cleanup: No temporary analysis files to delete."
   - Exit successfully.
2. Otherwise, show a concise table:
   - `[ ] ./analyses/vad-threshold-analysis.md (2.3 KB, modified 2025-11-20)`
3. If `--dry-run` is set:
   - Print the table and exit without making changes.
4. If not `--dry-run`:
   - Use AskUserQuestion to confirm:
     - header: "Cleanup temporary analysis files"
     - question: "Delete the following temporary analysis files now? (Y/n)"
     - options:
       - "Y - Delete them" (default)
       - "n - Keep them"

</step2_present_plan>

<step3_apply_cleanup>

1. If user chooses "n - Keep them", print:
   - "Cleanup: User chose to keep all temporary analysis files."
   - Exit without modifying files.
2. If user accepts default or chooses "Y - Delete them":
   - For each candidate file:
     - Prefer actual deletion if the environment/tooling allows it.
     - If deletion is not available, overwrite the file with a short stub:
       - "This temporary analysis file was cleaned up by /cleanup-temporary-analyses and is no longer needed."
3. Print a short summary:
   - "Cleanup: Deleted N files, kept M files (pattern or user choice)."
</step3_apply_cleanup>

<step4_integration_guidance>
Commands that create temporary analysis files should:

- Not re-implement cleanup logic.
- At the end of their process, suggest or invoke `/cleanup-temporary-analyses`.
- Clearly mark any files that should **not** be cleaned up automatically.

Examples:

- `/run-prompt` can run all prompts, then call `/cleanup-temporary-analyses`.
- `/heal-docs` and `/heal-rules` can rely on this command to clean `./analyses/augment-docs-latest.md`.
</step4_integration_guidance>

<output>
- Clear summary of which files were considered, deleted, or kept.
- Safe behavior by default (confirmation required before destructive actions).
- Centralized cleanup behavior that other commands can rely on.
</output>

<success_criteria>

- Command identifies temporary analysis files under `./analyses/` and optional `--path`.
- Dry-run mode shows planned actions without making changes.
- User confirmation is required before any deletion/overwrite.
- Files are either deleted or overwritten with a stub when cleanup is confirmed.
- Other commands can simply chain `/cleanup-temporary-analyses` instead of duplicating logic.
- Behavior is aligned with `.augment/rules/90-documentation-cleanup.md`.

</success_criteria>

