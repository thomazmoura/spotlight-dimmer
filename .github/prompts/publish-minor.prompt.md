---
description: Create a minor release by incrementing the minor version (0.X.0)
mode: agent
---

# Publish Minor Release

Create a minor release for new features (backward-compatible functionality).

## Version Calculation

1. **Read current versions** from:
   - `package.json` → `version` field
   - `src/Cargo.toml` → `[package] version` field

2. **Calculate minor version**:
   - Pattern: `X.Y.Z` → `X.(Y+1).0`
   - Examples:
     - `0.4.9` → `0.5.0`
     - `0.4.9-beta.2` → `0.5.0`
   - Rules:
     - Increment minor number (Y)
     - Reset patch to 0
     - Remove any beta suffix

3. **Update version files**:
   - `package.json`
   - `src/Cargo.toml`
   - Run: `cargo update -p spotlight-dimmer --precise X.Y.0`

## Pre-Release Validation

**CRITICAL**: Run in order. If ANY fail, STOP and cancel release.

```powershell
# Step 1: Library tests (with Wine for Windows target)
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER="wine64"
cargo test --lib --verbose --target x86_64-pc-windows-gnu

# Step 2: Doc tests
cargo test --doc --verbose

# Step 3: Clippy checks (Windows target)
cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code

# Step 4: Release build (Windows binaries)
cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config
```

**If validation fails:**
1. STOP immediately - DO NOT proceed
2. Revert version changes
3. Display error to user
4. Tell user: "Validation failed. Please run the check workflow first."
5. DO NOT attempt to auto-fix

## Generate Release Commit Message

Review changes:
```powershell
git status
git diff
```

**Commit Message Requirements:**

**DO NOT include:**
- ❌ "Generated with Claude Code"
- ❌ "Co-Authored-By: Claude"

**DO include:**
- ✅ Subject line (50 chars max)
- ✅ Brief explanation of new features
- ✅ List specific features added

**Example:**
```
Add profile support for saving display configurations

Implemented profile system allowing users to save and switch between different dimming configurations.

- Add profile CRUD operations to config module
- Implement profile switching via system tray menu
- Add profile management CLI commands
- Update documentation with profile usage
```

## Execute Git Workflow

**Present as ONE command using `&&`:**

```powershell
git add . && git commit -m "YOUR_MESSAGE" && git pull --rebase origin main && git tag vX.Y.0 && git push origin main && git push origin vX.Y.0
```

Replace:
- `YOUR_MESSAGE` with actual commit message
- `X.Y.0` with actual version (e.g., `v0.5.0`)

## When to Use

Use minor releases for:
- ✅ New features (backward-compatible)
- ✅ New configuration options
- ✅ New commands or capabilities
- ✅ Significant improvements

Don't use for:
- ❌ Bug fixes (use patch release)
- ❌ Breaking changes (would need major version)
- ❌ Small tweaks (use patch release)

## Important Notes

- Stable release (no beta suffix)
- Validation is mandatory
- NO auto-fix on validation failure
- Single atomic git operation
- Tag format: `vX.Y.0` (always include `v` prefix)
- Patch is always 0 for minor releases
