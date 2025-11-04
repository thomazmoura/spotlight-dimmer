# Publish Patch Command

**Description**: Create a patch release by incrementing the patch version (0.0.X)

**Usage**: `/publish-patch`

## What this command does:

1. Checks current version in `Directory.Build.props`
2. Increments the patch version (e.g., 0.8.0 → 0.8.1, or 0.8.0-beta → 0.8.1)
3. Updates version in `Directory.Build.props`
4. **Regenerates the JSON schema** from C# configuration classes
5. **Runs pre-commit validation**: build and tests
6. **If validation fails**: Cancels the release and prompts user to fix issues first
7. Generates a commit message based on git diff and changelog
8. Creates a git commit with all pending changes
9. Creates a git tag with the new patch version
10. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current version from `Directory.Build.props` (the `<Version>` property)
2. Calculate new patch version:
   - If current is release (e.g., `0.8.0`): increment patch (→ `0.8.1`)
   - If current is beta (e.g., `0.8.0-beta`): strip beta suffix and increment patch (→ `0.8.1`)
   - Pattern: `X.Y.Z` → `X.Y.(Z+1)`
3. Update all version properties in `Directory.Build.props`:
   - `<Version>X.Y.(Z+1)</Version>`
   - `<AssemblyVersion>X.Y.(Z+1)</AssemblyVersion>`
   - `<FileVersion>X.Y.(Z+1)</FileVersion>`
   - `<InformationalVersion>X.Y.(Z+1)</InformationalVersion>`
4. **Regenerate JSON schema**: Run `pwsh SpotlightDimmer.Scripts/Generate-Schema.ps1`
5. **Run pre-commit validation in order**:
   - `dotnet build -c Release` - Build release binaries
   - `dotnet test` - Run all tests (if any test projects exist)
6. **If any validation step fails**:
   - **STOP immediately** and cancel the release
   - Revert version changes in `Directory.Build.props`
   - Display error output to user
   - Instruct user to fix errors first
   - **DO NOT proceed with release** and **DO NOT attempt to fix errors**
7. Once validation passes, run `dotnet format` to ensure code is properly formatted
8. Run `git status` and `git diff` to understand changes
9. Generate a descriptive commit message based on the changes and CHANGELOG.md
10. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.Z && git push origin main && git push origin vX.Y.Z
   ```

## Important Notes:

- **Patch release versioning**: Creates stable releases without beta suffix
- **Use for bug fixes**: Patch releases are for bug fixes and small changes
- **Validation is mandatory**: Must pass build and tests before releasing
- **NO auto-fix**: If validation fails, release is cancelled
- The entire git operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.Z` (e.g., `v0.8.1`)

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
Fix window dragging memory leak

Implemented proper disposal of DeferWindowPos handles to prevent memory growth during window operations.

- Fix DeferWindowPos handle cleanup
- Add GDI object monitoring in verbose mode
- Document handle leak prevention pattern
```

## Example:

**Scenario 1**: Current version is `0.8.0` (release version)
- Update to `0.8.1` in Directory.Build.props
- Run validation: build → tests
- If validation passes, create command like:
  ```bash
  git add . && git commit -m "Fix overlay flickering on multi-monitor setups

Resolved issue where overlays would flicker when moving windows between displays.

- Improve display change detection logic
- Add debouncing for rapid display events
- Optimize overlay recreation performance" && git pull --rebase origin main && git tag v0.8.1 && git push origin main && git push origin v0.8.1
  ```

**Scenario 2**: Current version is `0.8.0-beta` (beta version)
- Strip beta suffix and update to `0.8.1` in Directory.Build.props
- Run validation: build → tests
- If validation passes, create tag `v0.8.1`

**Scenario 3**: Validation fails - Release cancelled
```
Updating version to 0.8.1...
✓ Updated Directory.Build.props

Running validation...
Running dotnet build... ✗ Build failed

❌ Validation failed! Release cancelled.

Error details:
Program.cs(42,5): error CS0103: The name 'InvalidMethod' does not exist in the current context

Reverting version changes...
✓ Reverted Directory.Build.props to 0.8.0

Please fix validation errors first, then try /publish-patch again.
```

## Release Workflow:

1. **Development**: Make changes and commit (creates beta versions with /commit)
2. **Testing**: Test the beta versions thoroughly
3. **Quality Check**: Run builds and tests to ensure everything passes
4. **Release**: Run `/publish-patch` to create the stable patch release

## When to Use This Command:

- ✅ **Bug fixes**: Fixing bugs in the current release
- ✅ **Small improvements**: Minor enhancements that don't add new features
- ✅ **Documentation updates**: Significant documentation improvements
- ✅ **Performance fixes**: Fixing performance regressions
- ❌ **New features**: Use `/publish-minor` instead
- ❌ **Breaking changes**: Use `/publish-minor` and plan carefully
