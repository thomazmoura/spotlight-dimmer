# Publishing Setup - Quick Start

This document explains the **automated multi-platform publishing** for Spotlight Dimmer.

## What Changed

✅ **Before**: Manual publishing to crates.io and npm
✅ **After**: Automated publishing to both platforms via GitHub Actions!

## How It Works

### 1. Automated Workflow (GitHub Actions)

When you push a git tag:

```bash
git tag v0.4.8
git push origin v0.4.8
```

The workflow automatically:
1. ✅ Builds Windows binaries on GitHub runners
2. ✅ Creates GitHub Release with ZIP artifacts
3. ✅ **Publishes to crates.io** (Rust users)
4. ✅ **Publishes to npm** with pre-built binaries (Node.js users)
5. ✅ Updates CHANGELOG.md

### 2. User Experience

Users install with zero compilation:

```bash
npm install -g spotlight-dimmer
```

Requirements: **Only Node.js 14+ and Windows x64** (no Rust!)

## Setup Steps (One-Time)

### Step 1: Get Cargo Token (for crates.io)

1. Go to https://crates.io/settings/tokens
2. Click "New Token"
3. Name it: "GitHub Actions"
4. Copy the token (starts with `cio_...`)

### Step 2: Get npm Token

1. Go to https://www.npmjs.com/settings/YOUR_USERNAME/tokens
2. Click "Generate New Token" → Choose "Automation"
3. Copy the token (starts with `npm_...`)

### Step 3: Add GitHub Secrets

1. Go to your repo → Settings → Secrets and variables → Actions
2. Add **two** repository secrets:

   **Secret 1: CARGO_TOKEN**
   - Click "New repository secret"
   - Name: `CARGO_TOKEN`
   - Value: Paste your crates.io token
   - Click "Add secret"

   **Secret 2: NPM_TOKEN**
   - Click "New repository secret"
   - Name: `NPM_TOKEN`
   - Value: Paste your npm token
   - Click "Add secret"

### Step 4: Publish Your First Release

```bash
# 1. Update versions
# Edit package.json: version → 0.4.8
# Edit src/Cargo.toml: version → 0.4.8

# 2. Update CHANGELOG.md with release notes

# 3. Commit and push
git add package.json src/Cargo.toml CHANGELOG.md
git commit -m "Bump version to 0.4.8"
git push

# 4. Create and push tag
git tag v0.4.8
git push origin v0.4.8

# 5. Watch GitHub Actions do the rest!
```

### Step 5: Verify Publication

1. Check GitHub Actions: https://github.com/thomazmoura/spotlight-dimmer/actions
2. Wait for all jobs to complete:
   - ✅ build-and-release (creates GitHub release)
   - ✅ publish-cargo (publishes to crates.io)
   - ✅ publish-npm (publishes to npm)
   - ✅ update-changelog (updates CHANGELOG.md)

3. Verify crates.io: https://crates.io/crates/spotlight-dimmer

4. Verify npm: https://www.npmjs.com/package/spotlight-dimmer

5. Test both installation methods:
   ```bash
   # Via cargo
   cargo install spotlight-dimmer

   # Via npm
   npm install -g spotlight-dimmer@0.4.8
   spotlight-dimmer-config status
   ```

## File Structure

```
spotlight-dimmer/
├── .github/workflows/release.yml  # ← Builds & publishes to npm
├── package.json                   # ← npm package config (no Rust deps!)
├── scripts/
│   ├── install.js                 # ← Verifies pre-built binaries
│   └── uninstall.js               # ← Stops running instances
├── bin/                           # ← Pre-built binaries (copied by CI)
│   ├── *.exe                      # ← Built by GitHub Actions
│   ├── *.ico                      # ← Built by GitHub Actions
│   └── *.cmd                      # ← Command wrappers
└── index.js                       # ← Shows usage info
```

## What Gets Published to npm

Only these files (no Rust source, smaller package):

- ✅ `bin/*.exe` - Pre-built executables
- ✅ `bin/*.ico` - Icon files
- ✅ `bin/*.cmd` - Command wrappers
- ✅ `scripts/` - Install/uninstall scripts
- ✅ `index.js` - Entry point
- ✅ `README.md` - Documentation
- ✅ `LICENSE` - License file

**Excluded** (via `.npmignore`):
- ❌ `src/` - Rust source code
- ❌ `target/` - Build artifacts
- ❌ `Cargo.toml` - Rust config
- ❌ Development files

## Package Size Comparison

- **With Rust source**: ~2-3 MB (source + dependencies)
- **Pre-built binaries**: ~1.2 MB (just executables + icons)

Users download smaller packages and install instantly!

## Troubleshooting

### Problem: npm publish fails with 403 error

**Solution**: Check NPM_TOKEN secret is set correctly in GitHub

### Problem: Binaries not found in published package

**Solution**: Ensure GitHub Actions completed successfully. Check the "Prepare npm package" step copied files to `bin/`

### Problem: Users report "executable not found"

**Solution**: Verify package.json `files` field includes `bin/*.exe`. Check published package at npmjs.com

## Testing Before Publishing

To test locally without publishing:

```bash
# Build binaries
cargo build --release

# Copy to bin/
cp target/release/spotlight-dimmer.exe bin/
cp target/release/spotlight-dimmer-config.exe bin/
cp spotlight-dimmer-icon.ico bin/
cp spotlight-dimmer-icon-paused.ico bin/

# Create tarball
npm pack

# Install locally
npm install -g spotlight-dimmer-0.4.7.tgz

# Test
spotlight-dimmer-config status

# Clean up
npm uninstall -g spotlight-dimmer
rm spotlight-dimmer-0.4.7.tgz
```

## Key Benefits

✅ **No Rust requirement** - Users only need Node.js
✅ **Instant installation** - No compilation time
✅ **Smaller package** - Only binaries, no source
✅ **Automated publishing** - GitHub Actions does everything
✅ **Better UX** - Standard npm workflow

## Next Steps

1. Set up NPM_TOKEN secret in GitHub (see Step 2 above)
2. Test the workflow with a pre-release (e.g., `v0.4.8-beta.1`)
3. Verify package installs correctly
4. Publish official release!

For detailed information, see [PUBLISHING.md](PUBLISHING.md).
