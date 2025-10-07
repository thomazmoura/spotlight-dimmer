# Publishing Spotlight Dimmer

This guide explains how to publish Spotlight Dimmer to **both crates.io and npm** with automated GitHub Actions.

## Overview

**Publishing is fully automated via GitHub Actions!** When you push a git tag (e.g., `v0.4.8`), the workflow automatically:
1. Builds Windows binaries on GitHub Actions runners
2. Creates GitHub Release with ZIP artifacts
3. **Publishes to crates.io** (for Rust users: `cargo install spotlight-dimmer`)
4. **Publishes to npm** with pre-built binaries (for Node.js users: `npm install -g spotlight-dimmer`)
5. Updates CHANGELOG.md

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
3. **Release**: Creates GitHub Release with ZIP artifact
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
   - Creates GitHub Release with ZIP
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

Spotlight Dimmer is available through **three automated channels**:

1. **npm** (recommended): `npm install -g spotlight-dimmer`
   - Pre-built binaries included
   - No Rust toolchain required
   - Instant installation

2. **crates.io**: `cargo install spotlight-dimmer`
   - Builds from source
   - Requires Rust toolchain
   - For Rust developers

3. **GitHub Releases**: Direct download
   - Pre-built Windows binaries (ZIP)
   - No package manager needed
   - Manual installation

**All three channels are automatically updated** when you push a git tag!

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

## Support

For issues with npm publishing:
- npm documentation: https://docs.npmjs.com/
- npm support: https://www.npmjs.com/support
- GitHub issues: https://github.com/thomazmoura/spotlight-dimmer/issues
