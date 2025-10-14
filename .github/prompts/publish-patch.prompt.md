---
description: Create a patch release by incrementing the patch version (0.0.X)
mode: agent
---

# Publish Patch Release

Create a patch release for bug fixes and small changes (no new features).

## Version Calculation

1. **Read current versions** from:
   - `package.json` → `version` field
   - `src/Cargo.toml` → `[package] version` field

2. **Calculate patch version**:
   - Pattern: `X.Y.Z` → `X.Y.(Z+1)`
   - Examples:
     - `0.4.9` → `0.4.10`
     - `0.4.9-beta.2` → `0.4.10`
   - Rules:
     - Increment patch number (Z)
     - Keep major and minor the same
     - Remove any beta suffix

3. **Update version files**:
   - `package.json`
   - `src/Cargo.toml`
   - Run: `cargo update -p spotlight-dimmer --precise X.Y.Z`

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
- ✅ Brief explanation of fixes
- ✅ List specific bug fixes

**Example:**
```
Fix window dragging crash and ghost windows

Implemented intelligent mouse button detection to prevent system instability.

- Add GetAsyncKeyState mouse button detection
- Hide overlays when mouse button pressed
- Restore overlays when mouse released
- Document known limitation
```

## Execute Git Workflow

**Present as ONE command using `&&`:**

```powershell
git add . && git commit -m "YOUR_MESSAGE" && git pull --rebase origin main && git tag vX.Y.Z && git push origin main && git push origin vX.Y.Z
```

Replace:
- `YOUR_MESSAGE` with actual commit message
- `X.Y.Z` with actual version (e.g., `v0.4.10`)

## When to Use

Use patch releases for:
- ✅ Bug fixes
- ✅ Performance optimizations
- ✅ Documentation updates
- ✅ Small code quality improvements
- ✅ Dependency updates (if needed)

Don't use for:
- ❌ New features (use minor release)
- ❌ Breaking changes (would need major version)
- ❌ Adding configuration options (use minor release)

## Important Notes

- Stable release (no beta suffix)
- Validation is mandatory
- NO auto-fix on validation failure
- Single atomic git operation
- Tag format: `vX.Y.Z` (always include `v` prefix)
