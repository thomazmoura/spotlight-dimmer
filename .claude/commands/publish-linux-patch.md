# Publish Linux Patch Command

**Description**: Create a **Linux** patch release by incrementing the patch version (0.0.X) of the Rust workspace. For the Windows version, use `/publish-patch` instead.

**Usage**: `/publish-linux-patch`

## What this command does:

1. Checks current version in `SpotlightDimmer.LinuxDaemon/Cargo.toml` (`[workspace.package] version`)
2. Increments the patch version (e.g., 0.1.0 → 0.1.1)
3. Updates the version in `Cargo.toml`, `Cargo.lock` and the KWin script metadata
4. **Moves [Unreleased] to [X.Y.Z] in CHANGELOG.linux.md** using `Move-UnreleasedToVersion.ps1`
5. **Runs pre-commit validation**: tests and both build profiles
6. **If validation fails**: Cancels the release and prompts user to fix issues first
7. Generates a commit message based on git diff and changelog
8. Creates a git commit with all pending changes
9. Creates a git tag `vX.Y.Z-linux` (this triggers the `release-linux.yml` workflow, which builds and publishes the .deb packages)
10. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current version from `SpotlightDimmer.LinuxDaemon/Cargo.toml` (the `version` property under `[workspace.package]`)
2. Calculate new patch version: `X.Y.Z` → `X.Y.(Z+1)`
3. Update the version in all three places:
   - `SpotlightDimmer.LinuxDaemon/Cargo.toml`: `[workspace.package] version = "X.Y.(Z+1)"`
   - Refresh `SpotlightDimmer.LinuxDaemon/Cargo.lock`: run `cargo check` (or `cargo update -w`) inside `SpotlightDimmer.LinuxDaemon/`
   - `SpotlightDimmer.KwinScript/metadata.json`: `"Version": "X.Y.(Z+1)"` (keep the KWin script version in sync with the workspace)
4. **Move [Unreleased] to versioned section**: Run `pwsh SpotlightDimmer.Scripts/Move-UnreleasedToVersion.ps1 -Version X.Y.(Z+1) -ChangelogPath CHANGELOG.linux.md`
   - Extracts all content from [Unreleased] section
   - Creates new [X.Y.Z] - YYYY-MM-DD section with that content
   - Resets [Unreleased] to empty
5. **Run pre-commit validation in order** (from `SpotlightDimmer.LinuxDaemon/`):
   - `cargo test -p spotlight-dimmer-core` - Run the core test suite
   - `cargo build --release` - Build the layer-shell daemon (KDE profile)
   - `cargo build --release --no-default-features` - Build the headless daemon (GNOME profile)

   **KDE-profile fallback**: if `pkg-config --exists gtk4-layer-shell-0` fails, the machine has no gtk4-layer-shell (e.g. Ubuntu 24.04 GNOME boxes — the lib only exists in Ubuntu 25.10+ repositories). Skip the KDE-profile build (`cargo build --release`) and rely on CI instead: the `Test Linux Build` workflow builds that profile on every push touching Linux code, and the `Release Linux` workflow rebuilds it before publishing. The core tests and the headless build must still pass locally.
6. **If any validation step fails**:
   - **STOP immediately** and cancel the release
   - Revert version changes in `Cargo.toml`, `Cargo.lock` and `SpotlightDimmer.KwinScript/metadata.json`
   - Revert CHANGELOG.linux.md changes (move [X.Y.Z] back to [Unreleased])
   - Display error output to user
   - Instruct user to fix errors first
   - **DO NOT proceed with release** and **DO NOT attempt to fix errors**
7. Run `git status` and `git diff` to understand changes
8. Generate a descriptive commit message based on the changes and CHANGELOG.linux.md
9. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.Z-linux && git push origin main && git push origin vX.Y.Z-linux
   ```

## Important Notes:

- **Independent versioning**: The Linux version (Cargo workspace) is versioned separately from the Windows version (`Directory.Build.props`) — do NOT touch `Directory.Build.props` or `CHANGELOG.md`
- **Use for bug fixes**: Patch releases are for bug fixes and small changes
- **Validation is mandatory**: Must pass tests and both builds before releasing
- **NO auto-fix**: If validation fails, release is cancelled
- The entire git operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.Z-linux` (e.g., `v0.1.1-linux`) — the `-linux` suffix triggers the Linux release workflow; Windows releases use `vX.Y.Z-windows` tags

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
Fix stale overlay after monitor hotplug on KDE

Restored cached monitor layout handling so overlays recover after the daemon restarts.

- Cache monitor layout in the session runtime directory
- Restore layout on daemon startup
- Update Linux changelog
```

## Example:

**Scenario 1**: Current version is `0.1.0`
- Update to `0.1.1` in `Cargo.toml` (workspace), refresh `Cargo.lock`, bump KWin `metadata.json`
- Run validation: tests → KDE build → GNOME build
- If validation passes, create command like:
  ```bash
  git add . && git commit -m "Fix overlay recovery after KWin restart

Overlays now reappear immediately after KWin restarts instead of waiting for a monitor event.

- Re-register with the daemon on script reload
- Update Linux changelog" && git pull --rebase origin main && git tag v0.1.1-linux && git push origin main && git push origin v0.1.1-linux
  ```

**Scenario 2**: Validation fails - Release cancelled
```
Updating version to 0.1.1...
✓ Updated Cargo.toml, Cargo.lock and KWin metadata.json

Running validation...
Running cargo test... ✗ Tests failed

❌ Validation failed! Release cancelled.

Reverting version changes...
✓ Reverted Cargo.toml, Cargo.lock and metadata.json to 0.1.0

Please fix validation errors first, then try /publish-linux-patch again.
```

## When to Use This Command:

- ✅ **Bug fixes**: Fixing bugs in the Linux daemon, GNOME extension or KWin script
- ✅ **Small improvements**: Minor enhancements that don't add new features
- ✅ **Documentation updates**: Significant Linux documentation improvements
- ❌ **New features**: Use `/publish-linux-minor` instead
- ❌ **Windows changes**: Use `/publish-patch` instead
