# Publish Patch Command

**Description**: Create a patch release by incrementing the patch version (0.0.X)

**Usage**: `/publish-patch`

## What this command does:

1. Checks current versions in `package.json` and `Cargo.toml`
2. Increments the patch version (e.g., 0.4.9 → 0.4.10, or 0.4.9-beta.2 → 0.4.10)
3. Updates version in `package.json`, `Cargo.toml`, and `Cargo.lock`
4. **Runs pre-commit validation**: tests, clippy, and release build
5. **If validation fails**: Cancels the release and prompts user to run `/check` first
6. Generates a commit message based on git diff and changelog
7. Creates a git commit with all pending changes
8. Creates a git tag with the new patch version
9. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current versions from `package.json` and `Cargo.toml`
2. Calculate new patch version:
   - If current is release (e.g., `0.4.9`): increment patch (→ `0.4.10`)
   - If current is beta (e.g., `0.4.9-beta.2`): strip beta suffix and increment patch (→ `0.4.10`)
   - Pattern: `X.Y.Z` → `X.Y.(Z+1)`
3. Update version in `package.json`, `Cargo.toml`, and `Cargo.lock` (using `cargo update -p spotlight-dimmer --precise X.Y.Z`)
4. **Run pre-commit validation in order** (matching `/check` exactly):
   - `cargo test --lib --verbose --target x86_64-pc-windows-gnu` - Run library tests with Windows target
   - `cargo test --doc --verbose` - Run doc tests
   - `cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code` - Check for code issues with Windows target
   - `cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config` - Build Windows binaries
5. **If any validation step fails**:
   - **STOP immediately** and cancel the release
   - Revert version changes in `package.json`, `Cargo.toml`, and `Cargo.lock`
   - Display error output to user
   - Instruct user to run `/check` first to fix errors
   - **DO NOT proceed with release** and **DO NOT attempt to fix errors**
6. Once validation passes, run `git status` and `git diff` to understand changes
7. Generate a descriptive commit message based on the changes
8. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.Z && git push origin main && git push origin vX.Y.Z
   ```

## Important Notes:

- **Patch release versioning**: Creates stable releases without beta suffix
- **Use for bug fixes**: Patch releases are for bug fixes and small changes
- **Validation is mandatory**: Must pass tests, clippy, and build before releasing
- **NO auto-fix**: If validation fails, release is cancelled - run `/check` first to fix errors
- **Recommended workflow**: Run `/check` to fix any issues, then `/publish-patch` to create the release
- The entire git operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.Z` (e.g., `v0.4.10`)
- Cargo.lock is automatically updated to match the new version

## Commit Message Format (CRITICAL):

**DO NOT** include these in commit messages:
- ❌ "🤖 Generated with [Claude Code](https://claude.com/claude-code)"
- ❌ "Co-Authored-By: Claude <noreply@anthropic.com>"

This repository is built with Claude Code - these attributions are redundant.

**DO** use clear, descriptive commit messages:
- Subject line (50 chars max)
- Brief explanation of what changed and why
- List specific changes if multiple

### Example Format:
```
Fix window dragging crash and ghost windows

Implemented intelligent mouse button detection to prevent system instability during window drag operations.

- Add GetAsyncKeyState mouse button detection
- Hide overlays when mouse button pressed
- Restore overlays when mouse released
- Document known limitation in README files
```

## Example:

**Scenario 1**: Current version is `0.4.9` (release version)
- Update to `0.4.10` in all files
- Run validation: tests → clippy → build
- If validation passes, create command like:
  ```bash
  git add . && git commit -m "Fix overlay flickering on multi-monitor setups

Resolved issue where overlays would flicker when moving windows between displays.

- Improve display change detection logic
- Add debouncing for rapid display events
- Optimize overlay recreation performance" && git pull --rebase origin main && git tag v0.4.10 && git push origin main && git push origin v0.4.10
  ```

**Scenario 2**: Current version is `0.4.9-beta.2` (beta version)
- Strip beta suffix and update to `0.4.10` in all files
- Run validation: tests → clippy → build
- If validation passes, create tag `v0.4.10`

**Scenario 3**: Validation fails - Release cancelled
```
Updating version to 0.4.10...
✓ Updated package.json
✓ Updated Cargo.toml
✓ Updated Cargo.lock

Running validation...
Running cargo test... ✗ 2 tests failed

❌ Validation failed! Release cancelled.

Error details:
---- config::tests::test_config_load stdout ----
thread 'config::tests::test_config_load' panicked at 'assertion failed'

Reverting version changes...
✓ Reverted package.json to 0.4.9
✓ Reverted Cargo.toml to 0.4.9
✓ Reverted Cargo.lock to 0.4.9

Please run `/check` first to fix validation errors, then try `/publish-patch` again.
```

## Release Workflow:

1. **Development**: Make changes and commit with `/commit` (creates beta versions)
2. **Testing**: Test the beta versions thoroughly
3. **Quality Check**: Run `/check` to ensure all validation passes
4. **Release**: Run `/publish-patch` to create the stable patch release

## When to Use This Command:

- ✅ **Bug fixes**: Fixing bugs in the current release
- ✅ **Small improvements**: Minor enhancements that don't add new features
- ✅ **Documentation updates**: Significant documentation improvements
- ✅ **Dependency updates**: Updating dependencies without breaking changes
- ❌ **New features**: Use `/publish-minor` instead
- ❌ **Breaking changes**: Use `/publish-minor` and plan for major release
