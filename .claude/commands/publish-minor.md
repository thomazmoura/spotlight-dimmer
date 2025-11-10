# Publish Minor Command

**Description**: Create a minor release by incrementing the minor version (0.X.0)

**Usage**: `/publish-minor`

## What this command does:

1. Checks current version in `Directory.Build.props`
2. Increments the minor version and resets patch to 0 (e.g., 0.8.1 → 0.9.0, or 0.8.1-beta → 0.9.0)
3. Updates version in `Directory.Build.props`
4. **Moves [Unreleased] to [X.Y.0] in CHANGELOG.md** using `Move-UnreleasedToVersion.ps1`
5. **Regenerates the JSON schema** from C# configuration classes
6. **Runs pre-commit validation**: build and tests
7. **If validation fails**: Cancels the release and prompts user to fix issues first
8. Generates a commit message based on git diff and changelog
9. Creates a git commit with all pending changes
10. Creates a git tag with the new minor version
11. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current version from `Directory.Build.props` (the `<Version>` property)
2. Calculate new minor version:
   - If current is release (e.g., `0.8.1`): increment minor, reset patch (→ `0.9.0`)
   - If current is beta (e.g., `0.8.1-beta`): strip beta suffix, increment minor, reset patch (→ `0.9.0`)
   - Pattern: `X.Y.Z` → `X.(Y+1).0`
3. Update all version properties in `Directory.Build.props`:
   - `<Version>X.(Y+1).0</Version>`
   - `<AssemblyVersion>X.(Y+1).0</AssemblyVersion>`
   - `<FileVersion>X.(Y+1).0</FileVersion>`
   - `<InformationalVersion>X.(Y+1).0</InformationalVersion>`
4. **Move [Unreleased] to versioned section**: Run `pwsh SpotlightDimmer.Scripts/Move-UnreleasedToVersion.ps1 -Version X.(Y+1).0`
   - Extracts all content from [Unreleased] section
   - Creates new [X.Y.0] - YYYY-MM-DD section with that content
   - Resets [Unreleased] to empty
5. **Regenerate JSON schema**: Run `pwsh SpotlightDimmer.Scripts/Generate-Schema.ps1`
6. **Run pre-commit validation in order**:
   - `dotnet build -c Release` - Build release binaries
   - `dotnet test` - Run all tests (if any test projects exist)
7. **If any validation step fails**:
   - **STOP immediately** and cancel the release
   - Revert version changes in `Directory.Build.props`
   - Revert CHANGELOG.md changes (move [X.Y.0] back to [Unreleased])
   - Display error output to user
   - Instruct user to fix errors first
   - **DO NOT proceed with release** and **DO NOT attempt to fix errors**
8. Once validation passes, run `dotnet format` to ensure code is properly formatted
9. Run `git status` and `git diff` to understand changes
10. Generate a descriptive commit message based on the changes and CHANGELOG.md
11. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.0 && git push origin main && git push origin vX.Y.0
   ```

## Important Notes:

- **Minor release versioning**: Creates stable releases with new features
- **Use for new features**: Minor releases add backward-compatible functionality
- **Validation is mandatory**: Must pass build and tests before releasing
- **NO auto-fix**: If validation fails, release is cancelled
- The entire git operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.0` (e.g., `v0.9.0`)
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
- Add profile management commands
- Update documentation with profile usage examples
```

## Example:

**Scenario 1**: Current version is `0.8.1` (release version)
- Update to `0.9.0` in Directory.Build.props
- Run validation: build → tests
- If validation passes, create command like:
  ```bash
  git add . && git commit -m "Add keyboard shortcuts and hotkey support

Implemented global hotkey system for quick dimming control.

- Add hotkey registration using Windows API
- Implement configurable keyboard shortcuts
- Add Ctrl+D for toggle dimming
- Add Ctrl+Shift+D for pause/resume
- Update configuration with hotkey settings
- Document keyboard shortcuts in README" && git pull --rebase origin main && git tag v0.9.0 && git push origin main && git push origin v0.9.0
  ```

**Scenario 2**: Current version is `0.8.1-beta.2` (beta version)
- Strip beta suffix and update to `0.9.0` in Directory.Build.props
- Run validation: build → tests
- If validation passes, create tag `v0.9.0`

**Scenario 3**: Validation fails - Release cancelled
```
Updating version to 0.9.0...
✓ Updated Directory.Build.props

Running validation...
Running dotnet build... ✗ Build failed

❌ Validation failed! Release cancelled.

Error details:
WindowsBindings/FocusTracker.cs(115,42): error CS1061: 'Hotkey' does not contain a definition for 'Register'

Reverting version changes...
✓ Reverted Directory.Build.props to 0.8.1

Please fix validation errors first, then try /publish-minor again.
```

## Release Workflow:

1. **Development**: Make changes and commit (creates beta versions with /commit)
2. **Feature Testing**: Test the new features thoroughly
3. **Quality Check**: Run builds and tests to ensure everything passes
4. **Release**: Run `/publish-minor` to create the stable minor release

## When to Use This Command:

- ✅ **New features**: Adding new backward-compatible functionality
- ✅ **Major improvements**: Significant enhancements to existing features
- ✅ **API additions**: Adding new public APIs or functionality
- ✅ **Performance improvements**: Major performance optimizations
- ❌ **Bug fixes only**: Use `/publish-patch` instead
- ❌ **Breaking changes**: Plan carefully and communicate to users

## Semantic Versioning Reference:

Given a version number `MAJOR.MINOR.PATCH`:

- **PATCH** (`/publish-patch`): Bug fixes, small improvements (0.8.0 → 0.8.1)
- **MINOR** (`/publish-minor`): New features, backward-compatible (0.8.1 → 0.9.0)
- **MAJOR**: Breaking changes, incompatible API changes (0.9.0 → 1.0.0)
