---
description: Run validation checks (tests, clippy, build) and automatically fix any errors
mode: agent
---

# Validation Check Workflow

Run the complete validation pipeline (tests, clippy, build) and automatically fix any errors found.

## Validation Pipeline

Run these commands in order:

### 1. Library Tests
```powershell
cargo test --lib --verbose
```
- Must pass before proceeding
- If fails: Analyze, fix, restart from step 1

### 2. Doc Tests
```powershell
cargo test --doc --verbose
```
- Allowed to fail (may have no doc tests)
- Continue even if this fails

### 3. Clippy Checks
```powershell
cargo clippy --all-targets --all-features -- -W clippy::all -A dead_code
```
- Must have no warnings
- If fails: Analyze, fix, restart from step 1

### 4. Release Build
```powershell
cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config
```
- Must build successfully
- If fails: Analyze, fix, restart from step 1

## Auto-Fix Strategy

When any step fails:

1. **Stop immediately**
2. **Analyze error output**:
   - Test failures
   - Clippy warnings (unused code, needless borrows, etc.)
   - Compilation errors
   - Build failures

3. **Apply fixes**:
   - Add `#[allow(dead_code)]` for intentionally unused code
   - Remove unused imports/variables
   - Fix clippy suggestions
   - Update dependencies if needed

4. **Restart from step 1** - Run ENTIRE pipeline again

5. **Repeat until all checks pass**

## Success Criteria

All steps must pass. When successful, report:
- Number of tests passed
- Number of doc tests
- Clippy status (no warnings)
- Build status (successful)
- Binary locations in `target/release/`

## Important Notes

- Auto-fix enabled
- Full re-validation after any fix
- Non-destructive changes only
- Use before committing code
