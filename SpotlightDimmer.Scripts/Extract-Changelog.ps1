# Extract-Changelog.ps1
# Extracts the [Unreleased] section from CHANGELOG.md for use in release notes
#
# Usage:
#   .\Extract-Changelog.ps1
#
# This script reads CHANGELOG.md and extracts everything in the [Unreleased] section,
# which includes all changes since the last release. This content is used to generate
# comprehensive GitHub release descriptions.
#
# Output:
#   The [Unreleased] section content (without the ## [Unreleased] header)
#   Returns empty string if [Unreleased] section is empty

param()

$ErrorActionPreference = "Stop"

# Navigate to repository root (script should be in SpotlightDimmer.Scripts)
$repoRoot = Split-Path -Parent $PSScriptRoot
$changelogPath = Join-Path $repoRoot "CHANGELOG.md"

if (-not (Test-Path $changelogPath)) {
    Write-Error "CHANGELOG.md not found at: $changelogPath"
    exit 1
}

try {
    # Read the entire changelog
    $changelogContent = Get-Content -Path $changelogPath -Raw

    # Find the [Unreleased] section using regex
    # Match from ## [Unreleased] to the next ## [ (next version section)
    $unreleasedPattern = '(?s)## \[Unreleased\]\s*\n(.*?)(?=\n## \[|$)'

    if ($changelogContent -match $unreleasedPattern) {
        $unreleasedContent = $matches[1].Trim()

        # Check if there's actual content (not just whitespace)
        if ([string]::IsNullOrWhiteSpace($unreleasedContent)) {
            # Return empty string if no content
            Write-Output ""
        }
        else {
            # Return the extracted content
            Write-Output $unreleasedContent
        }
    }
    else {
        Write-Error "Could not find [Unreleased] section in CHANGELOG.md"
        exit 1
    }
}
catch {
    Write-Error "Failed to extract changelog: $($_.Exception.Message)"
    exit 1
}
