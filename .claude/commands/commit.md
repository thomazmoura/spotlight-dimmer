# Commit Command

**Description**: Create a commit with all pending changes, increment patch version, create and push tag

**Usage**: `/commit`

## What this command does:

1. Checks current versions in `package.json` and `src/Cargo.toml`
2. Increments the patch version (third number) by 1 in both files
3. Generates a commit message based on git diff and changelog
4. Creates a git commit with all pending changes
5. Creates a git tag with the new version number
6. Pushes both the commit and the tag to the main branch

## Process:

The agent will:
1. Read current versions from `package.json` and `src/Cargo.toml`
2. Calculate new version (increment patch number)
3. Update version in both files
4. Run `git status` and `git diff` to understand changes
5. Generate a descriptive commit message based on the changes
6. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git tag vX.Y.Z && git push origin main && git push origin vX.Y.Z
   ```

## Important Notes:

- The entire operation (add, commit, tag, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive
- Tag format: `vX.Y.Z` (e.g., `v0.1.10`)

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

If current version is `0.1.9`, the command will:
- Update to `0.1.10` in both files
- Create command like:
  ```bash
  git add . && git commit -m "Complete WinAPI refactor with 96% memory reduction

Migrated from Tauri to pure Rust with Windows API implementation for dramatic performance improvements.

- Remove Tauri dependency and web framework overhead
- Implement native Windows overlay system with layered windows
- Add direct Windows API calls for display and window management
- Reduce binary size by 95% and memory usage by 96%" && git pull --rebase origin main && git tag v0.1.10 && git push origin main && git push origin v0.1.10
  ```