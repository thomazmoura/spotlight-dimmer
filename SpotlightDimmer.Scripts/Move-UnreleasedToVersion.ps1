# Move-UnreleasedToVersion.ps1
# Moves the [Unreleased] section in a changelog to a versioned section
#
# Usage:
#   .\Move-UnreleasedToVersion.ps1 -Version "0.8.9"
#   .\Move-UnreleasedToVersion.ps1 -Version "0.2.0" -ChangelogPath "CHANGELOG.linux.md"
#
# This script:
# 1. Extracts all content from [Unreleased] section
# 2. Creates a new [X.Y.Z] - YYYY-MM-DD section with that content
# 3. Resets [Unreleased] to empty
# 4. Maintains proper Keep a Changelog format
#
# Prerequisites:
#   - The changelog file must exist (relative paths resolve against the repository root)

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,

    [string]$ChangelogPath = "CHANGELOG.md"
)

$ErrorActionPreference = "Stop"

Write-Host "`n=========================================" -ForegroundColor Cyan
Write-Host "  Move Unreleased to Version" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Version: $Version`n" -ForegroundColor Yellow

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

    # Locate the [Unreleased] section by index (regex with lazy matching
    # mis-captures the next version section when [Unreleased] is empty)
    $unreleasedIndex = $changelogContent.IndexOf("## [Unreleased]")
    if ($unreleasedIndex -lt 0) {
        Write-Error "Could not find [Unreleased] section in $changelogPath"
        exit 1
    }

    # The section content runs until the next "## [" header (or end of file)
    $contentStart = $unreleasedIndex + "## [Unreleased]".Length
    $nextSectionIndex = $changelogContent.IndexOf("`n## [", $contentStart)

    $unreleasedContent = if ($nextSectionIndex -ge 0) {
        $changelogContent.Substring($contentStart, $nextSectionIndex - $contentStart).Trim()
    } else {
        $changelogContent.Substring($contentStart).Trim()
    }

    # Check if there's actual content
    if ([string]::IsNullOrWhiteSpace($unreleasedContent)) {
        Write-Warning "The [Unreleased] section is empty. No changes to move."
        exit 0
    }

    # Get today's date in ISO format
    $date = Get-Date -Format "yyyy-MM-dd"

    # Create the new version section
    $versionSection = @"
## [$Version] - $date

$unreleasedContent
"@

    # Rebuild the changelog: everything before [Unreleased], a fresh empty
    # [Unreleased], the new version section, then everything from the next
    # version header onward
    $beforeUnreleased = $changelogContent.Substring(0, $unreleasedIndex)
    $afterUnreleased = if ($nextSectionIndex -ge 0) {
        $changelogContent.Substring($nextSectionIndex)
    } else {
        ""
    }

    $newChangelog = $beforeUnreleased + "## [Unreleased]`n`n" + $versionSection + "`n" + $afterUnreleased

    # Write back to file
    Set-Content -Path $changelogPath -Value $newChangelog -NoNewline

    Write-Host "=========================================" -ForegroundColor Green
    Write-Host "  SUCCESS!" -ForegroundColor Green
    Write-Host "=========================================" -ForegroundColor Green
    Write-Host "`n✓ Moved [Unreleased] to [$Version] - $date" -ForegroundColor Green
    Write-Host "✓ Created new empty [Unreleased] section" -ForegroundColor Green
    Write-Host "`n$changelogPath has been updated.`n" -ForegroundColor Gray
}
catch {
    Write-Host "`n=========================================" -ForegroundColor Red
    Write-Host "  OPERATION FAILED!" -ForegroundColor Red
    Write-Host "=========================================" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
