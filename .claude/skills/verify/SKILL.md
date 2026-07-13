---
name: verify
description: Build, launch, and drive the SpotlightDimmer Windows apps to verify changes end-to-end.
---

# Verifying SpotlightDimmer changes

## Config GUI (SpotlightDimmer.Config)

Build and locate the exe:

```powershell
dotnet build SpotlightDimmer.Config\SpotlightDimmer.Config.csproj
# exe: SpotlightDimmer.Config\bin\Debug\net10.0-windows\win-x64\SpotlightDimmer.Config.exe
```

Drive it with UI Automation from PowerShell (no test framework needed):

- `Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes`, find the
  window by `ProcessIdProperty` under `RootElement` (title: "SpotlightDimmer
  Configuration").
- Tabs/buttons: find by `NameProperty`, use `SelectionItemPattern` (tabs) and
  `InvokePattern` (buttons). TextBoxes: `ValuePattern.SetValue` then move
  focus elsewhere (`SetFocus` on another control) to fire `Leave` handlers.
  ComboBoxes: `ExpandCollapsePattern` + `SelectionItemPattern` on the item.
  NumericUpDowns surface as `Spinner` controls; set their inner `Edit` child.
- Screenshot via `System.Drawing` `CopyFromScreen` of the element's
  `BoundingRectangle`.

Observable output: `%AppData%\SpotlightDimmer\config.json` — dump
`AppIntegrations`/`Overlay` after each interaction to confirm saves.

**Gotchas:**
- **Back up `%AppData%\SpotlightDimmer\config.json` first and restore after** —
  the GUI saves on every change, and the user has real settings there.
- Every save triggers the app's own FileSystemWatcher reload (~1s); sleep
  ≥1s after each interaction before reading the file.
- External edits to config.json while the app runs test the hot-reload path;
  the GUI should re-populate without losing ListBox selection.
- Machine quirks: empty NuGet sources (`dotnet restore` needs `-s nuget.org`);
  no MSVC linker, so skip AOT publish — Debug build is fine for verification.

## Main app (SpotlightDimmer.WindowsClient)

`dotnet run --project SpotlightDimmer.WindowsClient` (add `-- --verbose` for
GDI/focus event logging). It creates overlay windows; verify behavior by
moving focus between windows and reading the verbose log output.
