# Agent Instructions for SpotlightDimmer

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Changelog Management (REQUIRED)

**CRITICAL**: Every code change, feature addition, bug fix, or improvement MUST be documented in `CHANGELOG.md`. This is not optional.

### When to Update the Changelog

Update `CHANGELOG.md` for ANY of these changes:
- ✅ New features or functionality
- ✅ Bug fixes and issue resolutions
- ✅ Breaking changes or API modifications
- ✅ Performance improvements
- ✅ UI/UX enhancements
- ✅ Configuration changes
- ✅ Dependency updates (if user-facing)
- ✅ Security fixes
- ✅ Documentation improvements (if significant)

### Changelog Format

Follow [Keep a Changelog](https://keepachangelog.com/) format. Always add entries under the `## [Unreleased]` section:

#### For New Features
```markdown
### Added
- Feature name: Clear description of what it does and why it's useful for users
- Another feature: Focus on user-facing benefits, not internal implementation details
```

#### For Bug Fixes
```markdown
### Fixed
- Issue description: What was broken and how it affects users
- Bug name: Clear explanation of the fix and its impact
```

#### For Breaking Changes
```markdown
### Changed
- Breaking change description: What changed and why
- Migration steps: If users need to take action, explain how
```

#### For Performance/Internal Improvements
```markdown
### Improved
- Performance enhancement: Measurable impact on user experience
- Internal optimization: Only if it affects user-visible behavior
```

#### For Removed Features
```markdown
### Removed
- Deprecated feature: What was removed and why
- Alternative solution: What users should use instead
```

### Portuguese Translation Requirement (MANDATORY)

**CRITICAL**: Every changelog entry MUST include a Portuguese translation. This is required for all entries without exception.

#### Format Structure
Each changelog entry must follow this bilingual format:
```markdown
### Added
- Feature name: Clear description of what it does and why it's useful for users
- Another feature: Focus on user-facing benefits, not internal implementation details

---

### Adicionado
- Nome da funcionalidade: Descrição clara do que faz e por que é útil para os usuários
- Outra funcionalidade: Foque nos benefícios voltados ao usuário, não em detalhes de implementação interna
```

#### Translation Guidelines
1. **Maintain Technical Accuracy**: Ensure technical terms are correctly translated or kept in English when appropriate
2. **User-Friendly Language**: Use Portuguese that Brazilian and Portuguese users can easily understand
3. **Consistent Terminology**: Keep consistent translations for recurring technical terms
4. **Section Headers**: Always translate section headers (Added→Adicionado, Fixed→Corrigido, Changed→Alterado, etc.)

#### Section Header Translations
- **Added** → **Adicionado**
- **Fixed** → **Corrigido**
- **Changed** → **Alterado**
- **Improved** → **Melhorado**
- **Removed** → **Removido**
- **Security** → **Segurança**
- **Deprecated** → **Obsoleto**

### Changelog Writing Guidelines

1. **User-Focused**: Write for end users, not developers
2. **Clear Impact**: Explain what changed and why it matters
3. **Actionable**: Include migration steps for breaking changes
4. **Specific**: Use concrete examples rather than vague descriptions
5. **Consistent**: Follow the same style and format for all entries
6. **Bilingual**: Always include Portuguese translations using the format above

### Example Entry
```markdown
### Added
- Dark mode support: Users can now switch between light and dark themes via configuration
- Keyboard shortcuts: Added Ctrl+D to toggle dimming and Ctrl+Q to quit application
- Multi-monitor performance: Reduced CPU usage by 40% when managing 3+ displays through event-driven architecture

### Fixed
- Display detection bug: Application now properly detects displays after sleep/wake cycles
- Memory leak: Fixed DeferWindowPos handle leak causing memory growth during window dragging

### Changed
- Overlay transparency: Changed default dimming from 30% to 60% for better visibility (users can adjust in config.json)

---

### Adicionado
- Suporte a modo escuro: Os usuários agora podem alternar entre temas claro e escuro através da configuração
- Atalhos de teclado: Adicionado Ctrl+D para alternar o escurecimento e Ctrl+Q para sair da aplicação
- Performance multi-monitor: Redução de 40% no uso de CPU ao gerenciar 3+ displays através de arquitetura orientada a eventos

### Corrigido
- Bug de detecção de display: A aplicação agora detecta adequadamente displays após ciclos de suspensão/despertar
- Vazamento de memória: Corrigido vazamento de handle DeferWindowPos causando crescimento de memória durante arrasto de janelas

### Alterado
- Transparência de sobreposição: Alterada transparência padrão de 30% para 60% para melhor visibilidade (usuários podem ajustar em config.json)
```

### Release Process Integration

The changelog directly feeds into GitHub Releases:
- Release notes are automatically generated from the `[Unreleased]` section
- This ensures comprehensive, professional release documentation
- No manual release note writing required

### Quality Standards

Each changelog entry should answer:
- **What** changed?
- **Why** did it change?
- **How** does it affect users?
- **What** should users do (if action required)?
- **Is the Portuguese translation accurate and user-friendly?**

**Enforcement**: Changes without proper changelog updates (including Portuguese translations) will be considered incomplete.

## Version Management

Version management is centralized in `Directory.Build.props` which is automatically imported by all .NET projects in the repository.

### Version File Location
- **File**: `Directory.Build.props`
- **Properties to update**:
  - `<Version>` - Full version including pre-release suffix (e.g., 0.8.0-beta)
  - `<AssemblyVersion>` - Major.Minor.Patch only (e.g., 0.8.0)
  - `<FileVersion>` - Major.Minor.Patch only (e.g., 0.8.0)
  - `<InformationalVersion>` - Full version including pre-release suffix (e.g., 0.8.0-beta)

### Automated Release Commands

Use the slash commands for version bumps and releases:

- `/publish-patch` - Increment patch version (0.8.0 → 0.8.1) for bug fixes
- `/publish-minor` - Increment minor version (0.8.1 → 0.9.0) for new features

These commands:
1. Update version in `Directory.Build.props`
2. Run validation (build + tests)
3. Create git commit and tag
4. Push to repository

See `.claude/commands/publish-patch.md` and `.claude/commands/publish-minor.md` for detailed documentation.

## Git Commit Messages

**IMPORTANT**: Do NOT include any references to Claude, Claude Code, AI tools, or co-authorship attributions in git commit messages.

- Write clear, descriptive commit messages focused on the changes made
- Use conventional commit format when appropriate (e.g., "Fix:", "Add:", "Update:")
- Keep commit messages professional and technical
- No footer lines like "Generated with Claude Code" or "Co-Authored-By: Claude"

### Example Commit Message Format:
```
Fix DeferWindowPos handle leak during window dragging

Implemented proper handle cleanup to prevent memory growth during drag operations.

- Clean up last valid handle on DeferWindowPos failure
- Add verbose GDI object monitoring
- Document handle leak prevention pattern
```

## Project Overview

SpotlightDimmer is a Windows utility that creates semi-transparent overlays to dim inactive displays or regions, creating a "spotlight" effect on the active window. This is a .NET 10 implementation that replaces the previous Rust version, leveraging native Windows event hooks for zero-polling, event-driven architecture.

**Key Features:**
- Multi-monitor support with event-driven focus tracking
- Three dimming modes: FullScreen, Partial, and PartialWithActive
- Hot-reloadable JSON configuration at `%AppData%\SpotlightDimmer\config.json`
- Native AOT compilation support for fast startup and small binaries
- Zero-allocation hot path design for minimal GC pressure

## Build & Run Commands

### Development
```bash
# Build the project
dotnet build

# Run with standard logging
dotnet run

# Run with verbose GDI object leak detection
dotnet run -- --verbose

# Run tests (test programs are in Test*.cs files)
# Note: Tests are standalone classes with Run() methods, not NUnit/xUnit tests
# Uncomment the test class's Run() call in Program.cs to execute
```

### AOT Compilation
```bash
# Publish with Native AOT (requires Visual Studio C++ tools)
dotnet publish -c Release -r win-x64

# Output location
# bin/Release/net10.0-windows/win-x64/publish/
```

### Testing
The project uses standalone test programs rather than a traditional test framework:
- `TestOverlayCalculator.cs` - Tests AppState in-place update logic
- `TestWindowMovement.cs` - Tests window movement detection
- To run tests: Uncomment the test's `Run()` method call in `Program.cs:7-15`

## Architecture

The codebase is organized into two main layers to separate platform-agnostic logic from Windows-specific implementation:

### Core Layer (`Core/`)
Pure C# calculation logic with zero Windows dependencies. All types are platform-agnostic.

**Key Files:**
- `AppState.cs` - Central state manager with pre-allocated overlay definitions. Uses in-place updates to achieve zero allocations in the hot path.
- `OverlayDefinition.cs` - Represents a single overlay's bounds, color, opacity, and visibility. Uses `CopyFrom()` for zero-allocation updates.
- `DisplayOverlayState.cs` - Holds 6 pre-allocated OverlayDefinitions per display (one for each OverlayRegion).
- `ConfigurationManager.cs` - Loads config from JSON and watches for file changes using FileSystemWatcher.
- `AppConfig.cs` - Deserialized configuration with mode, colors, and opacity settings.
- `DimmingMode.cs` - Enum for FullScreen, Partial, and PartialWithActive modes.
- `Primitives.cs` - Basic structs (Rectangle, Color, Point) used throughout Core.

### WindowsBindings Layer (`WindowsBindings/`)
Platform-specific Windows API integration using P/Invoke.

**Key Files:**
- `WinApi.cs` - All P/Invoke declarations for Win32 APIs (CreateWindowEx, SetWinEventHook, DeferWindowPos, etc.).
- `MonitorManager.cs` - Enumerates displays using EnumDisplayMonitors.
- `FocusTracker.cs` - Event-driven focus tracking using SetWinEventHook (EVENT_SYSTEM_FOREGROUND and EVENT_OBJECT_LOCATIONCHANGE). No polling!
- `OverlayRenderer.cs` - Manages a pool of overlay windows (6 per display), uses DeferWindowPos for atomic batch updates.

### Main Program (`Program.cs`)
Wires Core and WindowsBindings together. Creates instances, connects event handlers, runs Windows message loop.

**CRITICAL**: Never add Windows dependencies to the Core layer. Keep it pure and platform-agnostic.

## Critical Performance Patterns

### Zero-Allocation Hot Path
The codebase is designed to eliminate allocations during window movement/focus changes:

1. **Pre-allocation**: All overlays are created at startup (6 per display).
2. **In-place updates**: `OverlayDefinition.CopyFrom()` updates existing objects instead of creating new ones.
3. **Cached config**: `Program.cs:73-76` caches `displays` and `config` to avoid re-allocating on every event.
4. **Batch updates**: `OverlayRenderer.BatchUpdateWindows()` uses DeferWindowPos to update all windows atomically.

**IMPORTANT**: When modifying update logic, ensure you:
- Never allocate in the hot path (UpdateOverlays, Calculate, event handlers)
- Use `CopyFrom()` instead of creating new OverlayDefinition instances
- Reuse pre-allocated arrays and lists (see `_updateBatch` in OverlayRenderer.cs:23)

### Memory Leak Prevention
The codebase has specific patterns to prevent GDI/handle leaks:

**DeferWindowPos Handle Management** (`OverlayRenderer.cs:105-153`):
- Always call `EndDeferWindowPos()` even on failure (line 132)
- Track `lastValidHdwp` to ensure we clean up the correct handle
- See comment at line 128-131 for rationale

**GDI Object Monitoring** (`Program.cs:136-179`):
- Verbose mode logs GDI object counts every 5 seconds
- Helps detect brush/window handle leaks during development
- Use `--verbose` flag to enable

### Event-Driven Architecture
Unlike typical polling-based solutions, SpotlightDimmer is 100% event-driven:

- `EVENT_SYSTEM_FOREGROUND` - Fires when switching applications (0ms latency)
- `EVENT_OBJECT_LOCATIONCHANGE` - Fires on window movement (drag, Win+Arrow keyboard shortcuts)
- `FileSystemWatcher` - Detects config file changes for hot-reload

**CRITICAL**: Never add polling loops. Use Windows event hooks instead.

## Configuration System

Configuration is loaded from `%AppData%\SpotlightDimmer\config.json` with hot-reload support.

**File Structure:**
```json
{
  "Mode": "FullScreen",           // FullScreen, Partial, or PartialWithActive
  "InactiveColor": "#000000",     // Hex color for inactive overlays
  "InactiveOpacity": 153,         // 0-255 (153 = 60% opacity)
  "ActiveColor": "#000000",       // Hex color for active overlay (PartialWithActive only)
  "ActiveOpacity": 102            // 0-255 (102 = 40% opacity)
}
```

**Hot-Reload Behavior:**
- `ConfigurationManager` watches for file changes using `FileSystemWatcher`
- Fires `ConfigurationChanged` event when file is modified
- `Program.cs:86-101` handles the event by updating brushes and recalculating overlays
- Changes apply instantly without restart

See `CONFIGURATION.md` for detailed configuration options and examples.

## Overlay Calculation Logic

Each display can have up to 6 overlays (one per `OverlayRegion` enum value):
- `FullScreen` - Entire display
- `Top`, `Bottom`, `Left`, `Right` - Four sides around active window
- `Center` - Active window itself (used in PartialWithActive mode)

**Mode Behaviors:**
- **FullScreen**: Inactive displays show FullScreen overlay. Focused display has no overlays.
- **Partial**: Focused display shows Top/Bottom/Left/Right overlays around active window. Inactive displays show FullScreen overlay.
- **PartialWithActive**: Like Partial, but adds Center overlay on active window for enhanced spotlight effect.

**Calculation Flow** (`AppState.cs:41-79`):
1. `AppState.Calculate()` is called when focus/position changes
2. For each display, determine if it has the focused window
3. Based on mode and focus state, call appropriate update method:
   - `UpdateFullScreenOverlay()` - Single overlay covering entire display
   - `UpdatePartialOverlays()` - Four overlays around window
   - `UpdatePartialWithActiveOverlays()` - Five overlays (four sides + center)
4. Each method calls `OverlayDefinition.Update()` to modify existing objects
5. Hidden overlays are reset via `HideAllOverlays()` before calculation

## Code Style & Patterns

- Use `ReadOnlySpan<T>` for passing arrays without allocations
- Prefer structs over classes for hot-path data (Color, Rectangle, Point)
- Use `CopyFrom()` pattern for in-place updates instead of creating new instances
- Always dispose IDisposable resources in finally blocks
- Use XML doc comments (`///`) for all public APIs
- Keep Core layer pure - no Windows dependencies

## Common Development Scenarios

### Adding a New Dimming Mode
1. Add enum value to `Core/DimmingMode.cs`
2. Add case to `AppState.Calculate()` switch statement (line 60-71)
3. Implement update logic method (follow pattern of `UpdatePartialOverlays()`)
4. Update `CONFIGURATION.md` with new mode documentation
5. **Update CHANGELOG.md** with the new feature (including Portuguese translation)

### Modifying Overlay Rendering
1. Core logic changes go in `Core/AppState.cs`
2. Windows-specific changes go in `WindowsBindings/OverlayRenderer.cs`
3. Maintain separation: Core has no Windows dependencies
4. Ensure zero allocations in update paths
5. **Update CHANGELOG.md** with improvements (including Portuguese translation)

### Adding Configuration Options

**CRITICAL**: When adding new configuration options, you MUST update BOTH the JSON config AND the Config GUI app.

1. **Add property to `Core/AppConfig.cs`**
   - Add the new configuration property to the appropriate class (`OverlayConfig`, `SystemConfig`, etc.)
   - Include XML documentation comments explaining the purpose and valid values

2. **Update the Config GUI app** (`SpotlightDimmer.Config`)
   - Add UI control to `ConfigForm.Designer.cs`:
     - Create the control (ComboBox, CheckBox, NumericUpDown, etc.)
     - Position it appropriately in the form
     - Add event handler registration
     - Declare field at bottom of class
   - Add event handler to `ConfigForm.cs`:
     - Create `On[PropertyName]Changed` method
     - Check `if (!_isLoading)` to prevent recursion
     - Update `config.[Section].[Property]` value
     - Call `SaveConfiguration()`
   - Update `LoadConfiguration()` method:
     - Load the value from config and set the control's value
   - Adjust form size if needed to accommodate new controls

3. **Update `ToOverlayConfig()` method** (if overlay-related)
   - Map the new property to `OverlayCalculationConfig` if it affects rendering

4. **Handle in `Program.cs`** (if needed)
   - Add logic in `ConfigurationChanged` event handler if the change requires special handling

5. **Regenerate JSON schema**: Run `.\SpotlightDimmer.Scripts\Generate-Schema.ps1`
   - This ensures IntelliSense and validation in VS Code stay in sync

6. **Document in `CONFIGURATION.md`**
   - Add section explaining the new configuration option
   - Include valid values, defaults, and examples
   - Explain when users should use this option

7. **Update CHANGELOG.md** with configuration changes (including Portuguese translation)
   - Document the new option under "### Added" section
   - Include both English and Portuguese descriptions

**Example**: Adding a new `RendererBackend` option to `SystemConfig`:
- ✅ Added property to `AppConfig.cs` (`SystemConfig.RendererBackend`)
- ✅ Added ComboBox to `ConfigForm.Designer.cs` with "Legacy", "UpdateLayeredWindow", "Composition" options
- ✅ Added `OnRendererBackendChanged` event handler to `ConfigForm.cs`
- ✅ Updated `LoadConfiguration()` to load the value: `rendererBackendComboBox.SelectedItem = config.System.RendererBackend`
- ✅ Regenerated JSON schema
- ✅ Documented in CONFIGURATION.md with performance comparison chart
- ✅ Updated CHANGELOG.md (English + Portuguese)

**Why this matters**: The Config GUI is the primary way most users interact with configuration. If you only update `AppConfig.cs` without updating the GUI, users won't be able to change the setting through the UI.

### Regenerating JSON Schema
When configuration classes change (`AppConfig`, `OverlayConfig`, `SystemConfig`, `Profile`, or `DimmingMode`):
1. Run `.\SpotlightDimmer.Scripts\Generate-Schema.ps1`
2. Review changes to `config.schema.json`
3. Commit both code and schema changes together

**Why regenerate**: The JSON schema provides IntelliSense and validation in VS Code. It's automatically generated from C# types using NJsonSchema to ensure it stays in sync with the code.

**See**: `SpotlightDimmer.SchemaGenerator/README.md` for implementation details.

### Debugging Focus Tracking Issues
1. Run with `--verbose` flag to see all focus events
2. Check `FocusTracker.cs` event filtering (OBJID_WINDOW check at line 115)
3. Monitor GDI object counts to detect handle leaks
4. Use Spy++ to inspect window messages and hooks

## Platform-Specific Notes

### Windows-Only Compilation
- Project targets `net10.0-windows` (see `SpotlightDimmer.WindowsClient.csproj:5`)
- Requires `AllowUnsafeBlocks` for P/Invoke (line 8)
- Core layer is platform-agnostic but WindowsClient requires Windows

### Native AOT Constraints
- Code must be AOT-compatible (see csproj lines 12-20)
- Avoid reflection and dynamic code generation
- All P/Invoke signatures must be AOT-safe
- `InvariantGlobalization` is enabled (line 14)

### Visual Studio C++ Tools
Required for Native AOT compilation. Without them:
- Regular build/run works fine
- Publishing with AOT will fail with linker errors

## Key Win32 APIs Used

Understanding these APIs is essential for working with the WindowsBindings layer:

- **SetWinEventHook** - Registers callbacks for window events (focus, movement)
- **CreateWindowEx** - Creates overlay windows with WS_EX_LAYERED and WS_EX_TRANSPARENT
- **SetLayeredWindowAttributes** - Sets overlay opacity (60% for inactive, 40% for active)
- **DeferWindowPos/BeginDeferWindowPos/EndDeferWindowPos** - Batch window updates atomically
- **EnumDisplayMonitors** - Enumerates all connected displays
- **GetMessage/DispatchMessage** - Windows message loop in `Program.cs:154-180`

## Documentation Guidelines

**IMPORTANT**: All documentation files (*.md files other than README.md and CONFIGURATION.md in the root) should be created in the `docs/` folder unless explicitly instructed otherwise.

- ✅ Place new documentation in `docs/` directory
- ✅ Root-level README.md and CONFIGURATION.md are exceptions
- ❌ Do NOT create new .md files in the root directory
- ❌ Do NOT create documentation in `.github/` directory unless it's GitHub-specific (like PULL_REQUEST_TEMPLATE.md)

Examples:
- Setup guides → `docs/WINGET_SETUP.md`
- Architecture docs → `docs/ARCHITECTURE.md`
- Troubleshooting → `docs/TROUBLESHOOTING.md`

## Quality Checklist

Before completing any task, verify:
- [ ] Code follows layer separation (Core vs WindowsBindings)
- [ ] No allocations in hot path (event handlers, Calculate, UpdateOverlays)
- [ ] Memory leaks prevented (handles properly cleaned up)
- [ ] XML doc comments added for public APIs
- [ ] Configuration documented in CONFIGURATION.md if applicable
- [ ] **CHANGELOG.md updated with changes**
- [ ] **Portuguese translation included in CHANGELOG.md**
- [ ] Commit message is clear and descriptive
- [ ] No "Generated with Claude Code" in commit messages
