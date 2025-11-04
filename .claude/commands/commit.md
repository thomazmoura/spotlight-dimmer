# Commit Command

**Description**: Create a commit with all pending changes

**Usage**: `/commit`

## What this command does:

1. Regenerates the JSON schema from the C# configuration classes
2. Runs `git status` and `git diff` to understand changes
3. Generates a descriptive commit message based on the changes
4. Creates a git commit with all pending changes
5. Pushes the commit to the main branch

## Process:

The agent will:
1. Run the schema generator: `pwsh SpotlightDimmer.Scripts/Generate-Schema.ps1`
2. Run `git status` and `git diff` to understand changes
3. Generate a descriptive commit message based on the changes and CHANGELOG.md
4. Execute a **single bash command** that does all of the following:
   ```bash
   git add . && git commit -m "message" && git pull --rebase origin main && git push origin main
   ```

## Important Notes:

- The entire git operation (add, commit, pull, push) must be presented as **ONE command line** using `&&`
- This allows the user to approve once with a single execution
- If any git step fails, subsequent steps won't execute (due to `&&` behavior)
- The commit message should be concise and descriptive

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

```bash
git add . && git commit -m "Add EVENT_OBJECT_LOCATIONCHANGE for real-time window tracking

Enhanced .NET PoC with fully event-driven window movement detection.

- Add EVENT_OBJECT_LOCATIONCHANGE hook for window movement events
- Filter OBJID_WINDOW to exclude cursor movement noise
- Track foreground window position changes in real-time
- Update documentation with dual event hook architecture" && git pull --rebase origin main && git push origin main
```