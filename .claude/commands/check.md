# Check Command

**Description**: Run validation checks (tests, clippy, build) and automatically fix any errors

**Usage**: `/check`

## What this command does:

1. **Runs validation pipeline**:
   - `cargo test --lib --verbose` - Run library tests
   - `cargo test --doc --verbose` - Run doc tests
   - `cargo clippy --all-targets --all-features -- -W clippy::all -A dead_code` - Check for code issues
   - `cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config` - Build release binaries

2. **If any validation step fails**:
   - Analyzes the error output
   - Fixes the errors (code fixes, dependency updates, etc.)
   - Restarts from step 1 (re-run all validation)
   - Continues until all validation passes

3. **Reports success** when all checks pass

## Process:

The agent will:
1. **Run validation pipeline in order**:
   - First: `cargo test --lib --verbose` (library tests)
   - Second: `cargo test --doc --verbose` (doc tests, allowed to fail)
   - Third: `cargo clippy --all-targets --all-features -- -W clippy::all -A dead_code` (clippy)
   - Fourth: `cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config` (release binaries)

2. **If any step fails**:
   - Stop the pipeline
   - Analyze the error output
   - Fix the errors (add `#[allow(dead_code)]`, fix warnings, update code, etc.)
   - Restart from step 1 (re-run all validation from the beginning)
   - Continue the fix-retry loop until all checks pass

3. **When all checks pass**:
   - Report success to the user
   - Show location of built binaries (`target/release/*.exe`)
   - Confirm the codebase is ready for commit

## Important Notes:

- **Auto-fix enabled**: This command automatically fixes validation errors
- **Full re-validation**: After any fix, all checks run again from the start
- **Non-destructive**: Only fixes code quality issues, doesn't change functionality
- **Use before commit**: Run this before `/commit` to ensure a smooth commit process

## Example Workflow:

**Scenario 1**: All checks pass immediately
```
Running cargo test --lib --verbose... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features -- -W clippy::all -A dead_code... ✓ No warnings
Running cargo build --release... ✓ Built successfully

All validation checks passed! ✅
Binaries built at:
  - target/release/spotlight-dimmer.exe
  - target/release/spotlight-dimmer-config.exe
Your codebase is ready for commit.
```

**Scenario 2**: Validation fails, auto-fix, retry
```
Running cargo test --lib --verbose... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features -- -W clippy::all -A dead_code... ✗ 3 warnings found

Analyzing errors...
- Found unused function 'setup_test_config_dir'
- Found unused variable in test
- Found needless borrow

Fixing errors...

Re-running validation pipeline...
Running cargo test --lib --verbose... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features -- -W clippy::all -A dead_code... ✓ No warnings
Running cargo build --release... ✓ Built successfully

All validation checks passed! ✅
Binaries ready for testing.
```

## When to Use This Command:

- ✅ Before creating a commit (run `/check` then `/commit`)
- ✅ After making code changes to verify everything still works
- ✅ To fix code quality issues automatically
- ✅ To ensure tests, clippy, and build are all passing

## Validation Steps Details:

### 1. cargo test
- Runs all unit tests and integration tests
- Ensures code correctness
- Verifies no regressions

### 2. cargo clippy
- Runs with `--all-targets --all-features`
- Treats all warnings as errors (`-D warnings`)
- Catches code quality issues, potential bugs, and style violations

### 3. cargo build --release
- Builds both binaries: `spotlight-dimmer` and `spotlight-dimmer-config`
- Ensures production builds succeed
- Verifies optimized compilation works
