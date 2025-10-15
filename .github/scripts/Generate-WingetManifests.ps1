param(
    [Parameter(Mandatory=$true)]
    [string]$Version,

    [Parameter(Mandatory=$true)]
    [string]$InstallerUrl,

    [Parameter(Mandatory=$true)]
    [string]$InstallerSha256
)

# Create manifest directory
$manifestDir = "winget-manifests/t/ThomazMoura/SpotlightDimmer/$Version"
New-Item -ItemType Directory -Path $manifestDir -Force | Out-Null

# Version manifest
$versionManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json

PackageIdentifier: ThomazMoura.SpotlightDimmer
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
"@
Set-Content -Path "$manifestDir/ThomazMoura.SpotlightDimmer.yaml" -Value $versionManifest -Encoding UTF8

# Installer manifest
$installerManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json

PackageIdentifier: ThomazMoura.SpotlightDimmer
PackageVersion: $Version
Platform:
  - Windows.Desktop
MinimumOSVersion: 10.0.0.0
InstallerType: nullsoft
Scope: user
InstallModes:
  - interactive
  - silent
UpgradeBehavior: install
Installers:
  - Architecture: x64
    InstallerUrl: $InstallerUrl
    InstallerSha256: $InstallerSha256
ManifestType: installer
ManifestVersion: 1.6.0
"@
Set-Content -Path "$manifestDir/ThomazMoura.SpotlightDimmer.installer.yaml" -Value $installerManifest -Encoding UTF8

# Locale manifest
$localeManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json

PackageIdentifier: ThomazMoura.SpotlightDimmer
PackageVersion: $Version
PackageLocale: en-US
Publisher: Thomaz Moura
PublisherUrl: https://github.com/thomazmoura
PublisherSupportUrl: https://github.com/thomazmoura/spotlight-dimmer/issues
PackageName: Spotlight Dimmer
PackageUrl: https://github.com/thomazmoura/spotlight-dimmer
License: MIT
LicenseUrl: https://github.com/thomazmoura/spotlight-dimmer/blob/main/LICENSE
ShortDescription: A lightweight Windows application that dims inactive displays to highlight the active one
Description: |-
  Spotlight Dimmer helps you focus by dimming all displays except the one with your currently active window.

  Features:
  • Ultra-lightweight: Only ~7.6 MB RAM usage
  • Real-time Monitoring: Instantly detects window focus changes
  • Click-through Overlays: Dimming doesn't interfere with your workflow
  • Native Windows API: No browser engine overhead, instant startup
  • Auto-reload Config: Changes detected instantly via file system notifications

  Perfect for multi-monitor setups!
Moniker: spotlight-dimmer
Tags:
  - desktop
  - display
  - dimming
  - focus
  - multi-monitor
  - productivity
  - windows
  - rust
  - native
ManifestType: defaultLocale
ManifestVersion: 1.6.0
"@
Set-Content -Path "$manifestDir/ThomazMoura.SpotlightDimmer.locale.en-US.yaml" -Value $localeManifest -Encoding UTF8

Write-Host "✅ Manifests generated in $manifestDir"
Get-ChildItem -Path $manifestDir | Format-Table Name, Length
