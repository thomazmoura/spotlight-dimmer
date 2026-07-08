# Publish Linux Minor Command

**Description**: Create a **Linux** minor release by incrementing the minor version (0.X.0) of the Rust workspace. For the Windows version, use `/publish-minor` instead.

**Usage**: `/publish-linux-minor`

## What this command does:

1. Checks current version in `SpotlightDimmer.LinuxDaemon/Cargo.toml` (`[workspace.package] version`)
2. Increments the minor version and resets patch to 0 (e.g., 0.1.1 → 0.2.0)
3. Updates the version in `Cargo.toml`, `Cargo.lock` and the KWin script metadata
4. **Moves [Unreleased] to [X.Y.0] in CHANGELOG.linux.md** using `Move-UnreleasedToVersion.ps1`
5. **Runs pre-commit validation**: tests and both build profiles
6. **If validation fails**: Cancels the release and prompts user to fix issues first
7. Generates a commit message based on git diff and changelog
8. Creates a git commit with all pending changes
9. Creates a git tag `vX.Y.0-linux` (this triggers the `release-linux.yml` workflow, which builds and publishes the .deb packages)
10. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current version from `SpotlightDimmer.LinuxDaemon/Cargo.toml` (the `version` property under `[workspace.package]`)
2. Calculate new minor version: `X.Y.Z` → `X.(Y+1).0`
3. Update the version in all three places:
   - `SpotlightDimmer.LinuxDaemon/Cargo.toml`: `[workspace.package] version = "X.(Y+1).0"`
   - Refresh `SpotlightDimmer.LinuxDaemon/Cargo.lock`: run `cargo check` (or `cargo update -w`) inside `SpotlightDimmer.LinuxDaemon/`
   - `SpotlightDimmer.KwinScript/metadata.json`: `"Version": "X.(Y+1).0"` (keep the KWin script version in sync with the workspace)
4. **Move [Unreleased] to versioned section**: Run `pwsh SpotlightDimmer.Scripts/Move-UnreleasedToVersion.ps1 -Version X.(Y+1).0 -ChangelogPath CHANGELOG.linux.md`
   - Extracts all content from [Unreleased] section
   - Creates new [X.Y.0] - YYYY-MM-DD section with that content
   - Resets [Unreleased] to empty
5. **Run pre-commit validation in order** (from `SpotlightDimmer.LinuxDaemon/`):
   - `cargo test -p spotlight-dimmer-core` - Run the core test suite
   - `cargo build --release` - Build the layer-shell daemon (KDE profile)
   - `cargo build --release --no-default-features` - Build the headless daemon (GNOME profile)
6. **If any validation step fails**:
   - **STOP immediately** and cancel the release
   - Revert version changes in `Cargo.toml`, `Cargo.lock` and `SpotlightDimmer.KwinScript/metadata.json`
   - Revert CHANGELOG.linux.md changes (move [X.Y.0] back to [Unreleased])
   - Display error output to user
   - Instruct user to fix errors first
   - **DO NOT proceed with release** and **DO NOT attempt to fix errors**
7. Run `git status` and `git diff` to understand changes
8. Generate a descriptive commit message based on the changes and CHANGELOG.linux.md
9. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.0-linux && git push origin main && git push origin vX.Y.0-linux
   ```

## Important Notes:

- **Independent versioning**: The Linux version (Cargo workspace) is versioned separately from the Windows version (`Directory.Build.props`) — do NOT touch `Directory.Build.props` or `CHANGELOG.md`
- **Use for new features**: Minor releases add backward-compatible functionality
- **Validation is mandatory**: Must pass tests and both builds before releasing
- **NO auto-fix**: If validation fails, release is cancelled
- The entire git operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.0-linux` (e.g., `v0.2.0-linux`) — the `-linux` suffix triggers the Linux release workflow; Windows releases use `vX.Y.Z-windows` tags
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
Add sway/wlroots compositor support

Implemented a wlroots adapter so the daemon can dim inactive outputs on sway.

- Add wlr-foreign-toplevel focus tracking
- Document sway setup in LINUX_DAEMON.md
- Update Linux changelog
```

## Example:

**Scenario 1**: Current version is `0.1.1`
- Update to `0.2.0` in `Cargo.toml` (workspace), refresh `Cargo.lock`, bump KWin `metadata.json`
- Run validation: tests → KDE build → GNOME build
- If validation passes, create command like:
  ```bash
  git add . && git commit -m "Add per-application dimming rules

Users can now exclude specific applications from dimming via config.

- Add ExcludedApps config section
- Match windows by class in the daemon
- Update CONFIGURATION.md and Linux changelog" && git pull --rebase origin main && git tag v0.2.0-linux && git push origin main && git push origin v0.2.0-linux
  ```

**Scenario 2**: Validation fails - Release cancelled
```
Updating version to 0.2.0...
✓ Updated Cargo.toml, Cargo.lock and KWin metadata.json

Running validation...
Running cargo build --release... ✗ Build failed

❌ Validation failed! Release cancelled.

Reverting version changes...
✓ Reverted Cargo.toml, Cargo.lock and metadata.json to 0.1.1

Please fix validation errors first, then try /publish-linux-minor again.
```

## When to Use This Command:

- ✅ **New features**: Adding new backward-compatible Linux functionality
- ✅ **Major improvements**: Significant enhancements to the daemon, extension or KWin script
- ✅ **New compositor support**: Adding support for additional desktops/compositors
- ❌ **Bug fixes only**: Use `/publish-linux-patch` instead
- ❌ **Windows changes**: Use `/publish-minor` instead

## Semantic Versioning Reference:

Given a version number `MAJOR.MINOR.PATCH`:

- **PATCH** (`/publish-linux-patch`): Bug fixes, small improvements (0.1.0 → 0.1.1)
- **MINOR** (`/publish-linux-minor`): New features, backward-compatible (0.1.1 → 0.2.0)
- **MAJOR**: Breaking changes, incompatible API changes (0.2.0 → 1.0.0)
