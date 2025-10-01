# Spotlight Dimmer Uninstallation Script
# This script removes both binaries and icon files

Write-Host "Uninstalling Spotlight Dimmer..." -ForegroundColor Cyan

# Get the cargo bin directory
$cargoBin = "$env:USERPROFILE\.cargo\bin"

# Stop any running instances
Write-Host "`nStopping any running instances..." -ForegroundColor Yellow
Get-Process spotlight-dimmer -ErrorAction SilentlyContinue | Stop-Process -Force
if ($?) {
    Write-Host "  Stopped running instances" -ForegroundColor Green
} else {
    Write-Host "  No running instances found" -ForegroundColor Gray
}

# Uninstall binaries
Write-Host "`nUninstalling binaries..." -ForegroundColor Yellow
cargo uninstall spotlight-dimmer

if ($LASTEXITCODE -ne 0) {
    Write-Host "Warning: Failed to uninstall binaries (may not be installed)" -ForegroundColor Yellow
}

# Remove icon files
Write-Host "`nRemoving icon files from $cargoBin..." -ForegroundColor Yellow

$icons = @("spotlight-dimmer-icon.ico", "spotlight-dimmer-icon-paused.ico")
foreach ($icon in $icons) {
    $iconPath = Join-Path $cargoBin $icon
    if (Test-Path $iconPath) {
        Remove-Item $iconPath -Force
        Write-Host "  Removed $icon" -ForegroundColor Green
    } else {
        Write-Host "  $icon not found (already removed?)" -ForegroundColor Gray
    }
}

Write-Host "`nUninstallation complete!" -ForegroundColor Green
