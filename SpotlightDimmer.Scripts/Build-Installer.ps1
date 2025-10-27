# Build-Installer.ps1
# Builds the Spotlight Dimmer installer locally (replicates GitHub Actions workflow)
#
# Usage:
#   .\Build-Installer.ps1 [-Version <version>]
#
# Example:
#   .\Build-Installer.ps1 -Version 0.9.0
#
# Prerequisites:
#   - Visual Studio C++ Build Tools (for Native AOT)
#   - Inno Setup 6 installed at C:\Program Files (x86)\Inno Setup 6
#   - .NET 10 SDK

param(
    [Parameter(Mandatory = $false)]
    [string]$Version = "0.0.0-dev"
)

$ErrorActionPreference = "Stop"

Write-Host "`n=========================================" -ForegroundColor Cyan
Write-Host "  Spotlight Dimmer Installer Build" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Version: $Version`n" -ForegroundColor Yellow

# Navigate to repository root (script should be in SpotlightDimmer.Scripts)
$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot

try {
    # Step 1: Clean previous builds
    Write-Host "==> Cleaning previous builds..." -ForegroundColor Cyan
    if (Test-Path "publish") {
        Remove-Item -Recurse -Force publish
        Write-Host "    Removed publish/" -ForegroundColor Gray
    }
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force dist
        Write-Host "    Removed dist/" -ForegroundColor Gray
    }

    # Step 2: Restore dependencies
    Write-Host "`n==> Restoring dependencies..." -ForegroundColor Cyan
    dotnet restore SpotlightDimmer.sln
    if ($LASTEXITCODE -ne 0) {
        Write-Error "dotnet restore failed"
        exit $LASTEXITCODE
    }

    # Step 3: Build SpotlightDimmer.WindowsClient with Native AOT
    Write-Host "`n==> Building SpotlightDimmer.WindowsClient (Native AOT)..." -ForegroundColor Cyan
    dotnet publish SpotlightDimmer.WindowsClient/SpotlightDimmer.WindowsClient.csproj -c Release -r win-x64 -o publish
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to build SpotlightDimmer.WindowsClient"
        exit $LASTEXITCODE
    }

    # Step 4: Build SpotlightDimmer.Config with Native AOT
    Write-Host "`n==> Building SpotlightDimmer.Config (Native AOT)..." -ForegroundColor Cyan
    dotnet publish SpotlightDimmer.Config/SpotlightDimmer.Config.csproj -c Release -r win-x64 -o publish
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to build SpotlightDimmer.Config"
        exit $LASTEXITCODE
    }

    # Step 5: Verify build outputs
    Write-Host "`n==> Verifying build outputs..." -ForegroundColor Cyan
    Write-Host "    Contents of publish/:" -ForegroundColor Gray
    Get-ChildItem -Path publish -Filter "*.exe" | ForEach-Object {
        $sizeKB = [math]::Round($_.Length / 1KB, 2)
        Write-Host "      $($_.Name) ($sizeKB KB)" -ForegroundColor Gray
    }

    if (-not (Test-Path "publish/SpotlightDimmer.exe")) {
        Write-Error "SpotlightDimmer.exe not found in publish/"
        exit 1
    }
    if (-not (Test-Path "publish/SpotlightDimmer.Config.exe")) {
        Write-Error "SpotlightDimmer.Config.exe not found in publish/"
        exit 1
    }
    Write-Host "`n    ✓ Both executables built successfully" -ForegroundColor Green

    # Step 6: Verify required icon files exist
    Write-Host "`n==> Verifying icon files..." -ForegroundColor Cyan
    $requiredFiles = @(
        "spotlight-dimmer-icon.ico",
        "spotlight-dimmer-icon-paused.ico",
        "README.md",
        "CONFIGURATION.md"
    )

    foreach ($file in $requiredFiles) {
        if (-not (Test-Path $file)) {
            Write-Error "Required file not found: $file"
            exit 1
        }
        Write-Host "    ✓ $file" -ForegroundColor Gray
    }

    # Step 7: Build the installer with Inno Setup
    Write-Host "`n==> Building Inno Setup installer..." -ForegroundColor Cyan
    $isccCommand = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($isccCommand) {
        $isccPath = $isccCommand.Source
    } else {
        $isccPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
    }

    if (-not (Test-Path $isccPath)) {
        Write-Error "Inno Setup not found at: $isccPath"
        Write-Host "`n    Please install Inno Setup 6 from:" -ForegroundColor Yellow
        Write-Host "    https://jrsoftware.org/isdl.php`n" -ForegroundColor Yellow
        exit 1
    }

    Write-Host "    Using: $isccPath" -ForegroundColor Gray
    Write-Host "    Script: spotlight-dimmer.iss" -ForegroundColor Gray
    Write-Host "    Version: $Version" -ForegroundColor Gray

    & $isccPath "/DAppVersion=$Version" spotlight-dimmer.iss

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Inno Setup compilation failed with exit code $LASTEXITCODE"
        exit $LASTEXITCODE
    }

    # Step 8: Display results
    Write-Host "`n=========================================" -ForegroundColor Green
    Write-Host "  BUILD SUCCESSFUL!" -ForegroundColor Green
    Write-Host "=========================================" -ForegroundColor Green

    Write-Host "`nInstaller created:" -ForegroundColor Cyan
    Get-ChildItem -Path dist -Filter '*.exe' | ForEach-Object {
        $sizeMB = [math]::Round($_.Length / 1MB, 2)
        Write-Host "  $($_.FullName)" -ForegroundColor Yellow
        Write-Host "  Size: $sizeMB MB`n" -ForegroundColor Gray
    }

    Write-Host "Executables built:" -ForegroundColor Cyan
    Write-Host "  $repoRoot\publish\SpotlightDimmer.exe" -ForegroundColor Yellow
    Write-Host "  $repoRoot\publish\SpotlightDimmer.Config.exe`n" -ForegroundColor Yellow

}
catch {
    Write-Host "`n=========================================" -ForegroundColor Red
    Write-Host "  BUILD FAILED!" -ForegroundColor Red
    Write-Host "=========================================" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
finally {
    Pop-Location
}
