---
description: Create comprehensive pull requests with GitHub issue linking, conventional commit titles, and automated quality validation for chipp-rs SDK development
argument-hint: [issue-number] [--draft] [--dry-run]
allowed-tools: [github-api, launch-process, view, codebase-retrieval]
---

<objective>
Create a comprehensive pull request for the currently checked-out branch with:
- Conventional commit title (feat/fix/docs/etc.)
- GitHub issue linking
- Comprehensive description with testing instructions
- Quality validation (cargo fmt, clippy, test, doc)
- crates.io publication readiness check
</objective>

<input>
Arguments:
- No arguments: Create PR for current branch → main
- `<issue-number>` (e.g., `3`): Link to specific GitHub issue
- `--draft`: Create as draft PR
- `--dry-run`: Preview PR without creating
- `--force`: Skip validation checks (use carefully)

Environment:

- BASE_BRANCH: Override default base branch (default: `main`)
</input>

<scope>
- Repository: `paulbreuler/chipp-rs`
- Always uses currently checked-out branch as PR head
- Prevents PR creation from `main` branch
- Validates git state and working directory cleanliness
- Integrates with GitHub API for PR creation
</scope>

<process>

<step_1_validate_environment>
Validate prerequisites:

1. **Git state validation**:
   - Check current branch is NOT `main`
   - Verify working directory is clean (no uncommitted changes)
   - Confirm branch has commits ahead of main
   - Check if branch exists on remote (offer to push if not)

2. **Repository check**:
   - Verify we're in paulbreuler/chipp-rs repository
   - Check remote is configured correctly

**If validation fails**: Stop and report specific issue with resolution steps
</step_1_validate_environment>

<step_2_gather_context>
Gather all context needed to generate PR title and description:

1. **Review all commits** (from HEAD back to main):
   ```bash
   git log main..HEAD --oneline
   git log main..HEAD --format="%s%n%b" # full messages with bodies
   ```

2. **Review changed files**:
   ```bash
   git diff main..HEAD --stat
   git diff main..HEAD --name-only
   ```

3. **Fetch linked GitHub issue** (if issue number provided or found in branch name):
   ```bash
   # Extract issue number from branch name (e.g., fix/issue-3-foo → 3)
   # Or use provided argument
   ```
   Then fetch via GitHub API:
   - GET `/repos/paulbreuler/chipp-rs/issues/{number}`
   - Extract: title, body, labels

4. **Detect breaking changes**:
   - Search commit messages for `BREAKING CHANGE:` or `!` suffix
   - Check for public API signature changes in diff
</step_2_gather_context>

<step_3_generate_pr_title>
**Derive title from GitHub issue and commit context**:

1. **If linked to a GitHub issue**: Use the issue title as the basis
   - Issue #3: "Fix `expect()` panic in `ChippClient::new()` - Return `Result` instead"
   - Convert to conventional commit: `fix!: ChippClient::new() returns Result instead of panicking`

2. **If no linked issue**: Summarize the commits into a single title
   - Review all commit messages
   - Identify the primary change/theme
   - Write a conventional commit title that captures the overall PR purpose

3. **Determine commit type** from context:
   - `fix` - bug fixes, error corrections
   - `feat` - new features, capabilities
   - `docs` - documentation only
   - `test` - test additions/changes only
   - `chore` - tooling, config, maintenance
   - `refactor` - code restructuring without behavior change

4. **Add `!` for breaking changes** if detected

**Format**: `<type>[!]: <description>` or `<type>(<scope>)[!]: <description>`

**Rules**:
- Type: lowercase
- Scope: optional, lowercase (client, config, streaming)
- Description: imperative mood, lowercase, no period
- Max 72 characters total

**Examples**:
- `fix!: ChippClient::new() returns Result instead of panicking`
- `feat(streaming): add SSE chunk parsing`
- `docs: update README with error handling examples`
</step_3_generate_pr_title>

<step_4_generate_pr_description>
Generate PR description by synthesizing gathered context:

1. **Summary**: Write based on:
   - The GitHub issue description (if linked)
   - The commit messages explaining what was done
   - The actual file changes

2. **Changes section**: Populate from `git diff --stat` and commit messages
   - Group into Added/Changed/Fixed/Removed

3. **Testing/Quality sections**: Pre-check boxes based on what was actually verified

**Template**:

```markdown
## Summary

[Synthesize from issue description + commits: what this PR does and why]

## Linked Issues

Closes #[issue-number]

## Changes

### Added
- [From commits and diff]

### Changed
- [From commits and diff]

### Fixed
- [From commits and diff]

## Testing

- [x/] Unit tests added/updated
- [x/] All tests passing (`cargo test`)
- [x/] Doc tests passing (`cargo test --doc`)
- [x/] Examples run successfully

## Quality Checks

- [x/] `cargo fmt --check` passes
- [x/] `cargo clippy --all-targets -- -D warnings` passes
- [x/] `cargo doc --no-deps` produces no warnings
- [x/] `cargo publish --dry-run` succeeds

## Breaking Changes

[If detected, describe changes and migration path with code examples]

## Checklist

- [x/] Code follows SDK patterns
- [x/] No `unwrap()` or `expect()` in library code
- [x/] Documentation updated
- [x/] CHANGELOG.md updated (if user-facing change)
```

</step_4_generate_pr_description>

<step_5_run_quality_checks>
Run validation checks (skip if `--force` flag present):

1. **Formatting**: `cargo fmt --check`
2. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Tests**: `cargo test`
4. **Documentation**: `cargo doc --no-deps --all-features`
5. **Publish readiness**: `cargo publish --dry-run --allow-dirty`

**If checks fail**: Report issues and suggest fixes or `--force` flag
</step_5_run_quality_checks>

<step_6_push_and_create_pr>
Push branch and create PR:

1. **Push branch** (if not already on remote):

   ```bash
   git push -u origin [branch-name]
   ```

2. **Create PR via GitHub API**:
   - POST to `/repos/paulbreuler/chipp-rs/pulls`
   - Set title, body, base (main), head (current branch)
   - Add labels based on change type

3. **Apply labels**:
   - Type-based: `fix`, `feature`, `documentation`, `breaking-change`
   - Auto-detect from commit type

**If --dry-run**: Print title, description without creating
</step_6_push_and_create_pr>

<step_7_output_summary>
Display summary:

```
✅ Pull Request Created

Title: fix!: ChippClient::new() returns Result instead of panicking
URL: https://github.com/paulbreuler/chipp-rs/pull/XX

Linked Issues:
  - Closes #3: Fix expect() panic in ChippClient::new()

Quality Checks:
  ✅ Formatting (cargo fmt)
  ✅ Linting (cargo clippy)
  ✅ Tests (38 passed)
  ✅ Documentation (no warnings)
  ✅ Publish ready (dry-run passed)

Next Steps:
  1. Review PR on GitHub
  2. Wait for CI to pass
  3. Request review if needed
  4. Merge when approved
```

</step_7_output_summary>

</process>

<success_criteria>

- PR created successfully on GitHub
- Title follows conventional commit format
- Description is comprehensive
- Issues properly linked with "Closes #N"
- Quality checks passed (or --force used)
- Branch pushed to remote
- User receives clear summary with PR URL
</success_criteria>
