# Move-UnreleasedToVersion.ps1
# Moves the [Unreleased] section in CHANGELOG.md to a versioned section
#
# Usage:
#   .\Move-UnreleasedToVersion.ps1 -Version "0.8.9"
#
# This script:
# 1. Extracts all content from [Unreleased] section
# 2. Creates a new [X.Y.Z] - YYYY-MM-DD section with that content
# 3. Resets [Unreleased] to empty
# 4. Maintains proper Keep a Changelog format
#
# Prerequisites:
#   - CHANGELOG.md must exist in repository root

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

Write-Host "`n=========================================" -ForegroundColor Cyan
Write-Host "  Move Unreleased to Version" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Version: $Version`n" -ForegroundColor Yellow

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

    # Extract the [Unreleased] section using regex
    # Match from ## [Unreleased] to the next ## [ (next version section)
    $unreleasedPattern = '(?s)(## \[Unreleased\]\s*\n)(.*?)(\n## \[|$)'

    if ($changelogContent -match $unreleasedPattern) {
        $unreleasedHeader = $matches[1]
        $unreleasedContent = $matches[2].Trim()
        $nextSection = $matches[3]

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

        # Create new empty [Unreleased] section
        $newUnreleased = "## [Unreleased]`n"

        # Replace in the changelog
        # Pattern: everything before [Unreleased] + new sections + everything after
        $beforeUnreleased = $changelogContent.Substring(0, $changelogContent.IndexOf("## [Unreleased]"))

        $newChangelog = $beforeUnreleased + $newUnreleased + "`n" + $versionSection + $nextSection

        # If there was content after the unreleased section, append it
        if ($nextSection -ne '$') {
            $afterMatch = $changelogContent.Substring($changelogContent.IndexOf($nextSection))
            $newChangelog = $beforeUnreleased + $newUnreleased + "`n" + $versionSection + "`n" + $afterMatch
        }

        # Write back to file
        Set-Content -Path $changelogPath -Value $newChangelog -NoNewline

        Write-Host "=========================================" -ForegroundColor Green
        Write-Host "  SUCCESS!" -ForegroundColor Green
        Write-Host "=========================================" -ForegroundColor Green
        Write-Host "`n✓ Moved [Unreleased] to [$Version] - $date" -ForegroundColor Green
        Write-Host "✓ Created new empty [Unreleased] section" -ForegroundColor Green
        Write-Host "`nCHANGELOG.md has been updated.`n" -ForegroundColor Gray
    }
    else {
        Write-Error "Could not find [Unreleased] section in CHANGELOG.md"
        exit 1
    }
}
catch {
    Write-Host "`n=========================================" -ForegroundColor Red
    Write-Host "  OPERATION FAILED!" -ForegroundColor Red
    Write-Host "=========================================" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
