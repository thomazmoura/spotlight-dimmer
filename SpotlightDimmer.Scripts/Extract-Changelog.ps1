# Extract-Changelog.ps1
# Extracts a section from a changelog for use in release notes
#
# Usage:
#   .\Extract-Changelog.ps1                                          # [Unreleased] from CHANGELOG.md
#   .\Extract-Changelog.ps1 -Version "0.8.13"                        # [0.8.13] section
#   .\Extract-Changelog.ps1 -Version "0.1.0" -ChangelogPath "CHANGELOG.linux.md"
#
# By default this extracts the [Unreleased] section. Release workflows should
# pass -Version instead: the publish commands move [Unreleased] into [X.Y.Z]
# BEFORE the tag is pushed, so by the time the release workflow runs,
# [Unreleased] is already empty and the released changes live in the [X.Y.Z]
# section.
#
# Output:
#   The section content (without the ## [...] header)
#   Returns empty string if the section is empty

param(
    [string]$ChangelogPath = "CHANGELOG.md",

    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

# Navigate to repository root (script should be in SpotlightDimmer.Scripts)
$repoRoot = Split-Path -Parent $PSScriptRoot
$changelogPath = if ([System.IO.Path]::IsPathRooted($ChangelogPath)) {
    $ChangelogPath
} else {
    Join-Path $repoRoot $ChangelogPath
}

if (-not (Test-Path $changelogPath)) {
    Write-Error "Changelog not found at: $changelogPath"
    exit 1
}

try {
    # Read the entire changelog
    $changelogContent = Get-Content -Path $changelogPath -Raw

    # Determine which section header to look for. Version sections are
    # "## [X.Y.Z] - YYYY-MM-DD", so match on "## [X.Y.Z]" only.
    $sectionHeader = if ([string]::IsNullOrWhiteSpace($Version)) {
        "## [Unreleased]"
    } else {
        "## [$Version]"
    }

    # Locate the section by index (regex with lazy matching mis-captures the
    # next version section when the target section is empty)
    $sectionIndex = $changelogContent.IndexOf($sectionHeader)
    if ($sectionIndex -lt 0) {
        Write-Error "Could not find $sectionHeader section in $changelogPath"
        exit 1
    }

    # Content starts after the header line and runs until the next "## ["
    # header (or end of file)
    $headerLineEnd = $changelogContent.IndexOf("`n", $sectionIndex)
    if ($headerLineEnd -lt 0) {
        # Header is the last line of the file - no content
        Write-Output ""
        exit 0
    }

    $nextSectionIndex = $changelogContent.IndexOf("`n## [", $headerLineEnd)

    $sectionContent = if ($nextSectionIndex -ge 0) {
        $changelogContent.Substring($headerLineEnd, $nextSectionIndex - $headerLineEnd).Trim()
    } else {
        $changelogContent.Substring($headerLineEnd).Trim()
    }

    # Check if there's actual content (not just whitespace)
    if ([string]::IsNullOrWhiteSpace($sectionContent)) {
        # Return empty string if no content
        Write-Output ""
    }
    else {
        # Return the extracted content
        Write-Output $sectionContent
    }
}
catch {
    Write-Error "Failed to extract changelog: $($_.Exception.Message)"
    exit 1
}
