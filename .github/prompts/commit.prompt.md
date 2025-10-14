---
description: Create a beta version commit with validation and proper git workflow
mode: agent
---

# Commit Workflow

Create a versioned beta commit with full validation.

## Version Calculation

1. **Read current versions** from:
   - `package.json` → `version` field
   - `src/Cargo.toml` → `[package] version` field

2. **Calculate next beta version**:
   - If release (e.g., `0.1.10`): → `0.1.11-beta.1`
   - If beta (e.g., `0.1.11-beta.1`): → `0.1.11-beta.2`

3. **Update version files**:
   - `package.json`
   - `src/Cargo.toml`
   - Run: `cargo update -p spotlight-dimmer --precise X.Y.Z-beta.N`

## Pre-Commit Validation

**CRITICAL**: Run in order. If ANY fail, STOP and cancel commit.

```powershell
# Step 1: Library tests
cargo test --lib --verbose

# Step 2: Doc tests (allowed to fail)
cargo test --doc --verbose

# Step 3: Clippy checks
cargo clippy --all-targets --all-features -- -W clippy::all -A dead_code

# Step 4: Release build
cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config
```

**If validation fails:**
1. STOP immediately - DO NOT proceed
2. Revert version changes
3. Display error to user
4. Tell user: "Validation failed. Please run the check workflow first."
5. DO NOT attempt to auto-fix

## Format Code

```powershell
cargo fmt --all
```

## Generate Commit Message

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
- ✅ Brief explanation of changes
- ✅ List specific changes if multiple

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
git add . && git commit -m "YOUR_MESSAGE" && git pull --rebase origin main && git tag vX.Y.Z-beta.N && git push origin main && git push origin vX.Y.Z-beta.N
```

Replace:
- `YOUR_MESSAGE` with actual commit message
- `X.Y.Z-beta.N` with actual version (e.g., `v0.1.11-beta.1`)

## Important Notes

- Beta versioning for all commits
- Validation is mandatory
- NO auto-fix on validation failure
- Single atomic git operation
- Tag format: `vX.Y.Z-beta.N` (always include `v` prefix)
