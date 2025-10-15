# Publishing Spotlight Dimmer

This guide explains how to publish Spotlight Dimmer to **crates.io, npm, and Winget** with automated GitHub Actions.

## Overview

**Publishing is mostly automated via GitHub Actions!** When you push a git tag (e.g., `v0.6.1`), the workflow automatically:
1. Builds Windows binaries on GitHub Actions runners
2. Creates GitHub Release with a Windows installer and portable ZIP artifacts
3. **Publishes to crates.io** (for Rust users: `cargo install spotlight-dimmer`)
4. **Publishes to npm** with pre-built binaries (for Node.js users: `npm install -g spotlight-dimmer`)
5. **Generates Winget manifests** (ready for manual PR to microsoft/winget-pkgs)
6. Updates CHANGELOG.md

## Prerequisites

### 1. crates.io Account & Token

1. **Create account**: Visit https://crates.io/ and sign in with GitHub
2. **Generate token**:
   - Go to https://crates.io/settings/tokens
   - Click "New Token"
   - Name: "GitHub Actions"
   - Copy the token (starts with `cio_...`)

### 2. npm Account & Token

1. **Create account**: Visit https://www.npmjs.com/signup
2. **Generate token**:
   - Go to https://www.npmjs.com/settings/your-username/tokens
   - Click "Generate New Token" → "Automation"
   - Copy the token (starts with `npm_...`)

### 3. GitHub Secrets Setup

Add both tokens to your GitHub repository:

1. Go to your repository → Settings → Secrets and variables → Actions
2. Click "New repository secret" and add:

   **CARGO_TOKEN**
   - Name: `CARGO_TOKEN`
   - Value: Your crates.io token

   **NPM_TOKEN**
   - Name: `NPM_TOKEN`
   - Value: Your npm token

## Pre-publish Checklist

Before publishing, ensure:

1. ✅ All tests pass: `cargo test`
2. ✅ Code is formatted: `cargo fmt --check`
3. ✅ No linting issues: `cargo clippy`
4. ✅ Version is updated in both:
   - `package.json`
   - `src/Cargo.toml`
5. ✅ CHANGELOG.md is updated with latest changes
6. ✅ README.md has accurate installation instructions
7. ✅ Icon files exist:
   - `spotlight-dimmer-icon.ico`
   - `spotlight-dimmer-icon-paused.ico`

## Automated Publishing Workflow

### How It Works

The `.github/workflows/release.yml` workflow handles everything:

1. **Trigger**: Push a git tag matching `v*` (e.g., `v0.4.8`)
2. **Build**: Compiles Windows binaries on GitHub Actions
3. **Release**: Creates GitHub Release with NSIS installer and ZIP artifact
4. **Cargo Publish**: Publishes to crates.io
5. **npm Package**: Copies `.exe` and `.ico` files to `bin/`
6. **npm Publish**: Publishes to npm with pre-built binaries
7. **Changelog**: Updates CHANGELOG.md

### Publishing Steps

1. **Update version** in both files:
   ```bash
   # Update package.json version: 0.4.7 → 0.4.8
   # Update src/Cargo.toml version: 0.4.7 → 0.4.8
   ```

2. **Update CHANGELOG.md** with release notes

3. **Commit changes**:
   ```bash
   git add package.json src/Cargo.toml CHANGELOG.md
   git commit -m "Bump version to 0.4.8"
   git push
   ```

4. **Create and push git tag**:
   ```bash
   git tag v0.4.8
   git push origin v0.4.8
   ```

5. **Watch GitHub Actions**:
   - The workflow builds binaries
   - Creates GitHub Release with Windows installer and ZIP
   - Publishes to **crates.io** automatically
   - Publishes to **npm** automatically
   - Updates CHANGELOG.md

6. **Verify publication**:
   - **crates.io**: Visit `https://crates.io/crates/spotlight-dimmer`
   - **npm**: Visit `https://www.npmjs.com/package/spotlight-dimmer`
   - **Test cargo**: `cargo install spotlight-dimmer`
   - **Test npm**: `npm install -g spotlight-dimmer@0.4.8`

### Manual Publishing (Not Recommended)

If you need to publish manually (e.g., testing):

```bash
# Build binaries
cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config

# Copy to bin directory
cp target/release/*.exe bin/
cp *.ico bin/

# Verify package contents
npm pack --dry-run

# Publish
npm publish --access public
```

**Note**: Manual publishing should only be used for testing. Always use the automated workflow for official releases.

## Publishing Updates

For subsequent releases:

1. Update version in `package.json` and `src/Cargo.toml`
2. Update `CHANGELOG.md` with new changes
3. Commit changes to git
4. Create a git tag: `git tag v0.4.8`
5. Push tag: `git push origin v0.4.8`
6. Publish to npm: `npm publish`

## Version Management

This project uses [Semantic Versioning](https://semver.org/):

- **Patch** (0.4.7 → 0.4.8): Bug fixes, minor improvements
- **Minor** (0.4.7 → 0.5.0): New features, backward-compatible
- **Major** (0.4.7 → 1.0.0): Breaking changes

The `/commit` slash command automatically increments the patch version.

## Distribution Channels

Spotlight Dimmer is available through **four channels**:

1. **GitHub Releases** (recommended): Direct download
   - NSIS installer (`spotlight-dimmer-v*-installer.exe`) for guided setup
   - Portable ZIP with binaries/icons for manual installs
   - No package manager needed

2. **npm**: `npm install -g spotlight-dimmer`
   - Pre-built binaries included
   - No Rust toolchain required
   - Instant installation

3. **crates.io**: `cargo install spotlight-dimmer`
   - Builds from source
   - Requires Rust toolchain
   - For Rust developers

4. **Winget**: `winget install ThomazMoura.SpotlightDimmer`
   - Official Windows Package Manager
   - NSIS installer with silent install support
   - Requires manual PR submission (manifest generation is automated)

**Channels 1-3 are fully automated** when you push a git tag. **Winget (channel 4)** requires a manual Pull Request, but manifest generation is automated.

## Troubleshooting

### Problem: "You do not have permission to publish"

**Solution**: Ensure you're logged in to the correct npm account and have ownership of the package name.

```bash
npm whoami
npm owner ls spotlight-dimmer
```

### Problem: "Version already exists"

**Solution**: Increment the version number in `package.json` before publishing.

### Problem: Package size too large

**Solution**: Check `.npmignore` is properly excluding build artifacts:

```bash
npm pack --dry-run
```

Large files to exclude:
- `/target/` directory (Rust build artifacts)
- `.git/` directory
- Development files (`.vscode/`, `.devcontainer/`)

### Problem: Install script fails for users

**Solution**: The install script verifies pre-built binaries exist. If this fails:
- Ensure binaries were copied to `bin/` during GitHub Actions
- Check the `files` field in package.json includes `bin/*.exe`
- Verify the package was published from GitHub Actions, not manually

## Best Practices

1. **Test before publishing**: Always test local installation before publishing
2. **Update documentation**: Keep README.md and CHANGELOG.md current
3. **Semantic versioning**: Follow semver for version numbers
4. **Changelog**: Document all changes in CHANGELOG.md (bilingual EN/PT)
5. **Git tags**: Tag releases in git for traceability
6. **GitHub releases**: Sync npm releases with GitHub releases

## npm Package Settings

You may want to configure additional npm package settings:

### Enable 2FA (Recommended)

Protect your package from unauthorized publishes:

```bash
npm profile enable-2fa auth-and-writes
```

### Add Collaborators

Give others permission to publish:

```bash
npm owner add <username> spotlight-dimmer
```

### Set Access Level

By default, packages are public. To verify:

```bash
npm access public spotlight-dimmer
```

## Publishing to Winget

### How Winget Publishing Works

Unlike npm and crates.io, Winget packages are submitted via Pull Requests to the [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) repository. The manifest generation is automated, but the PR submission is manual.

### Automated Manifest Generation

When you push a git tag, the workflow automatically generates Winget manifests and uploads them as GitHub Actions artifacts. Each release generates three manifest files:

1. **Version manifest** (`ThomazMoura.SpotlightDimmer.yaml`) - Package metadata
2. **Installer manifest** (`ThomazMoura.SpotlightDimmer.installer.yaml`) - Download URL and SHA256 hash
3. **Locale manifest** (`ThomazMoura.SpotlightDimmer.locale.en-US.yaml`) - Human-readable descriptions

### Manual Workflow Trigger (For Existing Releases)

You can also generate manifests for existing releases manually:

```bash
# Via GitHub CLI
gh workflow run winget-publish.yml -f version=0.6.1

# Or via GitHub UI
# Go to Actions → Publish to Winget → Run workflow
# Enter version: 0.6.1
```

### Submission Steps

1. **Wait for release workflow** to complete (or trigger manual workflow)
2. **Download manifests** from GitHub Actions artifacts:
   - Go to the workflow run
   - Find "winget-manifests-X.Y.Z" artifact
   - Download and extract the ZIP file

3. **Fork winget-pkgs repository** (first time only):
   ```bash
   # Visit https://github.com/microsoft/winget-pkgs
   # Click "Fork" button
   ```

4. **Clone your fork** and add manifests:
   ```bash
   git clone https://github.com/YOUR_USERNAME/winget-pkgs
   cd winget-pkgs

   # Create new branch
   git checkout -b spotlight-dimmer-0.6.1

   # Copy the manifests (adjust path to your downloaded artifact)
   cp -r ~/Downloads/winget-manifests/t manifests/

   # Commit and push
   git add manifests/t/ThomazMoura/SpotlightDimmer/0.6.1/
   git commit -m "New version: ThomazMoura.SpotlightDimmer version 0.6.1"
   git push origin spotlight-dimmer-0.6.1
   ```

5. **Create Pull Request**:
   - Go to your fork on GitHub
   - Click "Contribute" → "Open pull request"
   - Title: `ThomazMoura.SpotlightDimmer version 0.6.1`
   - Submit the PR

6. **Wait for automated validation**:
   - Microsoft's bots will validate your manifests
   - Check for SHA256 hash correctness
   - Verify installer download and silent install
   - If validation passes, PR will be auto-merged

### Winget Requirements

The automated workflow ensures these requirements are met:

✅ **Silent install support** - NSIS installer configured for silent mode
✅ **Stable download URL** - GitHub Releases provides permanent URLs
✅ **Correct SHA256 hash** - Automatically calculated from installer
✅ **Manifest schema 1.6.0** - Latest Winget manifest format
✅ **No telemetry/adware** - Pure Rust application, no bundled software

### Troubleshooting Winget Submissions

#### Problem: Validation fails with "SHA256 mismatch"

**Solution**: The workflow automatically calculates the correct SHA256. If validation fails:
- Ensure the release was fully uploaded to GitHub
- Re-run the workflow to regenerate manifests
- Verify the installer URL is accessible: `https://github.com/thomazmoura/spotlight-dimmer/releases/download/vX.Y.Z/spotlight-dimmer-vX.Y.Z-installer.exe`

#### Problem: "Installer does not support silent install"

**Solution**: Our NSIS installer is configured for silent install via `install-mode = "currentUser"` in `packager.toml`. This shouldn't fail. If it does:
- Check that cargo-packager built the installer correctly
- Test silent install locally: `spotlight-dimmer-vX.Y.Z-installer.exe /S`

#### Problem: PR stuck in review

**Solution**: Winget PRs with new versions for existing packages usually auto-merge if validation passes. If stuck:
- Check the PR comments for validation errors
- Ensure all three manifest files are present
- Verify the package identifier matches: `ThomazMoura.SpotlightDimmer`

## Support

For issues with publishing:
- **npm documentation**: https://docs.npmjs.com/
- **npm support**: https://www.npmjs.com/support
- **Winget documentation**: https://learn.microsoft.com/en-us/windows/package-manager/package/
- **Winget repository**: https://github.com/microsoft/winget-pkgs
- **GitHub issues**: https://github.com/thomazmoura/spotlight-dimmer/issues
