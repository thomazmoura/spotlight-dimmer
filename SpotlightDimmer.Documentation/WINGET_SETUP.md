# Winget Publishing Setup

This document explains how to set up and use the automated Winget manifest generation workflow.

## Overview

The `.github/workflows/winget-publish.yml` workflow automates publishing new versions of SpotlightDimmer to the [Windows Package Manager Community Repository](https://github.com/microsoft/winget-pkgs).

## One-Time Setup

### 1. Fork the winget-pkgs Repository

Fork [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) to your GitHub account. This fork will be used by `winget-create` to submit pull requests.

### 2. Create a GitHub Personal Access Token (PAT)

1. Go to **GitHub Settings** → **Developer settings** → **Personal access tokens** → **Tokens (classic)**
2. Click **Generate new token (classic)**
3. Set a descriptive name: `Winget Publishing - SpotlightDimmer`
4. Select the following scope:
   - ✅ **`public_repo`** - Access public repositories
5. Set an appropriate expiration date (consider 90 days or 1 year)
6. Click **Generate token**
7. **⚠️ Copy the token immediately** - you won't be able to see it again!

### 3. Add Token as Repository Secret

1. Go to your `spotlight-dimmer` repository
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Name: `WINGET_TOKEN`
5. Value: Paste the PAT you created in step 2
6. Click **Add secret**

## Publishing a New Version

### Prerequisites

Before running the workflow, ensure:

1. ✅ You've created a new release (e.g., `v0.9.0`)
2. ✅ The release is **published** (not a draft)
3. ✅ The release includes `spotlight-dimmer-installer.exe` as an asset

The release workflow (`.github/workflows/release.yml`) automatically creates releases with the correct assets when you push a version tag.

### Running the Workflow

1. Go to **Actions** → **Publish to Winget** in your repository
2. Click **Run workflow**
3. Enter the version number (e.g., `0.9.0`) - **without the `v` prefix**
4. Click **Run workflow**

### What Happens

The workflow will:

1. ✅ Validate the version format
2. ✅ Check that the GitHub release exists
3. ✅ Download `winget-create` (Microsoft's official manifest tool)
4. ✅ Update the Winget manifest by:
   - Downloading the installer to analyze it
   - Auto-detecting installer type (Inno Setup)
   - Calculating SHA256 hash
   - Preserving existing metadata (description, publisher info, etc.)
5. ✅ Submit a pull request to `microsoft/winget-pkgs` via your fork

### After the Workflow Completes

1. Check your fork: `https://github.com/YOUR_USERNAME/winget-pkgs/pulls`
2. The PR should appear automatically in the main repository
3. Wait for automated validation checks (usually 5-10 minutes):
   - SmartScreen validation
   - Binary validation
   - Manifest validation
   - Installation test
4. If checks pass, a Winget maintainer will review and merge (usually within 24-48 hours)
5. Once merged, the new version will be available via `winget install ThomazMoura.SpotlightDimmer`

## Troubleshooting

### "Release not found" Error

- Ensure the release is **published** (not a draft)
- Verify you entered the version without the `v` prefix (use `0.9.0`, not `v0.9.0`)
- Check that the release tag matches `v{version}` format

### "Failed to download installer" Error

- Verify the release includes `spotlight-dimmer-installer.exe`
- Check that the release is public (not in a private repository)

### "Authentication failed" Error

- Verify the `WINGET_TOKEN` secret is configured correctly
- Check that your PAT hasn't expired
- Ensure the PAT has `public_repo` scope

### "Failed to create PR" Error

- Verify you've forked `microsoft/winget-pkgs`
- Ensure your fork is up-to-date with upstream
- Check if there's already an open PR for this version

### Validation Check Failures

If automated checks fail on the PR:

- **SmartScreen validation failed**: Wait 24-48 hours for SmartScreen reputation to build, then re-run checks
- **Binary validation failed**: Ensure the installer is correctly signed (if applicable)
- **Manifest validation failed**: Check the PR comments for specific schema errors
- **Installation test failed**: Verify the installer works correctly when downloaded manually

## Technical Details

### Tool Used: winget-create

We use Microsoft's official [winget-create](https://github.com/microsoft/winget-create) tool because:

- ✅ Official Microsoft tool for manifest generation
- ✅ Auto-detects installer characteristics (type, SHA256, architecture)
- ✅ Preserves existing metadata during updates
- ✅ Handles complex installer types (Inno Setup, NSIS, MSI, etc.)
- ✅ Supports automatic PR submission

### Manifest Files

Each version creates three files in `microsoft/winget-pkgs`:

```
manifests/t/ThomazMoura/SpotlightDimmer/{version}/
├── ThomazMoura.SpotlightDimmer.yaml                 # Version metadata
├── ThomazMoura.SpotlightDimmer.installer.yaml       # Installer details
└── ThomazMoura.SpotlightDimmer.locale.en-US.yaml    # Localized descriptions
```

### Installer Type Detection

`winget-create` downloads and analyzes the installer to auto-detect:

- **Installer Type**: `inno` (Inno Setup) - changed from `nullsoft` (NSIS) in v0.7.2
- **Architecture**: `x64`
- **Scope**: `user` (no admin required)
- **Silent Flags**: `/SILENT` and `/VERYSILENT` for Inno Setup

## Resources

- [Windows Package Manager Docs](https://learn.microsoft.com/en-us/windows/package-manager/)
- [winget-create GitHub](https://github.com/microsoft/winget-create)
- [winget-pkgs Repository](https://github.com/microsoft/winget-pkgs)
- [Package Submission Guidelines](https://learn.microsoft.com/en-us/windows/package-manager/package/repository)
