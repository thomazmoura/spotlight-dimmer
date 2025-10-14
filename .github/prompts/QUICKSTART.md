# Quick Start Guide for GitHub Copilot Prompt Files

## 🚀 Setup (First Time Only)

### Step 1: Enable Prompt Files

Open VS Code settings and enable prompt files:

**Quick Method:**
1. Press `Ctrl+,` (Windows/Linux) or `Cmd+,` (Mac)
2. Search for `chat.promptFiles`
3. Check the box ✅

**Alternative Method:**
1. Press `Ctrl+Shift+P` or `Cmd+Shift+P`
2. Type "Open User Settings (JSON)"
3. Add this line:
   ```json
   "chat.promptFiles": true
   ```

### Step 2: Reload VS Code (Optional)

Press `Ctrl+Shift+P` / `Cmd+Shift+P` and run "Reload Window"

---

## 🎯 How to Use

### In Copilot Chat View

1. Open Chat: Click the Copilot icon in the activity bar
2. Type `/` followed by the prompt name:
   - `/check` - Run validation checks
   - `/check-ci` - Check CI build status
   - `/commit` - Create beta commit
   - `/publish-patch` - Create patch release
   - `/publish-minor` - Create minor release

### Examples

**Run validation before committing:**
```
/check
```

**Check CI status after push:**
```
/check-ci
```

**Create a beta commit:**
```
/commit
```

**Publish bug fixes:**
```
/publish-patch
```

**Publish new features:**
```
/publish-minor
```

---

## 📋 Available Commands

| Command | Description | When to Use |
|---------|-------------|-------------|
| `/check` | Run tests, clippy, build | Before committing |
| `/check-ci` | Check GitHub Actions CI | After pushing code |
| `/commit` | Create beta version commit | After making changes |
| `/publish-patch` | Release bug fixes (0.0.X) | For bug fixes only |
| `/publish-minor` | Release new features (0.X.0) | For new features |

---

## 💡 Tips

1. **Always run `/check` before `/commit`**
   - This ensures your code passes all validation
   - Prevents failed commits

2. **Use `/check-ci` after pushing**
   - Verifies CI build succeeded
   - Auto-fixes any CI errors

3. **Copilot asks for confirmation**
   - Unlike Claude Code, Copilot shows you commands before running
   - Review carefully before approving

4. **Pass additional context**
   ```
   /commit Add dark mode feature
   /check After fixing memory leak
   ```

---

## 🔍 Troubleshooting

### Prompt files not showing up?

1. **Check settings are enabled:**
   - Open Settings (Ctrl+,)
   - Search "chat.promptFiles"
   - Should be checked ✅

2. **Verify file location:**
   - Prompts must be in `.github/prompts/`
   - Must have `.prompt.md` extension

3. **Reload VS Code:**
   - Press Ctrl+Shift+P
   - Run "Reload Window"

### GitHub CLI errors with `/check-ci`?

Install and authenticate GitHub CLI:
```powershell
# Install
winget install GitHub.cli

# Authenticate
gh auth login
```

### Commands not executing?

- Copilot shows commands for approval
- You must confirm each command
- This is different from Claude Code's automatic execution

---

## 📚 Learn More

- **Full Documentation**: See `README.md` in `.github/prompts/`
- **VS Code Docs**: https://code.visualstudio.com/docs/copilot/customization/prompt-files
- **Project Guidelines**: See `AGENTS.md` in project root

---

## 🎓 Workflow Example

**Complete development cycle:**

1. **Make code changes**
2. **Run validation:**
   ```
   /check
   ```
3. **If validation passes, commit:**
   ```
   /commit
   ```
4. **After push, verify CI:**
   ```
   /check-ci
   ```
5. **When ready to release, choose:**
   ```
   /publish-patch    # For bug fixes
   /publish-minor    # For new features
   ```

---

## ⚙️ Advanced Configuration

### Workspace-Specific Settings

Create `.vscode/settings.json`:
```json
{
  "chat.promptFiles": true,
  "chat.promptFilesLocations": [
    ".github/prompts"
  ]
}
```

### User Prompts (Available Everywhere)

Create your own prompts in:
- **Windows**: `%USERPROFILE%\.vscode\prompts\`
- **Mac/Linux**: `~/.vscode/prompts/`

---

**Need Help?** Open an issue or check the project documentation!
