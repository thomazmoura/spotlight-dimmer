# GitHub Copilot Prompt Files

This directory contains reusable prompt files for common development workflows in the Spotlight Dimmer project.

## What are Prompt Files?

Prompt files are Markdown files with a `.prompt.md` extension that define reusable prompts for specific development tasks. They can be run directly in GitHub Copilot Chat and include task-specific context and guidelines.

## Available Prompts

### `/check-ci` - Check CI Build Status
**File**: `check-ci.prompt.md`

Fetches the latest CI build results from GitHub Actions, analyzes any errors, and automatically fixes them.

**Usage in VS Code**:
1. Open Chat view
2. Type `/check-ci` in the chat input
3. Press Enter

**Requirements**: GitHub CLI (`gh`) must be installed and authenticated

---

### `/check` - Run Validation Checks
**File**: `check.prompt.md`

Runs the complete validation pipeline (tests, clippy, build) and automatically fixes any errors found.

**Usage in VS Code**:
1. Open Chat view
2. Type `/check` in the chat input
3. Press Enter

**What it does**:
- Runs library tests
- Runs doc tests
- Runs clippy checks
- Builds release binaries
- Auto-fixes errors and re-validates

---

### `/commit` - Create Beta Version Commit
**File**: `commit.prompt.md`

Creates a versioned beta commit with full validation and proper git workflow.

**Usage in VS Code**:
1. Open Chat view
2. Type `/commit` in the chat input
3. Press Enter

**What it does**:
- Calculates next beta version
- Updates version files
- Runs pre-commit validation
- Generates commit message
- Creates commit and tag
- Pushes to remote

---

### `/publish-minor` - Publish Minor Release
**File**: `publish-minor.prompt.md`

Creates a minor release (0.X.0) for new features.

**Usage in VS Code**:
1. Open Chat view
2. Type `/publish-minor` in the chat input
3. Press Enter

**When to use**: New backward-compatible features

---

### `/publish-patch` - Publish Patch Release
**File**: `publish-patch.prompt.md`

Creates a patch release (0.0.X) for bug fixes.

**Usage in VS Code**:
1. Open Chat view
2. Type `/publish-patch` in the chat input
3. Press Enter

**When to use**: Bug fixes, optimizations, small changes

---

## Enabling Prompt Files

Before using prompt files, you need to enable them in VS Code:

### Option 1: Via Settings UI
1. Open Settings (Ctrl+,)
2. Search for "chat.promptFiles"
3. Enable the "Chat: Prompt Files" checkbox

### Option 2: Via settings.json
1. Open Command Palette (Ctrl+Shift+P)
2. Type "Open User Settings (JSON)"
3. Add: `"chat.promptFiles": true`

### Option 3: Via Workspace Settings
1. Open Command Palette (Ctrl+Shift+P)
2. Type "Open Workspace Settings (JSON)"
3. Add: `"chat.promptFiles": true`

## How Prompt Files Work

### File Structure

Each prompt file has:

1. **YAML Frontmatter** (optional):
   ```yaml
   ---
   description: Short description of the prompt
   mode: agent  # or 'ask', 'edit'
   model: gpt-4  # optional, uses selected model if not specified
   tools: [terminal, workspace]  # optional, available tools
   ---
   ```

2. **Markdown Body**:
   - Instructions in Markdown format
   - Code blocks with commands
   - References to other files
   - Variables like `${selection}`, `${file}`, etc.

### Using Prompt Files

**Method 1: Type in Chat**
```
/prompt-name
```

**Method 2: Command Palette**
1. Press Ctrl+Shift+P
2. Run "Chat: Run Prompt"
3. Select prompt from list

**Method 3: Editor Play Button**
1. Open `.prompt.md` file
2. Click play button in title bar
3. Choose to run in current or new chat session

## Tips

1. **Always enable prompt files first** via settings
2. **Use `/` prefix** to invoke prompts in chat
3. **Combine with custom instructions** for best results
4. **Test prompts** using the editor play button
5. **Pass additional context** after the prompt name: `/commit Add dark mode feature`

## Differences from Claude Code Commands

| Claude Code | GitHub Copilot | Notes |
|-------------|----------------|-------|
| `/check` | `/check` | Same workflow, Copilot asks for confirmation |
| `/check-ci` | `/check-ci` | Requires GitHub CLI |
| `/commit` | `/commit` | Copilot asks for approval before git operations |
| `/publish-patch` | `/publish-patch` | Copilot validates each step |
| `/publish-minor` | `/publish-minor` | Copilot validates each step |

**Key difference**: Copilot prompts are interactive - you'll be asked to approve commands before execution, unlike Claude Code which runs them automatically after your approval.

## Customization

You can modify these prompt files to fit your workflow:

1. Open the `.prompt.md` file in VS Code
2. Edit the YAML frontmatter or Markdown content
3. Save the file
4. Changes take effect immediately

## Related Resources

- [VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/customization/overview)
- [Prompt Files Documentation](https://code.visualstudio.com/docs/copilot/customization/prompt-files)
- [Custom Instructions](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)
- [Community Examples](https://github.com/github/awesome-copilot)

## Project-Specific Notes

This project uses:
- **Rust** with Windows API
- **Cargo** for build management
- **PowerShell** as the default shell
- **GitHub Actions** for CI/CD
- **Beta versioning** for all commits
- **Stable versioning** for releases (patch/minor)

All prompts are configured with `mode: agent` to allow full access to tools and workspace operations.
