# Publish Minor Command

**Description**: Create a minor release by incrementing the minor version (0.X.0)

**Usage**: `/publish-minor`

## What this command does:

1. Checks current versions in `package.json` and `Cargo.toml`
2. Increments the minor version and resets patch to 0 (e.g., 0.4.9 → 0.5.0, or 0.4.9-beta.2 → 0.5.0)
3. Updates version in `package.json`, `Cargo.toml`, and `Cargo.lock`
4. **Runs pre-commit validation**: tests, clippy, and release build
5. **If validation fails**: Cancels the release and prompts user to run `/check` first
6. Generates a commit message based on git diff and changelog
7. Creates a git commit with all pending changes
8. Creates a git tag with the new minor version
9. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current versions from `package.json` and `Cargo.toml`
2. Calculate new minor version:
   - If current is release (e.g., `0.4.9`): increment minor, reset patch (→ `0.5.0`)
   - If current is beta (e.g., `0.4.9-beta.2`): strip beta suffix, increment minor, reset patch (→ `0.5.0`)
   - Pattern: `X.Y.Z` → `X.(Y+1).0`
3. Update version in `package.json`, `Cargo.toml`, and `Cargo.lock` (using `cargo update -p spotlight-dimmer --precise X.Y.0`)
4. **Run pre-commit validation in order** (matching `/check` exactly):
   - `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine64 cargo test --lib --verbose --target x86_64-pc-windows-gnu` - Run library tests via Wine
   - `cargo test --doc --verbose` - Run doc tests
   - `cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code` - Check for code issues with Windows target
   - `cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config` - Build Windows binaries
5. **If any validation step fails**:
   - **STOP immediately** and cancel the release
   - Revert version changes in `package.json`, `Cargo.toml`, and `Cargo.lock`
   - Display error output to user
   - Instruct user to run `/check` first to fix errors
   - **DO NOT proceed with release** and **DO NOT attempt to fix errors**
6. Once validation passes, run `cargo fmt --all` to ensure code is properly formatted
7. Run `git status` and `git diff` to understand changes
8. Generate a descriptive commit message based on the changes
9. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.0 && git push origin main && git push origin vX.Y.0
   ```

## Important Notes:

- **Minor release versioning**: Creates stable releases with new features
- **Use for new features**: Minor releases add backward-compatible functionality
- **Validation is mandatory**: Must pass tests, clippy, and build before releasing
- **NO auto-fix**: If validation fails, release is cancelled - run `/check` first to fix errors
- **Recommended workflow**: Run `/check` to fix any issues, then `/publish-minor` to create the release
- The entire git operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.0` (e.g., `v0.5.0`)
- Cargo.lock is automatically updated to match the new version
- Patch number is always reset to 0 for minor releases

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
Add profile support for saving display configurations

Implemented profile system allowing users to save and switch between different dimming configurations for various workflows.

- Add profile CRUD operations to config module
- Implement profile switching via system tray menu
- Add profile management CLI commands
- Update documentation with profile usage examples
```

## Example:

**Scenario 1**: Current version is `0.4.10` (release version)
- Update to `0.5.0` in all files
- Run validation: tests → clippy → build
- If validation passes, create command like:
  ```bash
  git add . && git commit -m "Add keyboard shortcuts and hotkey support

Implemented global hotkey system for quick dimming control without opening the system tray.

- Add hotkey registration using Windows API
- Implement configurable keyboard shortcuts
- Add Ctrl+D for toggle dimming
- Add Ctrl+Shift+D for pause/resume
- Update settings UI with hotkey configuration
- Document keyboard shortcuts in README" && git pull --rebase origin main && git tag v0.5.0 && git push origin main && git push origin v0.5.0
  ```

**Scenario 2**: Current version is `0.4.10-beta.3` (beta version)
- Strip beta suffix and update to `0.5.0` in all files
- Run validation: tests → clippy → build
- If validation passes, create tag `v0.5.0`

**Scenario 3**: Validation fails - Release cancelled
```
Updating version to 0.5.0...
✓ Updated package.json
✓ Updated Cargo.toml
✓ Updated Cargo.lock

Running validation...
Running cargo test... ✗ 5 tests failed

❌ Validation failed! Release cancelled.

Error details:
---- hotkey::tests::test_register_hotkey stdout ----
thread 'hotkey::tests::test_register_hotkey' panicked at 'assertion failed'

Reverting version changes...
✓ Reverted package.json to 0.4.10
✓ Reverted Cargo.toml to 0.4.10
✓ Reverted Cargo.lock to 0.4.10

Please run `/check` first to fix validation errors, then try `/publish-minor` again.
```

## Release Workflow:

1. **Development**: Make changes and commit with `/commit` (creates beta versions)
2. **Feature Testing**: Test the new features thoroughly
3. **Quality Check**: Run `/check` to ensure all validation passes
4. **Release**: Run `/publish-minor` to create the stable minor release

## When to Use This Command:

- ✅ **New features**: Adding new backward-compatible functionality
- ✅ **Major improvements**: Significant enhancements to existing features
- ✅ **API additions**: Adding new public APIs or commands
- ✅ **Performance improvements**: Major performance optimizations
- ❌ **Bug fixes only**: Use `/publish-patch` instead
- ❌ **Breaking changes**: Plan for major version (1.0.0) release

## Semantic Versioning Reference:

Given a version number `MAJOR.MINOR.PATCH`:

- **PATCH** (`/publish-patch`): Bug fixes, small improvements (0.4.9 → 0.4.10)
- **MINOR** (`/publish-minor`): New features, backward-compatible (0.4.10 → 0.5.0)
- **MAJOR**: Breaking changes, incompatible API changes (0.5.0 → 1.0.0)
