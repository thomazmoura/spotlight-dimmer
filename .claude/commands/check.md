# Check Command

**Description**: Run validation checks (tests, clippy, build) matching the CI pipeline exactly using cross-compilation for Windows with Wine, and automatically fix any errors

**Usage**: `/check`

## What this command does:

1. **Sets up cross-compilation toolchain**:
   - Ensures `x86_64-pc-windows-gnu` target is installed
   - Installs MinGW cross-compiler if needed (`mingw-w64`)
   - Verifies Wine is available for running Windows tests
   - Enables building and testing Windows binaries from Linux

2. **Runs validation pipeline (matching CI exactly)**:
   - `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine64 cargo test --lib --verbose --target x86_64-pc-windows-gnu` - Run library tests via Wine (matching CI test job)
     - **Note**: Tests are compiled for Windows and executed via Wine64
     - Wine allows actual test execution on Linux, not just compilation
   - `cargo test --doc --verbose` - Run doc tests (matching CI test job)
   - `cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code` - Check for code issues (matching CI clippy job)
   - `cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config` - Build Windows binaries (matching CI build job)

3. **If any validation step fails**:
   - Analyzes the error output
   - Fixes the errors (code fixes, dependency updates, etc.)
   - Restarts from step 1 (re-run all validation)
   - Continues until all validation passes

4. **Reports success** when all checks pass

## Process:

The agent will:
1. **Setup cross-compilation toolchain**:
   - Check if `x86_64-pc-windows-gnu` target is installed
   - If not installed: Run `rustup target add x86_64-pc-windows-gnu`
   - Check if MinGW cross-compiler is available
   - If not available: Install `mingw-w64` package
   - Verify Wine is installed (required for running tests)

2. **Run validation pipeline in order (matching CI exactly)**:
   - First: `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine64 cargo test --lib --verbose --target x86_64-pc-windows-gnu` (library tests via Wine)
   - Second: `cargo test --doc --verbose` (doc tests, allowed to fail)
   - Third: `cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code` (clippy with Windows target and CI flags)
   - Fourth: `cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config` (Windows binaries)

3. **If any step fails**:
   - Stop the pipeline
   - Analyze the error output
   - Fix the errors (add `#[allow(dead_code)]`, fix warnings, update code, etc.)
   - Restart from step 1 (re-run all validation from the beginning)
   - Continue the fix-retry loop until all checks pass

4. **When all checks pass**:
   - Report success to the user
   - Show location of built Windows binaries (`target/x86_64-pc-windows-gnu/release/*.exe`)
   - Confirm the codebase is ready for commit

## Important Notes:

- **Matches CI exactly**: Uses the same commands, flags, and target as GitHub Actions CI pipeline
- **Cross-compilation with Wine**: Builds Windows binaries on Linux using MinGW toolchain and runs them via Wine64
- **Full validation**: Validates AND executes ALL code including `#[cfg(windows)]` sections
- **Auto-fix enabled**: This command automatically fixes validation errors
- **Full re-validation**: After any fix, all checks run again from the start
- **Non-destructive**: Only fixes code quality issues, doesn't change functionality
- **Use before commit**: Run this before `/commit` to ensure a smooth commit process
- **Requires toolchain setup**: First run will install `x86_64-pc-windows-gnu` target and MinGW
- **Wine integration**: Tests actually execute via Wine64, providing real test results instead of just compilation validation

## Example Workflow:

**Scenario 1**: All checks pass immediately (with cross-compilation)
```
Setting up cross-compilation toolchain...
✓ Target x86_64-pc-windows-gnu already installed
✓ MinGW cross-compiler available

Running cargo test --lib --verbose --target x86_64-pc-windows-gnu... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code... ✓ No warnings
Running cargo build --release --target x86_64-pc-windows-gnu... ✓ Built successfully

All validation checks passed! ✅
Windows binaries built at:
  - target/x86_64-pc-windows-gnu/release/spotlight-dimmer.exe
  - target/x86_64-pc-windows-gnu/release/spotlight-dimmer-config.exe
Your codebase is ready for commit.
```

**Scenario 2**: Validation fails, auto-fix, retry
```
Setting up cross-compilation toolchain...
✓ Target x86_64-pc-windows-gnu already installed

Running cargo test --lib --verbose --target x86_64-pc-windows-gnu... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code... ✗ 3 warnings found

Analyzing errors...
- Found unused function 'setup_test_config_dir'
- Found unused variable in test
- Found needless borrow

Fixing errors...

Re-running validation pipeline...
Running cargo test --lib --verbose --target x86_64-pc-windows-gnu... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code... ✓ No warnings
Running cargo build --release --target x86_64-pc-windows-gnu... ✓ Built successfully

All validation checks passed! ✅
Windows binaries ready for testing.
```

**Scenario 3**: Multiple fix iterations (including Windows-specific errors)
```
Setting up cross-compilation toolchain...
✓ Target x86_64-pc-windows-gnu already installed

Running cargo test --lib --verbose --target x86_64-pc-windows-gnu... ✗ 2 tests failed

Fixing test failures...

Re-running validation pipeline...
Running cargo test --lib --verbose --target x86_64-pc-windows-gnu... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code... ✗ 1 warning found

Fixing clippy warning...

Re-running validation pipeline...
Running cargo test --lib --verbose --target x86_64-pc-windows-gnu... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code... ✓ No warnings
Running cargo build --release --target x86_64-pc-windows-gnu... ✗ Compilation error in Windows code (missing imports)

Fixing compilation error (adding missing imports to main_new.rs)...

Re-running validation pipeline...
Running cargo test --lib --verbose --target x86_64-pc-windows-gnu... ✓ 37 tests passed
Running cargo test --doc --verbose... ✓ 0 doc tests passed
Running cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code... ✓ No warnings
Running cargo build --release --target x86_64-pc-windows-gnu... ✓ Built successfully

All validation checks passed! ✅
✅ Windows-specific code validated successfully (would have caught today's CI error!)
Windows binaries ready for testing.
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
