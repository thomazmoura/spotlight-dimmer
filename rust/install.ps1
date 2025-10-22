# Spotlight Dimmer Installation Script
# This script installs both binaries and icon files

Write-Host "Installing Spotlight Dimmer..." -ForegroundColor Cyan

# Install binaries
Write-Host "`nInstalling binaries to ~/.cargo/bin/..." -ForegroundColor Yellow
cargo install --path . --bin spotlight-dimmer --bin spotlight-dimmer-config

if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to install binaries!" -ForegroundColor Red
    exit 1
}

# Get the cargo bin directory
$cargoBin = "$env:USERPROFILE\.cargo\bin"

# Copy icon files
Write-Host "`nCopying icon files to $cargoBin..." -ForegroundColor Yellow

$icons = @("spotlight-dimmer-icon.ico", "spotlight-dimmer-icon-paused.ico")
foreach ($icon in $icons) {
    if (Test-Path $icon) {
        Copy-Item $icon $cargoBin -Force
        Write-Host "  Copied $icon" -ForegroundColor Green
    } else {
        Write-Host "  Warning: $icon not found in current directory" -ForegroundColor Red
    }
}

Write-Host "`nInstallation complete!" -ForegroundColor Green
Write-Host "Run 'spotlight-dimmer' to start the application" -ForegroundColor Cyan
