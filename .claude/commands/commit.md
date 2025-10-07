# Commit Command

**Description**: Run tests, build, and create a beta version commit with all pending changes

**Usage**: `/commit`

## What this command does:

1. Checks current versions in `package.json` and `Cargo.toml`
2. Increments to the next beta version (e.g., 0.1.10 → 0.1.11-beta.1, or 0.1.10-beta.1 → 0.1.10-beta.2)
3. Updates version in `package.json`, `Cargo.toml`, and `Cargo.lock`
4. **Runs pre-commit validation**: tests, clippy, and release build
5. **If validation fails**: Cancels the commit and prompts user to run `/check` first
6. Generates a commit message based on git diff and changelog
7. Creates a git commit with all pending changes
8. Creates a git tag with the new beta version
9. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current versions from `package.json` and `Cargo.toml`
2. Calculate new beta version:
   - If current is release (e.g., `0.1.10`): increment patch and add `-beta.1` (→ `0.1.11-beta.1`)
   - If current is beta (e.g., `0.1.10-beta.1`): increment beta number (→ `0.1.10-beta.2`)
3. Update version in `package.json`, `Cargo.toml`, and `Cargo.lock` (using `cargo update -p spotlight-dimmer --precise X.Y.Z-beta.N`)
4. **Run pre-commit validation in order** (matching `/check` exactly):
   - `cargo test --lib --verbose --target x86_64-pc-windows-gnu` - Run library tests with Windows target
   - `cargo test --doc --verbose` - Run doc tests
   - `cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code` - Check for code issues with Windows target
   - `cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config` - Build Windows binaries
5. **If any validation step fails**:
   - **STOP immediately** and cancel the commit
   - Revert version changes in `package.json`, `Cargo.toml`, and `Cargo.lock`
   - Display error output to user
   - Instruct user to run `/check` first to fix errors
   - **DO NOT proceed with commit** and **DO NOT attempt to fix errors**
6. Once validation passes, run `git status` and `git diff` to understand changes
7. Generate a descriptive commit message based on the changes
8. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.Z-beta.N && git push origin main && git push origin vX.Y.Z-beta.N
   ```

## Important Notes:

- **Beta versioning**: All commits use beta versions for safe iteration
- **Validation is mandatory**: Must pass tests, clippy, and build before committing
- **NO auto-fix**: If validation fails, commit is cancelled - run `/check` first to fix errors
- **Recommended workflow**: Run `/check` to fix any issues, then `/commit` to create the versioned commit
- The entire git operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.Z-beta.N` (e.g., `v0.1.11-beta.1`)
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

**Scenario 1**: Current version is `0.1.10` (release version)
- Update to `0.1.11-beta.1` in both files
- Run validation: tests → clippy → build
- If validation passes, create command like:
  ```bash
  git add . && git commit -m "Complete WinAPI refactor with 96% memory reduction

Migrated from Tauri to pure Rust with Windows API implementation for dramatic performance improvements.

- Remove Tauri dependency and web framework overhead
- Implement native Windows overlay system with layered windows
- Add direct Windows API calls for display and window management
- Reduce binary size by 95% and memory usage by 96%" && git pull --rebase origin main && git tag v0.1.11-beta.1 && git push origin main && git push origin v0.1.11-beta.1
  ```

**Scenario 2**: Current version is `0.1.11-beta.1` (beta version)
- Update to `0.1.11-beta.2` in both files
- Run validation: tests → clippy → build
- If validation passes, create tag `v0.1.11-beta.2`

**Scenario 3**: Validation fails - Commit cancelled
```
Updating version to 0.1.11-beta.1...
✓ Updated package.json
✓ Updated Cargo.toml
✓ Updated Cargo.lock

Running validation...
Running cargo test... ✗ 2 tests failed

❌ Validation failed! Commit cancelled.

Error details:
---- config::tests::test_config_load stdout ----
thread 'config::tests::test_config_load' panicked at 'assertion failed'

Reverting version changes...
✓ Reverted package.json to 0.1.10
✓ Reverted Cargo.toml to 0.1.10
✓ Reverted Cargo.lock to 0.1.10

Please run `/check` first to fix validation errors, then try `/commit` again.
```

**Recommended Workflow**:
1. Make your code changes
2. Run `/check` to validate and auto-fix any issues
3. Run `/commit` to create the versioned commit and push