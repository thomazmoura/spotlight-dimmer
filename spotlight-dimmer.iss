; Inno Setup Script for Spotlight Dimmer
; This creates a compact, per-user installer that requires no admin rights

#define AppName "Spotlight Dimmer"
#define AppPublisher "Thomaz Moura"
#define AppURL "https://github.com/thomazmoura/spotlight-dimmer"
#define AppExeName "SpotlightDimmer.exe"
#define AppConfigExeName "SpotlightDimmer.Config.exe"

; Version will be set by the build script via /DAppVersion=x.x.x
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
; Basic app info
AppId={{8E5D7A9C-2F3E-4B1C-9D8A-6F4E2C1B5A3D}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}/issues
AppUpdatesURL={#AppURL}/releases
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir=dist
OutputBaseFilename=spotlight-dimmer-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
SetupIconFile=spotlight-dimmer-icon.ico
UninstallDisplayIcon={app}\{#AppExeName}

; Per-user installation (no admin rights required)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; Architecture
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
; Main executables
Source: "publish\SpotlightDimmer.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "publish\SpotlightDimmer.Config.exe"; DestDir: "{app}"; Flags: ignoreversion

; Icon files (required at runtime)
Source: "spotlight-dimmer-icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "spotlight-dimmer-icon-paused.ico"; DestDir: "{app}"; Flags: ignoreversion

; Documentation
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "CONFIGURATION.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; Start Menu entries with proper icons
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}"; IconFilename: "{app}\spotlight-dimmer-icon.ico"
Name: "{autoprograms}\{#AppName} Config"; Filename: "{app}\{#AppConfigExeName}"; IconFilename: "{app}\spotlight-dimmer-icon-paused.ico"

[Run]
; Option to launch the app after installation
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Clean up config directory on uninstall (optional - user may want to keep settings)
; Type: filesandordirs; Name: "{userappdata}\SpotlightDimmer"

[Code]
// Custom code can be added here if needed for additional logic
