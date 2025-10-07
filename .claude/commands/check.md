# Check Command

**Description**: Run validation checks (tests, clippy, build) and automatically fix any errors

**Usage**: `/check`

## What this command does:

1. **Runs pre-commit validation pipeline**:
   - `cargo test` - Run all tests
   - `cargo clippy --all-targets --all-features -- -D warnings` - Check for code issues
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
   - First: `cargo test`
   - Second: `cargo clippy --all-targets --all-features -- -D warnings`
   - Third: `cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config`

2. **If any step fails**:
   - Stop the pipeline
   - Analyze the error output
   - Fix the errors (add `#[allow(dead_code)]`, fix warnings, update code, etc.)
   - Restart from step 1 (re-run all validation from the beginning)
   - Continue the fix-retry loop until all checks pass

3. **When all checks pass**:
   - Report success to the user
   - Confirm the codebase is ready for commit

## Important Notes:

- **Auto-fix enabled**: This command automatically fixes validation errors
- **Full re-validation**: After any fix, all checks run again from the start
- **Non-destructive**: Only fixes code quality issues, doesn't change functionality
- **Use before commit**: Run this before `/commit` to ensure a smooth commit process

## Example Workflow:

**Scenario 1**: All checks pass immediately
```
Running cargo test... ✓ 37 tests passed
Running cargo clippy... ✓ No warnings
Running cargo build --release... ✓ Built successfully

All validation checks passed! ✅
Your codebase is ready for commit.
```

**Scenario 2**: Validation fails, auto-fix, retry
```
Running cargo test... ✓ 37 tests passed
Running cargo clippy... ✗ 3 warnings found

Analyzing errors...
- Found unused function 'setup_test_config_dir'
- Found unused trait 'DisplayManager'
- Found unused method 'to_colorref'

Fixing errors by adding #[allow(dead_code)] annotations...

Re-running validation pipeline...
Running cargo test... ✓ 37 tests passed
Running cargo clippy... ✓ No warnings
Running cargo build --release... ✓ Built successfully

All validation checks passed! ✅
Your codebase is ready for commit.
```

**Scenario 3**: Multiple fix iterations
```
Running cargo test... ✗ 2 tests failed

Fixing test failures...

Re-running validation pipeline...
Running cargo test... ✓ 37 tests passed
Running cargo clippy... ✗ 1 warning found

Fixing clippy warning...

Re-running validation pipeline...
Running cargo test... ✓ 37 tests passed
Running cargo clippy... ✓ No warnings
Running cargo build --release... ✗ Compilation error

Fixing compilation error...

Re-running validation pipeline...
Running cargo test... ✓ 37 tests passed
Running cargo clippy... ✓ No warnings
Running cargo build --release... ✓ Built successfully

All validation checks passed! ✅
Your codebase is ready for commit.
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
