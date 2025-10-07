# Commit Command

**Description**: Run tests, build, and create a beta version commit with all pending changes

**Usage**: `/commit`

## What this command does:

1. Checks current versions in `package.json` and `src/Cargo.toml`
2. Increments to the next beta version (e.g., 0.1.10 → 0.1.11-beta.1, or 0.1.10-beta.1 → 0.1.10-beta.2)
3. **Runs pre-commit validation**: tests, clippy, and release build
4. **If validation fails**: Automatically fixes errors and retries the entire process
5. Generates a commit message based on git diff and changelog
6. Creates a git commit with all pending changes
7. Creates a git tag with the new beta version
8. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current versions from `package.json` and `src/Cargo.toml`
2. Calculate new beta version:
   - If current is release (e.g., `0.1.10`): increment patch and add `-beta.1` (→ `0.1.11-beta.1`)
   - If current is beta (e.g., `0.1.10-beta.1`): increment beta number (→ `0.1.10-beta.2`)
3. Update version in both files
4. **Run pre-commit validation in order**:
   - `cargo test` - Run all tests
   - `cargo clippy --all-targets --all-features -- -D warnings` - Check for code issues
   - `cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config` - Build release binaries
5. **If any validation step fails**:
   - Analyze the error output
   - Fix the errors (code fixes, dependency updates, etc.)
   - Restart from step 4 (re-run all validation)
   - Continue until all validation passes
6. Once validation passes, run `git status` and `git diff` to understand changes
7. Generate a descriptive commit message based on the changes
8. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.Z-beta.N && git push origin main && git push origin vX.Y.Z-beta.N
   ```

## Important Notes:

- **Beta versioning**: All commits use beta versions for safe iteration
- **Validation is mandatory**: Must pass tests, clippy, and build before committing
- **Auto-fix on failure**: Agent automatically fixes validation errors and retries
- The entire git operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.Z-beta.N` (e.g., `v0.1.11-beta.1`)

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

**Scenario 3**: Validation fails
- Agent runs `cargo test` → fails with 2 test errors
- Agent analyzes errors, fixes the code
- Agent re-runs `cargo test` → passes
- Agent runs `cargo clippy` → fails with 1 warning
- Agent fixes the clippy warning
- Agent re-runs all validation from start → all pass
- Agent proceeds with commit