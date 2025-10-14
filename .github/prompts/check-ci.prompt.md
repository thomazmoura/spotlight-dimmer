---
description: Fetch latest CI build results, identify errors, and automatically fix them
mode: agent
---

# Check CI Workflow

Fetch the latest CI build results from GitHub Actions, analyze any errors, and automatically fix them.

## Process

1. **Fetch CI status** using GitHub CLI:
   ```powershell
   gh run list --limit 1 --json databaseId,status,conclusion,workflowName,createdAt
   ```

2. **Download build logs** if the CI failed:
   ```powershell
   gh run view --log
   ```

3. **Analyze errors** found in the logs:
   - Test failures (assertion errors, panics)
   - Compilation errors (rustc errors)
   - Clippy warnings treated as errors
   - Build failures (linker errors)
   - Format violations

4. **If errors are found**:
   - Identify the root cause from error messages
   - Fix the code issues
   - Run local validation:
     ```powershell
     cargo test --lib --verbose
     cargo clippy --all-targets --all-features -- -W clippy::all -A dead_code
     cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config
     ```
   - Report what was fixed

5. **If no errors found**:
   - Report CI success status
   - Display build duration and workflow details

## Important Notes

- Requires GitHub CLI installed and authenticated
- Auto-fixes code quality issues only
- Non-destructive changes
- Always validate fixes locally before reporting success
