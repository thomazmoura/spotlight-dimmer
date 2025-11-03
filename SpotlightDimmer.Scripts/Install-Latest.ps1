# Install-Latest.ps1
# Downloads and installs the latest Spotlight Dimmer release from GitHub
#
# Usage:
#   .\Install-Latest.ps1 [-Silent]
#
# Examples:
#   .\Install-Latest.ps1                    # Downloads and runs installer with UI
#   .\Install-Latest.ps1 -Silent            # Downloads and installs silently
#
# Prerequisites:
#   - Internet connection to access GitHub releases
#   - Administrator privileges (installer may require elevation)

param(
    [Parameter(Mandatory = $false)]
    [switch]$Silent
)

$ErrorActionPreference = "Stop"

Write-Host "`n==========================================" -ForegroundColor Cyan
Write-Host "  Spotlight Dimmer Latest Installer" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

$githubReleaseUrl = "https://github.com/thomazmoura/spotlight-dimmer/releases/latest/download/spotlight-dimmer-installer.exe"
$tempPath = [System.IO.Path]::GetTempPath()
$installerPath = Join-Path $tempPath "spotlight-dimmer-installer.exe"

try {
    # Step 1: Download the latest installer
    Write-Host "`n==> Downloading latest installer from GitHub..." -ForegroundColor Cyan
    Write-Host "    URL: $githubReleaseUrl" -ForegroundColor Gray
    Write-Host "    Destination: $installerPath" -ForegroundColor Gray

    # Remove old installer if it exists
    if (Test-Path $installerPath) {
        Remove-Item -Path $installerPath -Force
        Write-Host "    Removed old installer" -ForegroundColor Gray
    }

    # Download with progress
    $ProgressPreference = 'SilentlyContinue'  # Speeds up Invoke-WebRequest
    try {
        Invoke-WebRequest -Uri $githubReleaseUrl -OutFile $installerPath -UseBasicParsing
    }
    catch {
        Write-Error "Failed to download installer: $($_.Exception.Message)"
        exit 1
    }
    finally {
        $ProgressPreference = 'Continue'
    }

    # Verify download
    if (-not (Test-Path $installerPath)) {
        Write-Error "Installer download failed - file not found at $installerPath"
        exit 1
    }

    $fileSizeMB = [math]::Round((Get-Item $installerPath).Length / 1MB, 2)
    Write-Host "`n    ✓ Download complete ($fileSizeMB MB)" -ForegroundColor Green

    # Step 2: Run the installer
    Write-Host "`n==> Running installer..." -ForegroundColor Cyan

    if ($Silent) {
        Write-Host "    Mode: Silent installation" -ForegroundColor Gray
        Write-Host "    This may take a moment..." -ForegroundColor Gray

        # Run installer silently (/VERYSILENT = no UI, /SUPPRESSMSGBOXES = no message boxes)
        $process = Start-Process -FilePath $installerPath -ArgumentList "/VERYSILENT", "/SUPPRESSMSGBOXES" -Wait -PassThru

        if ($process.ExitCode -ne 0) {
            Write-Error "Installation failed with exit code: $($process.ExitCode)"
            exit $process.ExitCode
        }

        Write-Host "`n    ✓ Installation completed silently" -ForegroundColor Green
    }
    else {
        Write-Host "    Mode: Interactive installation" -ForegroundColor Gray
        Write-Host "    Please follow the installer prompts..." -ForegroundColor Gray

        # Run installer with UI
        $process = Start-Process -FilePath $installerPath -Wait -PassThru

        if ($process.ExitCode -ne 0) {
            Write-Warning "Installer exited with code: $($process.ExitCode)"
            Write-Host "    Note: User may have cancelled the installation" -ForegroundColor Yellow
        }
        else {
            Write-Host "`n    ✓ Installation completed" -ForegroundColor Green
        }
    }

    # Step 3: Cleanup
    Write-Host "`n==> Cleaning up..." -ForegroundColor Cyan
    if (Test-Path $installerPath) {
        Remove-Item -Path $installerPath -Force
        Write-Host "    Removed temporary installer" -ForegroundColor Gray
    }

    # Success message
    Write-Host "`n==========================================" -ForegroundColor Green
    Write-Host "  INSTALLATION SUCCESSFUL!" -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host "`nSpotlight Dimmer has been installed." -ForegroundColor Cyan
    Write-Host "You can now launch it from the Start Menu or installation directory.`n" -ForegroundColor Gray

}
catch {
    Write-Host "`n==========================================" -ForegroundColor Red
    Write-Host "  INSTALLATION FAILED!" -ForegroundColor Red
    Write-Host "==========================================" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray

    # Cleanup on failure
    if (Test-Path $installerPath) {
        try {
            Remove-Item -Path $installerPath -Force -ErrorAction SilentlyContinue
        }
        catch {
            # Ignore cleanup errors
        }
    }

    exit 1
}
