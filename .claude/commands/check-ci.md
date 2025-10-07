# Check CI Command

**Description**: Fetch latest CI build results, identify errors, and automatically fix them

**Usage**: `/check-ci`

## What this command does:

1. **Fetches latest CI build results** from GitHub Actions
2. **Analyzes build logs** for errors and failures
3. **If errors are found**: Automatically fixes them and runs validation
4. **If no errors**: Reports success status

## Process:

The agent will:
1. **Fetch CI status** using `gh run list` to get the most recent workflow run
2. **Download build logs** using `gh run view` to see detailed error output
3. **Analyze errors**:
   - Test failures
   - Compilation errors
   - Clippy warnings
   - Build failures
   - Linting issues
4. **If errors found**:
   - Identify the root cause of each error
   - Fix the code issues (similar to `/check` command)
   - Run local validation to confirm fixes work
   - Report what was fixed
5. **If no errors**:
   - Report CI success status
   - Display build time and workflow details

## Important Notes:

- **Requires GitHub CLI**: Uses `gh` command to interact with GitHub Actions
- **Auto-fix enabled**: This command automatically fixes CI errors
- **Local validation**: After fixing, runs local tests to confirm fixes work
- **Non-destructive**: Only fixes code quality issues, doesn't change functionality
- **Use after push**: Run this after pushing to see if CI caught any issues

## Example Workflow:

**Scenario 1**: CI passed - No errors
```
Fetching latest CI run...
✓ Workflow: Rust CI
✓ Status: completed
✓ Conclusion: success
✓ Duration: 2m 34s

All CI checks passed! ✅
No errors to fix.
```

**Scenario 2**: CI failed - Tests failing
```
Fetching latest CI run...
✗ Workflow: Rust CI
✗ Status: completed
✗ Conclusion: failure

Downloading build logs...

Analyzing errors...
Found 2 test failures:
  - config::tests::test_profile_serialization
  - platform::tests::test_display_info_creation

Error details:
---- config::tests::test_profile_serialization stdout ----
thread 'config::tests::test_profile_serialization' panicked at 'assertion failed: `(left == right)`
  left: `"0.4.9"`,
 right: `"0.4.10"`'

Fixing test failures...
✓ Updated test assertions to match current version

Running local validation...
✓ cargo test - 37 tests passed
✓ cargo clippy - No warnings
✓ cargo build --release - Built successfully

All errors fixed! ✅
You can now commit and push the fixes.
```

**Scenario 3**: CI failed - Clippy warnings
```
Fetching latest CI run...
✗ Workflow: Rust CI
✗ Status: completed
✗ Conclusion: failure

Downloading build logs...

Analyzing errors...
Found 3 clippy warnings treated as errors:
  - Unused function 'setup_test_helper'
  - Unused import in config.rs
  - Needless borrow in main_new.rs

Fixing clippy warnings...
✓ Added #[allow(dead_code)] for test helper
✓ Removed unused import
✓ Fixed needless borrow

Running local validation...
✓ cargo test - 37 tests passed
✓ cargo clippy - No warnings
✓ cargo build --release - Built successfully

All errors fixed! ✅
You can now commit and push the fixes.
```

**Scenario 4**: CI failed - Compilation error
```
Fetching latest CI run...
✗ Workflow: Rust CI
✗ Status: completed
✗ Conclusion: failure

Downloading build logs...

Analyzing errors...
Found compilation error:
error[E0425]: cannot find value `config` in this scope
  --> src/main_new.rs:145:20
   |
145|     let color = config.overlay_color;
   |                 ^^^^^^ not found in this scope

Fixing compilation error...
✓ Added missing config variable initialization

Running local validation...
✓ cargo test - 37 tests passed
✓ cargo clippy - No warnings
✓ cargo build --release - Built successfully

All errors fixed! ✅
You can now commit and push the fixes.
```

## When to Use This Command:

- ✅ After pushing code to check if CI passed
- ✅ When GitHub Actions reports a failure
- ✅ To debug CI-specific issues that don't occur locally
- ✅ To automatically fix CI failures without manual intervention
- ✅ Before creating a pull request to ensure CI will pass

## CI Integration:

This command works with the GitHub Actions workflows in `.github/workflows/`:
- `rust.yml` - Rust compilation, testing, and clippy checks
- Any custom workflows you've configured

## GitHub CLI Commands Used:

```bash
# List recent workflow runs
gh run list --limit 1

# View specific run details
gh run view <run-id>

# Download run logs
gh run view <run-id> --log

# Check run status
gh run view <run-id> --json status,conclusion,workflowName
```

## Error Types Handled:

1. **Test Failures**: Analyzes test output and fixes assertions, logic errors
2. **Compilation Errors**: Fixes syntax errors, missing imports, type mismatches
3. **Clippy Warnings**: Adds annotations, fixes code style issues
4. **Build Failures**: Resolves dependency issues, configuration problems
5. **Platform-Specific Issues**: Fixes cross-platform compilation differences

## Validation After Fixes:

After fixing any errors, the command runs the same validation pipeline as `/check`:
1. `cargo test` - Ensure tests pass
2. `cargo clippy --all-targets --all-features -- -D warnings` - Verify no warnings
3. `cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config` - Confirm build succeeds

## Recommended Workflow:

1. Make code changes locally
2. Run `/check` to validate locally
3. Commit with `/commit` (or `/publish-patch`/`/publish-minor`)
4. Wait for CI to complete (or check GitHub Actions tab)
5. Run `/check-ci` to see if CI passed and auto-fix any issues
6. If fixes were made, commit again with a message like "Fix CI errors"
