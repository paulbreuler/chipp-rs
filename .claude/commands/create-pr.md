---
description: Create comprehensive pull requests with GitHub issue linking, conventional commit titles, and automated quality validation for chipp-rs SDK development
argument-hint: [issue-number] [--draft] [--dry-run]
model: sonnet
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

<step_2_analyze_changes>
Analyze changes to generate PR metadata:

1. **Determine conventional commit type from changed files**:
   - `src/lib.rs` API changes → `feat` or `fix`
   - `examples/` → `docs` or `feat`
   - `tests/` only → `test`
   - `*.md` only → `docs`
   - `Cargo.toml` deps only → `deps`
   - `.augment/` → `chore`
   - Branch name prefix overrides (fix/, feat/, docs/)

2. **Detect breaking changes**:
   - Search commit messages for `BREAKING CHANGE:` or `!` suffix
   - Check for public API signature changes
   - Flag if found

3. **Find linked issues**:
   - Parse branch name for issue numbers (e.g., `fix/issue-3-...`)
   - Search commit messages for `Fixes #`, `Closes #`, `Resolves #`
   - Use provided `<issue-number>` argument if present

4. **Extract commit messages** for summary generation
</step_2_analyze_changes>

<step_3_generate_pr_title>
**Use the first (oldest) commit message on the branch as the PR title**:

```bash
# Get the first commit message on this branch (the primary change)
git log main..HEAD --format=%s --reverse | head -1
```

The `--reverse` flag ensures we get the oldest commit first, which represents the primary change that started this branch.

**Verify it follows conventional commit format**:
- `<type>(<scope>): <description>`
- `<type>!: <description>` for breaking changes

**Examples**:
- `feat(client): add streaming chat support`
- `fix(client): return Result from ChippClient::new()`
- `docs: update README with streaming examples`
- `feat!: migrate to new authentication flow` (breaking change)

**Rules**:

- Type: lowercase (feat, fix, docs, test, chore, deps)
- Scope: optional, lowercase (client, config, streaming)
- Description: imperative mood, lowercase, no period
- Add `!` before `:` for breaking changes
- Max 72 characters total
</step_3_generate_pr_title>

<step_4_generate_pr_description>
Generate PR description using this template:

```markdown
## Summary

[One-paragraph summary of what this PR does and why]

## Linked Issues

Closes #[issue-number]

## Changes

### Added
- [New features]

### Changed
- [Modified behavior]

### Fixed
- [Bug fixes]

## Testing

- [ ] Unit tests added/updated
- [ ] All tests passing (`cargo test`)
- [ ] Doc tests passing (`cargo test --doc`)
- [ ] Examples run successfully

## Quality Checks

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo doc --no-deps` produces no warnings
- [ ] `cargo publish --dry-run` succeeds

## Breaking Changes

[If applicable, describe breaking changes and migration path]

## Checklist

- [ ] Code follows SDK patterns (`.augment/rules/50-rust-sdk-patterns.md`)
- [ ] No `unwrap()` or `expect()` in library code
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if user-facing change)
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
