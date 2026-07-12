# Install-WslTmuxIntegration.ps1
# Sets up the tmux-in-WSL pane spotlight integration inside a WSL distro:
#   1. Copies spotlight-dimmer-tmux-report.sh and spotlight-dimmer.tmux.conf
#      into ~/.config/SpotlightDimmer/tools/ (normalizing line endings and
#      making the script executable)
#   2. Records the SpotlightDimmer.PaneReport.exe location in
#      ~/.cache/spotlight-dimmer/report-exe so the report script can find it
#      even when Spotlight Dimmer is not installed in a default location
#      (e.g. development builds run from the repository)
#   3. Adds a source-file line for the hooks to ~/.tmux.conf (idempotent -
#      re-running never duplicates the line)
#   4. Smoke-tests the forwarder through WSL's Windows interop
#
# Usage:
#   .\Install-WslTmuxIntegration.ps1 [-Distribution <name>] [-PaneReportExePath <path>] [-ToolsSourceDir <path>] [-SkipTmuxConf]
#
# Examples:
#   .\Install-WslTmuxIntegration.ps1                          # Default distro, auto-detect paths
#   .\Install-WslTmuxIntegration.ps1 -Distribution Ubuntu     # Specific distro
#   .\Install-WslTmuxIntegration.ps1 -SkipTmuxConf            # Don't touch ~/.tmux.conf
#
# Prerequisites:
#   - WSL 2 with a Linux distro and tmux >= 3.0 installed in it
#   - Windows interop enabled in the distro (default; see /etc/wsl.conf)
#   - SpotlightDimmer.PaneReport.exe built or installed (see error hints)
#
# See docs/WINDOWS_TERMINAL_INTEGRATION.md for how the integration works.

param(
    [Parameter(Mandatory = $false)]
    [string]$Distribution,

    [Parameter(Mandatory = $false)]
    [string]$PaneReportExePath,

    [Parameter(Mandatory = $false)]
    [string]$ToolsSourceDir,

    [Parameter(Mandatory = $false)]
    [switch]$SkipTmuxConf
)

$ErrorActionPreference = "Stop"

# wsl.exe emits command output as UTF-8; make sure PowerShell decodes it that
# way regardless of the console code page (restored in the finally block).
$originalOutputEncoding = [Console]::OutputEncoding

# Runs a bash command line inside the target distro and returns its output.
# Throws on non-zero exit unless -AllowFailure is set (then $LASTEXITCODE
# carries the exit code for the caller to inspect).
function Invoke-WslBash {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Command,
        [switch]$AllowFailure
    )

    $wslArgs = @()
    if ($Distribution) { $wslArgs += @("-d", $Distribution) }
    $wslArgs += @("-e", "bash", "-c", $Command)

    $output = & wsl.exe @wslArgs 2>&1
    if ($LASTEXITCODE -ne 0 -and -not $AllowFailure) {
        throw "WSL command failed (exit $LASTEXITCODE): $Command`n$output"
    }
    return $output
}

# Converts a Windows path to its /mnt/... form as seen by the target distro.
# wslpath honors custom automount roots, so this works on non-default mounts.
function ConvertTo-WslPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$WindowsPath
    )

    $wslArgs = @()
    if ($Distribution) { $wslArgs += @("-d", $Distribution) }
    $wslArgs += @("-e", "wslpath", "-u", $WindowsPath)

    $result = (& wsl.exe @wslArgs) -join ""
    if ($LASTEXITCODE -ne 0 -or -not $result) {
        throw "Failed to convert path to WSL form: $WindowsPath"
    }
    return $result.Trim()
}

Write-Host "`n==========================================" -ForegroundColor Cyan
Write-Host "  Spotlight Dimmer WSL tmux Integration" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8

    # Step 1: Locate SpotlightDimmer.PaneReport.exe and the tool files.
    # The script runs from two layouts: an installed copy in {app}\tools
    # (parent dir holds the exe) or the repository's SpotlightDimmer.Scripts
    # (sibling projects hold build outputs).
    Write-Host "`n==> Locating Spotlight Dimmer files..." -ForegroundColor Cyan

    $parentDir = Split-Path $PSScriptRoot

    if (-not $PaneReportExePath) {
        $exeCandidates = @(
            (Join-Path $parentDir "SpotlightDimmer.PaneReport.exe"),
            (Join-Path $env:LOCALAPPDATA "Programs\Spotlight Dimmer\SpotlightDimmer.PaneReport.exe"),
            "C:\Program Files\Spotlight Dimmer\SpotlightDimmer.PaneReport.exe",
            (Join-Path $parentDir "SpotlightDimmer.PaneReport\bin\Release\net10.0\win-x64\publish\SpotlightDimmer.PaneReport.exe"),
            (Join-Path $parentDir "SpotlightDimmer.PaneReport\bin\Debug\net10.0\win-x64\SpotlightDimmer.PaneReport.exe")
        )
        $PaneReportExePath = $exeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

        if (-not $PaneReportExePath) {
            Write-Error ("SpotlightDimmer.PaneReport.exe not found. Install Spotlight Dimmer, or build it from the repository with:`n" +
                "    dotnet build SpotlightDimmer.PaneReport -c Debug`n" +
                "or pass the location explicitly with -PaneReportExePath.")
            exit 1
        }
    }
    elseif (-not (Test-Path $PaneReportExePath)) {
        Write-Error "PaneReport exe not found at: $PaneReportExePath"
        exit 1
    }

    $reportScriptName = "spotlight-dimmer-tmux-report.sh"
    $tmuxConfName = "spotlight-dimmer.tmux.conf"

    if (-not $ToolsSourceDir) {
        $toolsCandidates = @(
            $PSScriptRoot,
            (Join-Path $env:LOCALAPPDATA "Programs\Spotlight Dimmer\tools"),
            "C:\Program Files\Spotlight Dimmer\tools",
            (Join-Path $parentDir "SpotlightDimmer.WindowsClient\tools")
        )
        $ToolsSourceDir = $toolsCandidates | Where-Object {
            (Test-Path (Join-Path $_ $reportScriptName)) -and (Test-Path (Join-Path $_ $tmuxConfName))
        } | Select-Object -First 1

        if (-not $ToolsSourceDir) {
            Write-Error "Could not find $reportScriptName and $tmuxConfName. Pass their directory with -ToolsSourceDir."
            exit 1
        }
    }
    elseif (-not ((Test-Path (Join-Path $ToolsSourceDir $reportScriptName)) -and (Test-Path (Join-Path $ToolsSourceDir $tmuxConfName)))) {
        Write-Error "Directory does not contain $reportScriptName and ${tmuxConfName}: $ToolsSourceDir"
        exit 1
    }

    Write-Host "    PaneReport exe: $PaneReportExePath" -ForegroundColor Gray
    Write-Host "    Tools source:   $ToolsSourceDir" -ForegroundColor Gray

    # Step 2: Verify the distro boots and interop-relevant tools exist.
    Write-Host "`n==> Checking WSL distro..." -ForegroundColor Cyan

    $null = Invoke-WslBash "echo ok"
    $distroLabel = if ($Distribution) { $Distribution } else { "(default distro)" }
    Write-Host "    Distro reachable: $distroLabel" -ForegroundColor Gray

    $tmuxVersion = (Invoke-WslBash "command -v tmux >/dev/null && tmux -V || echo missing") -join ""
    if ($tmuxVersion -eq "missing") {
        Write-Warning "tmux is not installed in the distro. Install it (e.g. 'sudo apt install tmux') - the hooks need tmux >= 3.0."
    }
    else {
        Write-Host "    tmux: $tmuxVersion" -ForegroundColor Gray
    }

    # Step 3: Copy the tool files into the distro. Line endings are stripped
    # defensively (the repo forces LF via .gitattributes, but copies may pass
    # through tools that rewrite them) and the report script made executable.
    Write-Host "`n==> Installing tools into ~/.config/SpotlightDimmer/tools/..." -ForegroundColor Cyan

    $toolsWsl = ConvertTo-WslPath $ToolsSourceDir
    $installCmd = ('set -eu; ' +
        'mkdir -p ~/.config/SpotlightDimmer/tools ~/.cache/spotlight-dimmer; ' +
        'cp "{0}/spotlight-dimmer-tmux-report.sh" "{0}/spotlight-dimmer.tmux.conf" ~/.config/SpotlightDimmer/tools/; ' +
        'sed -i "s/\r$//" ~/.config/SpotlightDimmer/tools/spotlight-dimmer-tmux-report.sh ~/.config/SpotlightDimmer/tools/spotlight-dimmer.tmux.conf; ' +
        'chmod +x ~/.config/SpotlightDimmer/tools/spotlight-dimmer-tmux-report.sh') -f $toolsWsl
    $null = Invoke-WslBash $installCmd
    Write-Host "    Copied $reportScriptName and $tmuxConfName" -ForegroundColor Gray

    # Step 4: Record the exe location for the report script. The cache file is
    # the script's documented lookup mechanism after `$SPOTLIGHT_DIMMER_REPORT_EXE,
    # and the only one that works for non-default installs and repo builds.
    Write-Host "`n==> Recording PaneReport exe location..." -ForegroundColor Cyan

    $exeWsl = ConvertTo-WslPath $PaneReportExePath
    $null = Invoke-WslBash ('printf ''%s'' "{0}" > ~/.cache/spotlight-dimmer/report-exe' -f $exeWsl)
    Write-Host "    ~/.cache/spotlight-dimmer/report-exe -> $exeWsl" -ForegroundColor Gray

    # Step 5: Hook the tmux configuration (idempotent).
    if ($SkipTmuxConf) {
        Write-Host "`n==> Skipping ~/.tmux.conf changes (-SkipTmuxConf)" -ForegroundColor Cyan
    }
    else {
        Write-Host "`n==> Wiring hooks into ~/.tmux.conf..." -ForegroundColor Cyan

        $alreadyHooked = (Invoke-WslBash 'grep -qsF "spotlight-dimmer.tmux.conf" ~/.tmux.conf && echo yes || echo no') -join ""
        if ($alreadyHooked -eq "yes") {
            Write-Host "    source-file line already present - nothing to do" -ForegroundColor Gray
        }
        else {
            $null = Invoke-WslBash ('printf ''\n# Spotlight Dimmer pane spotlight\nsource-file ~/.config/SpotlightDimmer/tools/spotlight-dimmer.tmux.conf\n'' >> ~/.tmux.conf')
            Write-Host "    Added source-file line to ~/.tmux.conf" -ForegroundColor Gray
        }

        # Reload a running tmux server so the hooks apply without a restart.
        $null = Invoke-WslBash 'tmux source-file ~/.tmux.conf 2>/dev/null || true' -AllowFailure
    }

    # Step 6: Smoke-test the forwarder through Windows interop. The exe exits 0
    # by contract even when Spotlight Dimmer is not running, so a non-zero exit
    # here means interop itself is broken (or the exe path is wrong).
    Write-Host "`n==> Smoke-testing the forwarder via WSL interop..." -ForegroundColor Cyan

    $smokeOutput = Invoke-WslBash 'exe="$(cat ~/.cache/spotlight-dimmer/report-exe)"; "$exe" "v1|clear|tty=/dev/pts/none"' -AllowFailure
    if ($LASTEXITCODE -eq 0) {
        Write-Host "    Forwarder launched successfully" -ForegroundColor Gray
    }
    else {
        Write-Warning ("The forwarder could not be launched from WSL (exit $LASTEXITCODE): $smokeOutput`n" +
            "Windows interop may be disabled in the distro - check that /etc/wsl.conf does not set [interop] enabled=false.")
    }

    Write-Host "`n==========================================" -ForegroundColor Green
    Write-Host "  WSL TMUX INTEGRATION INSTALLED!" -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host "`nNext steps:" -ForegroundColor Cyan
    Write-Host "  1. Make sure Spotlight Dimmer is running with an AppIntegrations entry" -ForegroundColor Gray
    Write-Host "     for your terminal (see docs/WINDOWS_TERMINAL_INTEGRATION.md)." -ForegroundColor Gray
    Write-Host "  2. Open your terminal -> WSL -> tmux and split some panes." -ForegroundColor Gray
    Write-Host "     The spotlight should shrink to the focused pane.`n" -ForegroundColor Gray
}
catch {
    Write-Host "`n==========================================" -ForegroundColor Red
    Write-Host "  INSTALLATION FAILED!" -ForegroundColor Red
    Write-Host "==========================================" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
finally {
    [Console]::OutputEncoding = $originalOutputEncoding
}
