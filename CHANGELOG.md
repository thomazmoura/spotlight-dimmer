# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.11] - 2025-11-11

### Fixed
- **ARM64 build failure**: Removed hardcoded `PlatformTarget` from project files to allow multi-architecture builds
  - Removed `<PlatformTarget>x64</PlatformTarget>` from SpotlightDimmer.WindowsClient.csproj
  - Removed `<PlatformTarget>x64</PlatformTarget>` from SpotlightDimmer.Config.csproj
  - Platform target is now correctly inferred from the runtime identifier (win-x64 or win-arm64)
  - Fixes "NETSDK1032: The RuntimeIdentifier platform 'win-arm64' and the PlatformTarget 'x64' must be compatible" error
  - Enables successful ARM64 builds in CI/CD pipeline

---

### Corrigido
- **Falha na compilação ARM64**: Removido `PlatformTarget` fixo dos arquivos de projeto para permitir compilações multi-arquitetura
  - Removido `<PlatformTarget>x64</PlatformTarget>` do SpotlightDimmer.WindowsClient.csproj
  - Removido `<PlatformTarget>x64</PlatformTarget>` do SpotlightDimmer.Config.csproj
  - Plataforma alvo agora é corretamente inferida do identificador de runtime (win-x64 ou win-arm64)
  - Corrige erro "NETSDK1032: The RuntimeIdentifier platform 'win-arm64' and the PlatformTarget 'x64' must be compatible"
  - Habilita compilações ARM64 bem-sucedidas no pipeline CI/CD

## [Unreleased]

### Fixed
- **ARM64 build failure**: Removed hardcoded `PlatformTarget` from project files to allow multi-architecture builds
  - Removed `<PlatformTarget>x64</PlatformTarget>` from SpotlightDimmer.WindowsClient.csproj
  - Removed `<PlatformTarget>x64</PlatformTarget>` from SpotlightDimmer.Config.csproj
  - Platform target is now correctly inferred from the runtime identifier (win-x64 or win-arm64)
  - Fixes "NETSDK1032: The RuntimeIdentifier platform 'win-arm64' and the PlatformTarget 'x64' must be compatible" error
  - Enables successful ARM64 builds in CI/CD pipeline

---

### Corrigido
- **Falha na compilação ARM64**: Removido `PlatformTarget` fixo dos arquivos de projeto para permitir compilações multi-arquitetura
  - Removido `<PlatformTarget>x64</PlatformTarget>` do SpotlightDimmer.WindowsClient.csproj
  - Removido `<PlatformTarget>x64</PlatformTarget>` do SpotlightDimmer.Config.csproj
  - Plataforma alvo agora é corretamente inferida do identificador de runtime (win-x64 ou win-arm64)
  - Corrige erro "NETSDK1032: The RuntimeIdentifier platform 'win-arm64' and the PlatformTarget 'x64' must be compatible"
  - Habilita compilações ARM64 bem-sucedidas no pipeline CI/CD

## [0.8.10] - 2025-11-11

### Added
- **Full Windows ARM64 support**: Native ARM64 binaries for Windows on ARM devices
  - Native AOT compilation for win-arm64 runtime identifier (Surface, Snapdragon X Elite/Plus laptops)
  - Separate x64 and ARM64 installers for optimal performance on each architecture
  - No emulation overhead: native execution provides better battery life and performance
  - Automated ARM64 build testing using GitHub Actions windows-arm64 runners
  - Release workflow builds and publishes both architectures automatically
  - Winget manifest includes both x64 and ARM64 installers with proper architecture detection
  - ZIP portable packages available for both x64 and ARM64
  - Supports growing Windows on ARM ecosystem (Dell XPS 13, Lenovo ThinkPad T14s Gen 6, Microsoft Surface Laptop 7)

- **Comprehensive Winget installation testing in publish workflow**: Added automated testing before submitting to Windows Package Manager repository
  - Local installation test using generated manifest validates installer before public submission
  - Automated smoke test: installs, launches application, waits 10 seconds, closes gracefully
  - Log file validation: ensures logs are created and contain no error-level messages
  - Displays log preview (first 20 lines) in CI output for debugging
  - Clean uninstallation test verifies proper cleanup
  - Prevents broken manifests from reaching microsoft/winget-pkgs repository
  - Workflow only submits to public repository if all tests pass

### Fixed
- **Winget publishing workflow architecture detection**: Fixed `winget-create` misdetecting installer as X86 instead of x64
  - Added `--architecture-override "x64"` parameter to Winget publishing workflow
  - Resolves "Multiple matches found for X86 Inno installer" error during Winget manifest updates
  - Ensures correct x64 architecture detection despite Inno Setup's `x64compatible` configuration
  - Workflow now correctly publishes to Windows Package Manager repository

### Improved
- **Automated changelog versioning in release process**: Release commands now automatically maintain proper CHANGELOG.md versioning
  - Created `Move-UnreleasedToVersion.ps1` script to move [Unreleased] → [X.Y.Z] - YYYY-MM-DD
  - Updated `/publish-patch` and `/publish-minor` commands to run versioning script before validation
  - Ensures CHANGELOG.md always has proper version sections for each release
  - Eliminates manual changelog maintenance during releases
  - Versioned changelog sections automatically flow into GitHub release notes
  - Updated v0.8.9 release retroactively with full changelog content (first release with complete notes)
  - Future releases will automatically include comprehensive bilingual changelogs
  - Updated AGENTS.md documentation with release process integration details

---

### Adicionado
- **Suporte completo para Windows ARM64**: Binários nativos ARM64 para dispositivos Windows on ARM
  - Compilação Native AOT para identificador de runtime win-arm64 (Surface, laptops Snapdragon X Elite/Plus)
  - Instaladores separados x64 e ARM64 para desempenho ótimo em cada arquitetura
  - Sem sobrecarga de emulação: execução nativa proporciona melhor duração de bateria e desempenho
  - Testes automatizados de build ARM64 usando runners windows-arm64 do GitHub Actions
  - Workflow de release compila e publica ambas arquiteturas automaticamente
  - Manifesto Winget inclui instaladores x64 e ARM64 com detecção adequada de arquitetura
  - Pacotes portáteis ZIP disponíveis para x64 e ARM64
  - Suporta ecossistema crescente de Windows on ARM (Dell XPS 13, Lenovo ThinkPad T14s Gen 6, Microsoft Surface Laptop 7)

- **Testes abrangentes de instalação Winget no workflow de publicação**: Adicionados testes automatizados antes de submeter ao repositório do Windows Package Manager
  - Teste de instalação local usando manifesto gerado valida instalador antes de submissão pública
  - Teste de smoke automatizado: instala, inicia aplicação, aguarda 10 segundos, fecha graciosamente
  - Validação de arquivo de log: garante que logs são criados e não contêm mensagens de nível de erro
  - Exibe prévia do log (primeiras 20 linhas) na saída do CI para depuração
  - Teste de desinstalação limpa verifica limpeza adequada
  - Previne manifestos quebrados de alcançar repositório microsoft/winget-pkgs
  - Workflow apenas submete ao repositório público se todos os testes passarem

### Corrigido
- **Detecção de arquitetura no workflow de publicação Winget**: Corrigido `winget-create` detectando incorretamente instalador como X86 em vez de x64
  - Adicionado parâmetro `--architecture-override "x64"` ao workflow de publicação Winget
  - Resolve erro "Multiple matches found for X86 Inno installer" durante atualizações de manifesto Winget
  - Garante detecção correta de arquitetura x64 apesar da configuração `x64compatible` do Inno Setup
  - Workflow agora publica corretamente no repositório do Windows Package Manager

### Melhorado
- **Versionamento automatizado de changelog no processo de release**: Comandos de release agora mantêm automaticamente versionamento adequado do CHANGELOG.md
  - Criado script `Move-UnreleasedToVersion.ps1` para mover [Unreleased] → [X.Y.Z] - YYYY-MM-DD
  - Atualizados comandos `/publish-patch` e `/publish-minor` para executar script de versionamento antes da validação
  - Garante que CHANGELOG.md sempre tenha seções de versão adequadas para cada release
  - Elimina manutenção manual do changelog durante releases
  - Seções de changelog versionadas fluem automaticamente para notas de release do GitHub
  - Atualizado release v0.8.9 retroativamente com conteúdo completo do changelog (primeiro release com notas completas)
  - Releases futuros incluirão automaticamente changelogs bilíngues abrangentes
  - Atualizada documentação AGENTS.md com detalhes de integração do processo de release

## [0.8.9] - 2025-11-10

### Added
- **Pluggable renderer architecture with two rendering backends**: Implemented abstraction layer allowing users to choose between lightweight and smooth rendering
  - Created `IOverlayRenderer` interface to decouple core logic from rendering implementation
  - **LayeredWindow renderer** (default): SetWindowPos + SetLayeredWindowAttributes approach with lightweight memory footprint (~1-5 MB)
  - **UpdateLayeredWindow renderer**: Uses Windows UpdateLayeredWindow API with full-screen bitmaps for reduced visual gaps during window dragging
  - New `RendererBackend` configuration option in `System` section: "LayeredWindow" (default) or "UpdateLayeredWindow"
  - UpdateLayeredWindow significantly reduces visual gaps but requires ~50-100 MB memory per display due to full-screen bitmap allocation
  - Memory tradeoff: LayeredWindow uses GDI brushes (minimal memory), UpdateLayeredWindow uses 32-bit ARGB bitmaps (width × height × 4 bytes per overlay)
  - UpdateLayeredWindow makes gaps smaller and more uniform across all edges (not just right/bottom)
  - Automatic fallback to LayeredWindow renderer if configured backend fails or is unavailable
  - Both renderers share identical Core calculation logic ensuring consistent behavior
  - "Legacy" accepted as backward-compatible alias for "LayeredWindow"

- **CompositeOverlay renderer**: New rendering backend that uses only 2 windows per display (instead of 6) by compositing multiple overlay regions into a single bitmap
  - Reduces GDI handle count from 12 to 4 (for 2 displays) through window consolidation
  - Eliminates window resize/reposition operations - windows stay fullscreen-sized
  - Uses per-pixel alpha compositing to draw up to 5 overlay regions in a single bitmap
  - Memory tradeoff: ~16MB for 2 displays (1920×1080 × 4 bytes ARGB × 2) vs ~48KB for UpdateLayeredWindow
  - Best for multi-monitor setups where minimizing GDI handle count is a priority
  - Particularly beneficial for PartialWithActive mode with frequent window movement
  - Available via `RendererBackend: "CompositeOverlay"` in System configuration
  - Documented in CONFIGURATION.md with performance comparison table

- **JSON schema for configuration**: Added comprehensive JSON schema file for IntelliSense and validation in VS Code
  - Schema file `config.schema.json` provides autocomplete, validation, and inline documentation
  - Hover over properties to see descriptions, allowed values, and recommended settings
  - Dropdown suggestions for enum values (Mode: FullScreen/Partial/PartialWithActive, LogLevel: Error/Warning/Information/Debug)
  - Real-time validation for hex color codes, opacity ranges (0-255), and required properties
  - Example configuration updated to reference schema via `$schema` property
  - Documentation in CONFIGURATION.md explains how to enable schema validation
  - Works with both GitHub URL and local relative path references
  - Improves configuration editing experience and reduces user errors

---

### Adicionado
- **Arquitetura de renderização plugável com dois backends de renderização**: Implementada camada de abstração permitindo aos usuários escolher entre renderização leve e suave
  - Criada interface `IOverlayRenderer` para desacoplar lógica central da implementação de renderização
  - **Renderizador LayeredWindow** (padrão): Abordagem SetWindowPos + SetLayeredWindowAttributes com pegada de memória leve (~1-5 MB)
  - **Renderizador UpdateLayeredWindow**: Usa API Windows UpdateLayeredWindow com bitmaps de tela cheia para reduzir lacunas visuais durante arrasto de janelas
  - Nova opção de configuração `RendererBackend` na seção `System`: "LayeredWindow" (padrão) ou "UpdateLayeredWindow"
  - UpdateLayeredWindow reduz significativamente lacunas visuais mas requer ~50-100 MB de memória por display devido à alocação de bitmap de tela cheia
  - Tradeoff de memória: LayeredWindow usa brushes GDI (memória mínima), UpdateLayeredWindow usa bitmaps ARGB 32-bit (largura × altura × 4 bytes por sobreposição)
  - UpdateLayeredWindow torna lacunas menores e mais uniformes em todas as bordas (não apenas direita/inferior)
  - Fallback automático para renderizador LayeredWindow se backend configurado falhar ou não estiver disponível
  - Ambos os renderizadores compartilham lógica de cálculo Core idêntica garantindo comportamento consistente
  - "Legacy" aceito como alias retrocompatível para "LayeredWindow"

- **Renderizador CompositeOverlay**: Novo backend de renderização que usa apenas 2 janelas por display (em vez de 6) ao compor múltiplas regiões de sobreposição em um único bitmap
  - Reduz contagem de handles GDI de 12 para 4 (para 2 displays) através de consolidação de janelas
  - Elimina operações de redimensionamento/reposicionamento de janelas - janelas permanecem em tamanho de tela cheia
  - Usa composição alfa por pixel para desenhar até 5 regiões de sobreposição em um único bitmap
  - Tradeoff de memória: ~16MB para 2 displays (1920×1080 × 4 bytes ARGB × 2) vs ~48KB para UpdateLayeredWindow
  - Melhor para configurações multi-monitor onde minimizar contagem de handles GDI é prioridade
  - Particularmente benéfico para modo PartialWithActive com movimento frequente de janelas
  - Disponível via `RendererBackend: "CompositeOverlay"` na configuração System
  - Documentado em CONFIGURATION.md com tabela de comparação de desempenho

- **JSON schema para configuração**: Adicionado arquivo JSON schema abrangente para IntelliSense e validação no VS Code
  - Arquivo de schema `config.schema.json` fornece autocomplete, validação e documentação inline
  - Passe o mouse sobre propriedades para ver descrições, valores permitidos e configurações recomendadas
  - Sugestões dropdown para valores enum (Mode: FullScreen/Partial/PartialWithActive, LogLevel: Error/Warning/Information/Debug)
  - Validação em tempo real para códigos de cor hex, intervalos de opacidade (0-255) e propriedades obrigatórias
  - Configuração de exemplo atualizada para referenciar o schema via propriedade `$schema`
  - Documentação em CONFIGURATION.md explica como habilitar validação de schema
  - Funciona com referências de URL do GitHub e caminho relativo local
  - Melhora a experiência de edição de configuração e reduz erros do usuário

- **Automated schema generation**: Created tool to automatically generate JSON schema from C# configuration classes
  - New `SpotlightDimmer.SchemaGenerator` console app uses NJsonSchema to generate schema via reflection
  - PowerShell script `Generate-Schema.ps1` provides one-command schema regeneration
  - Ensures JSON schema stays synchronized with C# types automatically
  - Eliminates manual schema maintenance and prevents drift between code and schema
  - Documented in `SpotlightDimmer.SchemaGenerator/README.md` and `AGENTS.md`
  - Schema regenerates with single command: `.\SpotlightDimmer.Scripts\Generate-Schema.ps1`
  - Maintains single source of truth: C# classes drive the schema

- **Automatic schema injection and version-aware URLs**: Configuration files automatically get IntelliSense support without manual intervention
  - Application automatically injects `$schema` property into config.json on first run
  - Uses version-specific schema URLs (e.g., `v0.8.5`) for accurate autocomplete
  - Automatically updates schema URL when upgrading to newer versions
  - Added `ConfigVersion` property to track configuration file version
  - Created `SchemaInjector` class for manipulating JSON while preserving formatting
  - Older config files point to their original version's schema (prevents confusion from newer properties)
  - Seamless user experience: IntelliSense works immediately after installation
  - ConfigurationManager logs schema injection and version updates for transparency

---

### Adicionado
- **Geração automatizada de schema**: Criada ferramenta para gerar automaticamente JSON schema a partir das classes de configuração C#
  - Novo console app `SpotlightDimmer.SchemaGenerator` usa NJsonSchema para gerar schema via reflexão
  - Script PowerShell `Generate-Schema.ps1` fornece regeneração de schema com um único comando
  - Garante que JSON schema permaneça sincronizado com tipos C# automaticamente
  - Elimina manutenção manual do schema e previne divergência entre código e schema
  - Documentado em `SpotlightDimmer.SchemaGenerator/README.md` e `AGENTS.md`
  - Schema regenera com comando único: `.\SpotlightDimmer.Scripts\Generate-Schema.ps1`
  - Mantém fonte única de verdade: classes C# direcionam o schema

- **Injeção automática de schema e URLs versionadas**: Arquivos de configuração obtêm suporte IntelliSense automaticamente sem intervenção manual
  - Aplicação injeta automaticamente propriedade `$schema` no config.json na primeira execução
  - Usa URLs de schema específicas por versão (ex.: `v0.8.5`) para autocomplete preciso
  - Atualiza automaticamente URL do schema ao atualizar para versões mais recentes
  - Adicionada propriedade `ConfigVersion` para rastrear versão do arquivo de configuração
  - Criada classe `SchemaInjector` para manipular JSON preservando formatação
  - Arquivos de configuração antigos apontam para schema de sua versão original (previne confusão com propriedades mais recentes)
  - Experiência de usuário perfeita: IntelliSense funciona imediatamente após instalação
  - ConfigurationManager registra injeção de schema e atualizações de versão para transparência

### Improved
- **README documentation update**: Removed outdated references to Rust version and proof-of-concept status, presenting SpotlightDimmer as the main production version
  - Removed "Proof of Concept" from title and introduction
  - Removed entire "Comparison to Rust Version" section with performance comparisons and trade-offs
  - Updated description to focus on current capabilities rather than historical context
  - Added Configuration section linking to CONFIGURATION.md for better discoverability
  - Streamlined documentation to present professional, production-ready application
  - Corrected transparency descriptions to indicate configurable opacity instead of hardcoded values
  - Removed unbenchmarked performance metrics (e.g., "0ms latency") to maintain accuracy
  - Updated event hook descriptions to reflect current implementation without outdated filtering details
  - Enhanced Features section with detailed capabilities:
    - Highlighted independent color and opacity configuration for inactive and active regions
    - Documented rendering backend options with memory usage characteristics (LayeredWindow < 10MB, CompositeOverlay ~50MB for dual monitor)
    - Added installation footprint metrics (< 50MB installed, < 10MB installer)
    - Emphasized hot-reload capability and event-driven architecture benefits

- **Renderer backend hot-reload support**: Renderer backend changes now apply instantly without requiring application restart
  - Application detects when `System.RendererBackend` configuration changes
  - Automatically disposes old renderer and creates new renderer instance
  - Recreates all overlay windows with new rendering backend
  - Preserves screen capture exclusion and other settings during renderer recreation
  - Enables seamless comparison between LayeredWindow, UpdateLayeredWindow, and CompositeOverlay renderers
  - Logged transitions show old and new backend names for transparency
  - Consistent with existing hot-reload behavior for colors, opacity, and other settings

- **Simplified schema property management**: Configuration schema URL is now managed directly as a class property instead of post-serialization injection
  - Added `Schema` property to `AppConfig` class with `$schema` JSON attribute for direct serialization
  - Removed `SchemaInjector` class simplifying codebase by eliminating manual JSON string manipulation
  - Schema URL automatically updates via `UpdateVersion()` method ensuring consistency
  - `JsonPropertyOrder` attribute ensures `$schema` appears first in serialized JSON
  - Comprehensive unit tests added to verify schema property serialization and deserialization
  - Cleaner architecture with schema management integrated into the data model

- **Testable focus tracking architecture**: Focus change logic refactored to Core layer for comprehensive unit testing
  - Created `IOverlayUpdateService` interface to abstract overlay updates from focus tracking logic
  - New `FocusChangeHandler` class in Core layer contains platform-agnostic decision logic
  - Zero-dimension window filtering (0x0 windows) now has test coverage to prevent regressions
  - Display change detection logic now fully testable without Windows dependencies
  - FocusTracker simplified to thin adapter that translates Windows events to Core domain objects
  - Added NSubstitute for mocking in tests - enables verification of overlay update calls
  - 20+ comprehensive unit tests covering edge cases: zero dimensions, display changes, position changes, state tracking
  - Separation of concerns: WindowsBindings handles platform I/O, Core handles business logic
  - Enables future testing of UWP window selection logic and other focus tracking scenarios

- **Comprehensive release notes with changelog integration**: GitHub releases now automatically include detailed changelog information
  - Created `Extract-Changelog.ps1` script to extract [Unreleased] section from CHANGELOG.md
  - Updated release workflow to inject changelog content at the top of release descriptions
  - Release notes now show what changed in each version in both English and Portuguese
  - Matches historical format used in v0.7.2 and earlier releases
  - Improves user experience by making changes immediately visible in release pages
  - Maintains single source of truth: CHANGELOG.md drives both documentation and release notes

### Fixed
- **Popup window highlighting regression (v0.8.3)**: Fixed system tray menus and popup windows not being highlighted correctly
  - **Symptom**: When opening popup windows (context menus, dropdowns), overlays dimmed randomly instead of highlighting the popup
  - **Root cause**: Zero-dimension filtering was too aggressive - popup windows temporarily report 0x0 dimensions during initialization
  - **Impact**: The zero-dimension check caused early return without tracking display changes, leaving overlays in wrong position
  - **Solution**: Modified FocusChangeHandler to track display changes even for zero-dimension windows, but defer overlay updates until valid dimensions arrive
  - Restores v0.8.2 behavior where popups are correctly highlighted
  - Maintains flickering prevention for truly invalid window states
  - Added comprehensive test coverage for popup window scenarios

- **Zero-dimension window detection**: Fixed incorrect handling of windows with zero width or zero height
  - Bug: Code was using AND (&&) logic requiring BOTH width AND height to be zero before ignoring
  - Fix: Changed to OR (||) logic to properly ignore windows with zero width OR zero height
  - Impact: Prevents overlay updates on minimized windows, certain UI elements, and invalid window states
  - Fixed in FocusChangeHandler.cs:72 - changed condition from `Width == 0 && Height == 0` to `Width == 0 || Height == 0`
  - Also corrected test data bug in FocusChangeHandlerTests.cs:252 where test parameters were in wrong order

- **UWP app overlay positioning**: System Settings and other UWP apps now correctly position overlays on first launch
  - **Root cause**: UWP apps run inside ApplicationFrameHost.exe container. GetForegroundWindow() returns the container (reports 0x0/wrong dimensions), not the content window (CoreWindow)
  - **Additional issue**: EVENT_SYSTEM_FOREGROUND doesn't fire when UWP apps finish launching - the app is already "foreground" by the time it's ready
  - **Solution**: Two-part fix:
    1. Detect ApplicationFrameHost and enumerate child windows to find actual content window with correct dimensions
    2. Add lightweight polling (100ms) to catch foreground changes that don't fire events (hybrid event + polling approach)
  - Event hooks handle 99% of cases; polling only catches edge cases like UWP launches
  - Fixes mispositioned overlays when launching apps from Start Menu, PowerToys.Run, etc.
  - Works for all UWP/modern apps: Settings, Calculator, modern Office apps, etc.

- **Threading bug causing duplicate overlays with polling detection**: Fixed issue where UWP apps opened via polling showed both old and new overlays simultaneously
  - **Symptom**: When opening fullscreen UWP apps (SystemSettings, etc.) while PowerToys Run or other apps were open, both old partial overlays and new overlays appeared at the same time
  - **Root cause**: Polling timer runs on ThreadPool background thread. When it called `SetWindowPos()`, those GDI operations were queued in the message loop but didn't execute until the next message arrived, creating a race condition where old and new overlays were both visible
  - **Solution**: Created a message-only window to marshal polling updates to the UI thread via `PostMessage(WM_FOCUS_UPDATE)`, ensuring all overlay updates execute synchronously on the thread that owns the windows
  - Impact: Eliminates visual artifacts and duplicate overlays when UWP apps are detected through polling
  - All GDI operations now run on the correct thread, preventing message queue backlog
  - Event-driven updates continue working as before (they already ran on the UI thread)

- **Flickering during invalid window bounds**: Overlays now remain stable when Windows temporarily reports 0x0 window dimensions
  - Windows temporarily reports 0x0 dimensions during focus changes and window transitions
  - Early return prevents unnecessary overlay recalculation when bounds are invalid (width or height = 0)
  - Existing overlay state naturally persists due to in-place update architecture
  - Eliminates flickering and visual artifacts during focus switches
  - Zero allocations - simply skips update cycle until valid bounds arrive
  - Works across all dimming modes (FullScreen, Partial, PartialWithActive)

### Improved
- **Automatic version extraction in Build-Installer.ps1**: Build script now automatically extracts version from Directory.Build.props
  - Version parameter is now optional - defaults to version from Directory.Build.props with "-dev" suffix
  - Eliminates manual version synchronization when building local installers
  - Example: With version 0.8.2 in Directory.Build.props, running `.\Build-Installer.ps1` creates installer version "0.8.2-dev"
  - Can still override with explicit version: `.\Build-Installer.ps1 -Version 1.0.0`
  - PowerShell XML parsing extracts `<Version>` element directly from centralized version file
  - Friendly console messages indicate extracted version or fallback to default if file not found

### Added
- **Auto-start at login**: New system tray menu option to automatically start SpotlightDimmer when Windows boots
  - "Start at Login" checkbox menu item in system tray context menu
  - Clicking toggles auto-start on/off via Windows Registry (HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run)
  - Menu shows checkmark when auto-start is enabled
  - Works reliably across Windows restarts and user sessions
  - No administrator privileges required - uses per-user registry key
  - Based on proven Rust implementation design

- **Screenshot exclusion (EXPERIMENTAL)**: Optional setting to exclude overlay windows from screen captures
  - New configuration option: `ExcludeFromScreenCapture` in `Overlay` section (default: false)
  - Uses Windows `SetWindowDisplayAffinity` API with `WDA_EXCLUDEFROMCAPTURE` flag
  - When enabled, overlays are excluded from screenshots taken with PrintScreen, Snipping Tool, Greenshot, etc.
  - Checkbox in config app under "Experimental Features" section with clear disclaimer
  - Dynamically toggleable via config file - applies immediately without restart via hot-reload
  - Graceful fallback: If API fails (layered window incompatibility), overlays continue working normally
  - Diagnostic logging shows success/failure count when enabled
  - **Known limitation**: May not work on all Windows systems due to API restrictions with `WS_EX_LAYERED` windows
  - Success rate varies by Windows version and system configuration (works better on Windows 10 2004+ and Windows 11)
  - Particularly useful for creating demos, tutorials, or documentation without dimming artifacts in screenshots

- **System tray keyboard accessibility**: Full keyboard support for system tray icon navigation
  - Press **Space** to toggle pause/resume (same as double-click)
  - Press **Enter**, **Apps key**, or **Shift+F10** to open the context menu (same as right-click)
  - Intuitive key mapping: Space for quick toggle, Enter for menu access
  - Improves accessibility for keyboard-only users
  - Follows Windows accessibility guidelines for system tray interactions
  - Uses `GetAsyncKeyState` to properly distinguish between Enter and Space key presses

- **Improved context menu positioning**: Menu now appears at tray icon location instead of cursor position
  - Uses `Shell_NotifyIconGetRect` to get exact tray icon coordinates
  - Menu positioned at center-bottom of tray icon for consistency
  - Cursor stays in place (not moved to tray icon)
  - Fallback to cursor position if icon coordinates unavailable
  - Professional behavior matching Windows system applications

- **Diagnostics submenu in system tray**: New diagnostics tools for troubleshooting and log management
  - "View Logs Folder" - Opens the logs directory in Windows Explorer
  - "View Latest Log" - Opens the most recent log file in Notepad (with smart validation)
  - "Enable Logging" - Toggle checkbox to enable/disable logging on the fly
  - Logs stored in `%AppData%\SpotlightDimmer\logs\` with daily rotation
  - Intelligent error handling: shows message box if logging disabled and no logs exist

- **Help menu with About and GitHub access**: New Help submenu in system tray for app information and support
  - "About Spotlight Dimmer" - Shows dialog with version, author (Thomaz Moura), technology stack (.NET, WinForms, Windows APIs), and GitHub URL
  - "Visit Github page" - Opens the project's GitHub repository (github.com/thomazmoura/spotlight-dimmer) in default browser
  - Version automatically extracted from assembly metadata
  - Clean, informative MessageBox interface with proper icon
  - Direct browser integration using ShellExecute API

### Changed
- **System tray menu reorganization**: Menu structure improved with logical grouping into Settings and Help
  - New "Settings" submenu consolidates configuration and system options:
    - Configuration... and Open Config File moved to top for easy access
    - Start at Login option moved from top level
    - Diagnostics items (View Logs Folder, View Latest Log, Enable Logging) moved from submenu and flattened into Settings
  - New "Help" submenu for support and information:
    - About Spotlight Dimmer
    - Visit Github page
  - Simplified menu hierarchy reduces nesting depth for better accessibility
  - More intuitive organization following Windows UI conventions
- **File-based logging system replaces console output**: Professional structured logging with Serilog
  - Logs written to `%AppData%\SpotlightDimmer\logs\spotlight-YYYY-MM-DD.log`
  - Configurable log levels: Error, Information, Debug (default: Information)
  - Automatic daily log rotation with configurable retention (default: 7 days)
  - Hot-reload support - logging changes apply immediately without restart
  - Zero console window - application runs silently when launched from GUI
  - All console output migrated to structured logging with context-aware log levels
  - Configuration properties: `EnableLogging` (bool), `LogLevel` (string), `LogRetentionDays` (int)

### Fixed
- **Application now runs without console window**: Changed from console to Windows subsystem
  - No console window appears when launched from Start Menu, shortcuts, or auto-start
  - Silent operation with system tray as the only visible UI component
  - Professional user experience matching modern Windows applications

---

### Corrigido
- **Regressão de destaque de janelas popup (v0.8.3)**: Corrigido menus da bandeja do sistema e janelas popup não sendo destacadas corretamente
  - **Sintoma**: Ao abrir janelas popup (menus de contexto, dropdowns), sobreposições escureciam aleatoriamente ao invés de destacar o popup
  - **Causa raiz**: Filtragem de dimensão zero era muito agressiva - janelas popup temporariamente reportam dimensões 0x0 durante inicialização
  - **Impacto**: A verificação de dimensão zero causava retorno antecipado sem rastrear mudanças de display, deixando sobreposições na posição errada
  - **Solução**: Modificado FocusChangeHandler para rastrear mudanças de display mesmo para janelas com dimensão zero, mas adiar atualizações de sobreposição até dimensões válidas chegarem
  - Restaura comportamento v0.8.2 onde popups são corretamente destacados
  - Mantém prevenção de tremulação para estados de janela verdadeiramente inválidos
  - Adicionada cobertura de testes abrangente para cenários de janelas popup

- **Detecção de janelas com dimensão zero**: Corrigido tratamento incorreto de janelas com largura ou altura zero
  - Bug: Código estava usando lógica AND (&&) exigindo que AMBAS largura E altura fossem zero antes de ignorar
  - Correção: Alterado para lógica OR (||) para ignorar adequadamente janelas com largura OU altura zero
  - Impacto: Previne atualizações de sobreposição em janelas minimizadas, certos elementos de interface, e estados de janela inválidos
  - Corrigido em FocusChangeHandler.cs:72 - alterada condição de `Width == 0 && Height == 0` para `Width == 0 || Height == 0`
  - Também corrigido bug de dados de teste em FocusChangeHandlerTests.cs:252 onde parâmetros de teste estavam em ordem errada

- **Posicionamento de sobreposição em apps UWP**: Configurações do Sistema e outros apps UWP agora posicionam sobreposições corretamente no primeiro lançamento
  - **Causa raiz**: Apps UWP executam dentro do contêiner ApplicationFrameHost.exe. GetForegroundWindow() retorna o contêiner (reporta dimensões 0x0/incorretas), não a janela de conteúdo (CoreWindow)
  - **Problema adicional**: EVENT_SYSTEM_FOREGROUND não dispara quando apps UWP terminam de carregar - o app já está "em foco" quando está pronto
  - **Solução**: Correção em duas partes:
    1. Detectar ApplicationFrameHost e enumerar janelas filhas para encontrar janela de conteúdo real com dimensões corretas
    2. Adicionar polling leve (100ms) para capturar mudanças de foco que não disparam eventos (abordagem híbrida evento + polling)
  - Event hooks lidam com 99% dos casos; polling apenas captura casos extremos como lançamentos UWP
  - Corrige sobreposições mal posicionadas ao lançar apps do Menu Iniciar, PowerToys.Run, etc.
  - Funciona para todos os apps UWP/modernos: Configurações, Calculadora, apps modernos do Office, etc.

- **Bug de threading causando sobreposições duplicadas com detecção por polling**: Corrigido problema onde apps UWP abertos via polling mostravam sobreposições antigas e novas simultaneamente
  - **Sintoma**: Ao abrir apps UWP em tela cheia (Configurações do Sistema, etc.) enquanto PowerToys Run ou outros apps estavam abertos, sobreposições parciais antigas e novas apareciam ao mesmo tempo
  - **Causa raiz**: Timer de polling executa em thread de background do ThreadPool. Quando chamava `SetWindowPos()`, essas operações GDI eram enfileiradas no message loop mas não executavam até a próxima mensagem chegar, criando uma condição de corrida onde sobreposições antigas e novas ficavam visíveis simultaneamente
  - **Solução**: Criada janela message-only para organizar atualizações de polling para a thread UI via `PostMessage(WM_FOCUS_UPDATE)`, garantindo que todas as atualizações de sobreposição executem sincronamente na thread que possui as janelas
  - Impacto: Elimina artefatos visuais e sobreposições duplicadas quando apps UWP são detectados através de polling
  - Todas as operações GDI agora executam na thread correta, prevenindo acúmulo na fila de mensagens
  - Atualizações orientadas a eventos continuam funcionando como antes (já executavam na thread UI)

- **Tremulação durante bounds de janela inválidos**: Sobreposições agora permanecem estáveis quando Windows temporariamente reporta dimensões 0x0 de janela
  - Windows temporariamente reporta dimensões 0x0 durante mudanças de foco e transições de janela
  - Retorno antecipado previne recálculo desnecessário de sobreposição quando bounds são inválidos (largura ou altura = 0)
  - Estado de sobreposição existente naturalmente persiste devido à arquitetura de atualização in-place
  - Elimina tremulação e artefatos visuais durante trocas de foco
  - Zero alocações - simplesmente pula ciclo de atualização até que bounds válidos cheguem
  - Funciona em todos os modos de escurecimento (FullScreen, Partial, PartialWithActive)

### Melhorado
- **Atualização da documentação do README**: Removidas referências desatualizadas à versão Rust e status de prova de conceito, apresentando SpotlightDimmer como a versão de produção principal
  - Removido "Proof of Concept" do título e introdução
  - Removida seção inteira "Comparison to Rust Version" com comparações de desempenho e trade-offs
  - Atualizada descrição para focar nas capacidades atuais ao invés de contexto histórico
  - Adicionada seção Configuração vinculando a CONFIGURATION.md para melhor descobribilidade
  - Documentação simplificada para apresentar aplicação profissional e pronta para produção
  - Corrigidas descrições de transparência para indicar opacidade configurável ao invés de valores fixos
  - Removidas métricas de desempenho não testadas (ex: "0ms latency") para manter precisão
  - Atualizadas descrições de event hooks para refletir implementação atual sem detalhes de filtragem desatualizados
  - Aprimorada seção Features com capacidades detalhadas:
    - Destacada configuração independente de cor e opacidade para regiões inativas e ativas
    - Documentadas opções de backend de renderização com características de uso de memória (LayeredWindow < 10MB, CompositeOverlay ~50MB para dual monitor)
    - Adicionadas métricas de pegada de instalação (< 50MB instalado, < 10MB instalador)
    - Enfatizada capacidade de hot-reload e benefícios da arquitetura orientada a eventos

- **Suporte a hot-reload do backend de renderização**: Mudanças no backend de renderização agora aplicam instantaneamente sem necessidade de reiniciar a aplicação
  - Aplicação detecta quando configuração `System.RendererBackend` é alterada
  - Descarta automaticamente renderizador antigo e cria nova instância de renderizador
  - Recria todas as janelas de sobreposição com novo backend de renderização
  - Preserva exclusão de captura de tela e outras configurações durante recriação do renderizador
  - Permite comparação perfeita entre renderizadores LayeredWindow, UpdateLayeredWindow e CompositeOverlay
  - Transições registradas mostram nomes do backend antigo e novo para transparência
  - Consistente com comportamento de hot-reload existente para cores, opacidade e outras configurações

- **Gerenciamento simplificado de propriedade schema**: URL do schema de configuração agora é gerenciada diretamente como propriedade da classe ao invés de injeção pós-serialização
  - Adicionada propriedade `Schema` à classe `AppConfig` com atributo JSON `$schema` para serialização direta
  - Removida classe `SchemaInjector` simplificando codebase ao eliminar manipulação manual de string JSON
  - URL do schema atualiza automaticamente via método `UpdateVersion()` garantindo consistência
  - Atributo `JsonPropertyOrder` garante que `$schema` apareça primeiro no JSON serializado
  - Testes unitários abrangentes adicionados para verificar serialização e desserialização da propriedade schema
  - Arquitetura mais limpa com gerenciamento de schema integrado ao modelo de dados

- **Arquitetura de rastreamento de foco testável**: Lógica de mudança de foco refatorada para camada Core para testes unitários abrangentes
  - Criada interface `IOverlayUpdateService` para abstrair atualizações de sobreposição da lógica de rastreamento de foco
  - Nova classe `FocusChangeHandler` na camada Core contém lógica de decisão independente de plataforma
  - Filtragem de janelas com dimensão zero (janelas 0x0) agora tem cobertura de testes para prevenir regressões
  - Lógica de detecção de mudança de display agora totalmente testável sem dependências do Windows
  - FocusTracker simplificado para adaptador fino que traduz eventos do Windows para objetos de domínio Core
  - Adicionado NSubstitute para mocking em testes - permite verificação de chamadas de atualização de sobreposição
  - Mais de 20 testes unitários abrangentes cobrindo casos extremos: dimensões zero, mudanças de display, mudanças de posição, rastreamento de estado
  - Separação de responsabilidades: WindowsBindings lida com I/O de plataforma, Core lida com lógica de negócio
  - Permite testes futuros de lógica de seleção de janela UWP e outros cenários de rastreamento de foco

- **Notas de release abrangentes com integração de changelog**: Releases do GitHub agora incluem automaticamente informações detalhadas do changelog
  - Criado script `Extract-Changelog.ps1` para extrair seção [Unreleased] do CHANGELOG.md
  - Atualizado workflow de release para injetar conteúdo do changelog no topo das descrições de release
  - Notas de release agora mostram o que mudou em cada versão em inglês e português
  - Corresponde ao formato histórico usado em v0.7.2 e releases anteriores
  - Melhora experiência do usuário tornando mudanças imediatamente visíveis nas páginas de release
  - Mantém fonte única de verdade: CHANGELOG.md direciona tanto documentação quanto notas de release

- **Extração automática de versão no Build-Installer.ps1**: Script de build agora extrai automaticamente a versão do Directory.Build.props
  - Parâmetro de versão agora é opcional - padrão é a versão do Directory.Build.props com sufixo "-dev"
  - Elimina sincronização manual de versão ao construir instaladores locais
  - Exemplo: Com versão 0.8.2 no Directory.Build.props, executar `.\Build-Installer.ps1` cria instalador versão "0.8.2-dev"
  - Ainda pode sobrescrever com versão explícita: `.\Build-Installer.ps1 -Version 1.0.0`
  - Análise XML do PowerShell extrai elemento `<Version>` diretamente do arquivo de versão centralizado
  - Mensagens amigáveis no console indicam versão extraída ou fallback para padrão se arquivo não encontrado

### Adicionado
- **Exclusão de screenshots (EXPERIMENTAL)**: Configuração opcional para excluir janelas de sobreposição de capturas de tela
  - Nova opção de configuração: `ExcludeFromScreenCapture` na seção `Overlay` (padrão: false)
  - Usa a API `SetWindowDisplayAffinity` do Windows com flag `WDA_EXCLUDEFROMCAPTURE`
  - Quando habilitado, sobreposições são excluídas de screenshots tiradas com PrintScreen, Ferramenta de Captura, Greenshot, etc.
  - Checkbox no app de configuração na seção "Experimental Features" com aviso claro
  - Alternável dinamicamente via arquivo de configuração - aplica imediatamente sem reiniciar via hot-reload
  - Fallback gracioso: Se a API falhar (incompatibilidade com janelas em camadas), sobreposições continuam funcionando normalmente
  - Logging de diagnóstico mostra contagem de sucesso/falha quando habilitado
  - **Limitação conhecida**: Pode não funcionar em todos os sistemas Windows devido a restrições da API com janelas `WS_EX_LAYERED`
  - Taxa de sucesso varia por versão do Windows e configuração do sistema (funciona melhor no Windows 10 2004+ e Windows 11)
  - Particularmente útil para criar demos, tutoriais, ou documentação sem artefatos de escurecimento em screenshots

- **Início automático no login**: Nova opção no menu da bandeja do sistema para iniciar automaticamente o SpotlightDimmer quando o Windows inicializa
  - Item de menu com checkbox "Start at Login" no menu de contexto da bandeja do sistema
  - Clicar alterna o início automático ligado/desligado via Registro do Windows (HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run)
  - Menu mostra marca de seleção quando o início automático está habilitado
  - Funciona de forma confiável entre reinicializações do Windows e sessões de usuário
  - Não requer privilégios de administrador - usa chave de registro por usuário
  - Baseado no design comprovado da implementação Rust

- **Acessibilidade via teclado na bandeja do sistema**: Suporte completo de teclado para navegação do ícone da bandeja
  - Pressione **Espaço** para alternar pausar/retomar (igual ao duplo clique)
  - Pressione **Enter**, **tecla Apps**, ou **Shift+F10** para abrir o menu de contexto (igual ao clique direito)
  - Mapeamento intuitivo de teclas: Espaço para alternância rápida, Enter para acesso ao menu
  - Melhora a acessibilidade para usuários que usam apenas teclado
  - Segue as diretrizes de acessibilidade do Windows para interações com a bandeja do sistema
  - Usa `GetAsyncKeyState` para distinguir adequadamente entre as teclas Enter e Espaço

- **Posicionamento aprimorado do menu de contexto**: Menu agora aparece na localização do ícone da bandeja em vez da posição do cursor
  - Usa `Shell_NotifyIconGetRect` para obter coordenadas exatas do ícone da bandeja
  - Menu posicionado no centro-inferior do ícone da bandeja para consistência
  - Cursor permanece no lugar (não é movido para o ícone da bandeja)
  - Retorno à posição do cursor se as coordenadas do ícone não estiverem disponíveis
  - Comportamento profissional correspondente aos aplicativos do sistema Windows

- **Submenu de Diagnósticos na bandeja do sistema**: Novas ferramentas de diagnóstico para solução de problemas e gerenciamento de logs
  - "View Logs Folder" - Abre o diretório de logs no Windows Explorer
  - "View Latest Log" - Abre o arquivo de log mais recente no Notepad (com validação inteligente)
  - "Enable Logging" - Checkbox de alternância para habilitar/desabilitar logging instantaneamente
  - Logs armazenados em `%AppData%\SpotlightDimmer\logs\` com rotação diária
  - Tratamento inteligente de erros: mostra caixa de mensagem se logging desabilitado e não há logs existentes

- **Menu de Ajuda com Sobre e acesso ao GitHub**: Novo submenu Ajuda na bandeja do sistema para informações e suporte do app
  - "About Spotlight Dimmer" - Mostra diálogo com versão, autor (Thomaz Moura), stack de tecnologia (.NET, WinForms, Windows APIs), e URL do GitHub
  - "Visit Github page" - Abre o repositório GitHub do projeto (github.com/thomazmoura/spotlight-dimmer) no navegador padrão
  - Versão extraída automaticamente dos metadados do assembly
  - Interface MessageBox limpa e informativa com ícone apropriado
  - Integração direta com navegador usando API ShellExecute

### Alterado
- **Reorganização do menu da bandeja do sistema**: Estrutura do menu melhorada com agrupamento lógico em Configurações e Ajuda
  - Novo submenu "Settings" consolida configurações e opções do sistema:
    - Configuration... e Open Config File movidos para o topo para fácil acesso
    - Opção Start at Login movida do nível superior
    - Itens de Diagnósticos (View Logs Folder, View Latest Log, Enable Logging) movidos do submenu e achatados em Settings
  - Novo submenu "Help" para suporte e informações:
    - About Spotlight Dimmer
    - Visit Github page
  - Hierarquia de menu simplificada reduz profundidade de aninhamento para melhor acessibilidade
  - Organização mais intuitiva seguindo as convenções de UI do Windows
- **Sistema de logging baseado em arquivos substitui saída de console**: Logging estruturado profissional com Serilog
  - Logs escritos em `%AppData%\SpotlightDimmer\logs\spotlight-YYYY-MM-DD.log`
  - Níveis de log configuráveis: Error, Information, Debug (padrão: Information)
  - Rotação automática diária de logs com retenção configurável (padrão: 7 dias)
  - Suporte a hot-reload - mudanças de logging aplicadas imediatamente sem reiniciar
  - Zero janela de console - aplicação executa silenciosamente quando iniciada da GUI
  - Toda saída de console migrada para logging estruturado com níveis de log conscientes do contexto
  - Propriedades de configuração: `EnableLogging` (bool), `LogLevel` (string), `LogRetentionDays` (int)

### Corrigido
- **Aplicação agora executa sem janela de console**: Alterado de subsistema console para Windows
  - Nenhuma janela de console aparece quando iniciado do Menu Iniciar, atalhos, ou início automático
  - Operação silenciosa com a bandeja do sistema como único componente de interface visível
  - Experiência de usuário profissional correspondente a aplicações Windows modernas

## [0.8.0-beta] - TBD

### Changed
- **Complete .NET 10 rewrite**: Migrated entire codebase from Rust to .NET 10 for improved event-driven architecture
  - Replaced 50-200ms polling with 100% event-driven design using Windows event hooks
  - EVENT_SYSTEM_FOREGROUND for instant focus detection (0ms latency)
  - EVENT_OBJECT_LOCATIONCHANGE for real-time window movement tracking
  - Zero CPU usage when idle - only activates on actual window changes
  - Maintains all previous functionality while improving performance and responsiveness
  - Native AOT compilation support for fast startup and reduced memory footprint

### Added
- **Hot-reloadable JSON configuration**: Configuration changes now apply instantly without restart
  - FileSystemWatcher detects config.json changes in real-time
  - New JSON format replaces TOML for better tooling support
  - Configuration file location: `%AppData%\SpotlightDimmer\config.json`
  - Three dimming modes: FullScreen, Partial, and PartialWithActive
  - Customizable colors and opacity for both active and inactive overlays
  - See CONFIGURATION.md for detailed configuration options

### Improved
- **Zero-allocation hot path**: Eliminates memory allocations during window movement and focus changes
  - Pre-allocated overlay windows (6 per display) created at startup
  - In-place updates using CopyFrom() pattern instead of creating new objects
  - Cached configuration and display info to avoid re-allocation on every event
  - Batch window updates using DeferWindowPos for atomic operations
  - GDI object monitoring in verbose mode to detect potential leaks

- **Memory leak prevention**: Enhanced handle management to prevent resource leaks
  - Proper DeferWindowPos handle cleanup even on failure
  - Pre-allocated brushes for overlay colors (zero allocations during rendering)
  - Comprehensive disposal pattern for all Windows resources

- **Layered architecture**: Clear separation between platform-agnostic logic and Windows-specific code
  - Core layer: Pure C# calculation logic with zero Windows dependencies
  - WindowsBindings layer: Windows API integration using P/Invoke
  - Enables easier testing and potential future cross-platform support

---

### Alterado
- **Reescrita completa em .NET 10**: Migrado toda a base de código de Rust para .NET 10 para arquitetura orientada a eventos aprimorada
  - Substituído polling de 50-200ms por design 100% orientado a eventos usando event hooks do Windows
  - EVENT_SYSTEM_FOREGROUND para detecção instantânea de foco (latência de 0ms)
  - EVENT_OBJECT_LOCATIONCHANGE para rastreamento de movimento de janela em tempo real
  - Zero uso de CPU quando ocioso - apenas ativa em mudanças reais de janela
  - Mantém toda funcionalidade anterior enquanto melhora desempenho e responsividade
  - Suporte a compilação Native AOT para inicialização rápida e redução de footprint de memória

### Adicionado
- **Configuração JSON hot-reload**: Mudanças de configuração agora aplicam instantaneamente sem reinício
  - FileSystemWatcher detecta mudanças em config.json em tempo real
  - Novo formato JSON substitui TOML para melhor suporte de ferramentas
  - Localização do arquivo de configuração: `%AppData%\SpotlightDimmer\config.json`
  - Três modos de escurecimento: FullScreen, Partial e PartialWithActive
  - Cores e opacidade personalizáveis para sobreposições ativas e inativas
  - Veja CONFIGURATION.md para opções detalhadas de configuração

### Melhorado
- **Hot path sem alocações**: Elimina alocações de memória durante movimento de janela e mudanças de foco
  - Janelas de sobreposição pré-alocadas (6 por display) criadas na inicialização
  - Atualizações in-place usando padrão CopyFrom() ao invés de criar novos objetos
  - Configuração e informações de display cacheadas para evitar re-alocação a cada evento
  - Atualizações de janela em lote usando DeferWindowPos para operações atômicas
  - Monitoramento de objetos GDI em modo verbose para detectar vazamentos potenciais

- **Prevenção de vazamento de memória**: Gerenciamento aprimorado de handles para prevenir vazamentos de recursos
  - Limpeza adequada de handle DeferWindowPos mesmo em caso de falha
  - Brushes pré-alocados para cores de sobreposição (zero alocações durante renderização)
  - Padrão abrangente de disposição para todos recursos do Windows

- **Arquitetura em camadas**: Separação clara entre lógica agnóstica de plataforma e código específico do Windows
  - Camada Core: Lógica de cálculo C# pura sem dependências do Windows
  - Camada WindowsBindings: Integração com Windows API usando P/Invoke
  - Possibilita testes mais fáceis e potencial suporte cross-platform futuro

## [0.6.8] - 2025-10-16

## [0.5.6] - 2025-10-14

### Fixed
- **Installer icon visibility**: Fixed missing application icons in desktop shortcuts, Start Menu, and Add/Remove Programs
  - Fixed field naming in packager.toml: cargo-packager uses kebab-case (`installer-icon`) not camelCase (`installerIcon`)
  - Enabled `installer-icon = "spotlight-dimmer-icon.ico"` to set the installer's MUI_ICON (was empty due to incorrect field name)
  - Created custom NSIS template (`custom-installer-template.nsi`) based on cargo-packager default
  - Modified `CreateShortcut` commands to explicitly specify icon file: `"" "$INSTDIR\spotlight-dimmer-icon.ico" 0`
  - NSIS CreateShortcut parameters: link target parameters iconfile iconindex (empty strings required for skipped params)
  - Icons now properly displayed in installer window, desktop shortcuts, Start Menu, and Windows Settings > Apps
  - Previously shortcuts relied only on embedded .exe icon without explicit icon parameter
  - Solution ensures consistent icon display across all Windows versions and UI components

## [0.5.5-beta.11] - 2025-10-14
### Added
- **Professional NSIS installer for Windows**: Proper installation experience with Start Menu integration and uninstaller
  - Per-user installation requiring no administrator privileges
  - Start Menu shortcuts automatically created in "Spotlight Dimmer" folder
  - Application appears in Windows Settings > Apps & features for easy uninstallation
  - Desktop shortcut option during installation
  - Proper Windows Search integration - users can find "Spotlight Dimmer" by typing in Start Menu
  - Embedded application icon in executables for professional appearance in shortcuts, taskbar, and Add/Remove Programs
  - 1.7 MB compressed installer with LZMA compression
  - Built with cargo-packager for modern, automated packaging
  - WinGet compatible installer format for future Windows Package Manager submission

### Improved
- **Single-instance enforcement**: Prevents multiple instances from running simultaneously
  - Uses Windows named mutex (`Global\\SpotlightDimmerSingleInstanceMutex`) for reliable instance detection
  - Shows friendly message box if user tries to launch second instance
  - Prevents overlay conflicts and resource wastage from duplicate processes
  - Works across all launch methods (Start Menu, desktop shortcut, direct executable)
  - Mutex automatically released when application exits

- **Event-driven config file watching**: Configuration file changes now detected instantly via Windows file system notifications
  - Replaced 2-second polling with Windows `FindFirstChangeNotificationW` API for instant detection
  - Config changes applied immediately when file is modified (no polling delay)
  - Zero CPU overhead when config file is not being modified
  - Non-blocking check using `WaitForSingleObject` with 0 timeout
  - Notification automatically re-armed with `FindNextChangeNotification` for continuous monitoring
  - File watching handle properly cleaned up on application exit
  - Technical: Added `fileapi`, `synchapi`, and `winbase` features to winapi dependency

---

### Corrigido
- **Visibilidade de ícone do instalador**: Corrigidos ícones de aplicação ausentes em atalhos da área de trabalho, Menu Iniciar e Adicionar/Remover Programas
  - Corrigida nomenclatura de campo no packager.toml: cargo-packager usa kebab-case (`installer-icon`) não camelCase (`installerIcon`)
  - Habilitado `installer-icon = "spotlight-dimmer-icon.ico"` para definir MUI_ICON do instalador (estava vazio devido ao nome de campo incorreto)
  - Criado template NSIS customizado (`custom-installer-template.nsi`) baseado no padrão do cargo-packager
  - Modificados comandos `CreateShortcut` para especificar explicitamente arquivo de ícone: `"" "$INSTDIR\spotlight-dimmer-icon.ico" 0`
  - Parâmetros NSIS CreateShortcut: link target parameters iconfile iconindex (strings vazias necessárias para params pulados)
  - Ícones agora exibidos adequadamente na janela do instalador, atalhos da área de trabalho, Menu Iniciar e Configurações do Windows > Aplicativos
  - Anteriormente atalhos dependiam apenas de ícone embutido no .exe sem parâmetro de ícone explícito
  - Solução garante exibição consistente de ícone através de todas versões do Windows e componentes de interface

### Adicionado
- **Instalador NSIS profissional para Windows**: Experiência de instalação adequada com integração ao Menu Iniciar e desinstalador
  - Instalação por usuário sem necessidade de privilégios de administrador
  - Atalhos do Menu Iniciar criados automaticamente na pasta "Spotlight Dimmer"
  - Aplicação aparece em Configurações do Windows > Aplicativos e recursos para fácil desinstalação
  - Opção de atalho na área de trabalho durante instalação
  - Integração adequada com Pesquisa do Windows - usuários podem encontrar "Spotlight Dimmer" digitando no Menu Iniciar
  - Ícone da aplicação embutido nos executáveis para aparência profissional em atalhos, barra de tarefas e Adicionar/Remover Programas
  - Instalador comprimido de 1.7 MB com compressão LZMA
  - Construído com cargo-packager para empacotamento moderno e automatizado
  - Formato de instalador compatível com WinGet para futura submissão ao Windows Package Manager

### Melhorado
- **Aplicação de instância única**: Previne múltiplas instâncias de executarem simultaneamente
  - Usa mutex nomeado do Windows (`Global\\SpotlightDimmerSingleInstanceMutex`) para detecção confiável de instância
  - Mostra caixa de mensagem amigável se usuário tentar iniciar segunda instância
  - Previne conflitos de sobreposição e desperdício de recursos de processos duplicados
  - Funciona em todos os métodos de inicialização (Menu Iniciar, atalho da área de trabalho, executável direto)
  - Mutex automaticamente liberado quando aplicação sai

- **Observação de arquivo de configuração orientada a eventos**: Mudanças no arquivo de configuração agora detectadas instantaneamente via notificações do sistema de arquivos do Windows
  - Substituído polling de 2 segundos pela API `FindFirstChangeNotificationW` do Windows para detecção instantânea
  - Mudanças de configuração aplicadas imediatamente quando arquivo é modificado (sem atraso de polling)
  - Zero sobrecarga de CPU quando arquivo de configuração não está sendo modificado
  - Verificação não-bloqueante usando `WaitForSingleObject` com timeout 0
  - Notificação automaticamente rearma com `FindNextChangeNotification` para monitoramento contínuo
  - Handle de observação de arquivo devidamente limpo ao sair da aplicação
  - Técnico: Adicionadas features `fileapi`, `synchapi` e `winbase` à dependência winapi

## [0.5.5-beta.8] - 2025-10-14

## [0.5.5-beta.7] - 2025-10-14

## [0.5.5-beta.6] - 2025-10-14

### Improved
- **Atomic overlay updates for smooth visual rendering**: All overlays now update simultaneously without visual fragmentation
  - Implemented Windows deferred positioning API (`BeginDeferWindowPos` / `DeferWindowPos` / `EndDeferWindowPos`)
  - All partial overlays (top, bottom, left, right) AND active overlay update in a single atomic operation
  - Windows renders all overlay changes in a single frame, eliminating visual "tearing" or desynchronization
  - No more visible lag between individual overlay movements during window dragging
  - Significantly improved visual smoothness and professional appearance
  - Technical: Batch updates reduce Windows message queue pressure and improve rendering consistency

- **Partial dimming performance optimization**: Overlays now reposition instead of recreate during window movement
  - Previously: Destroyed and recreated all partial overlays on every position change (inefficient)
  - Now: Reuses existing overlay windows and repositions them via `SetWindowPos()` (much more efficient)
  - Overlays only recreate when topology changes (e.g., window touches screen edge, display change)
  - Significantly reduced CPU usage and eliminated visual stuttering during window dragging
  - Smoother, more fluid overlay animations when moving or resizing windows
  
- **Partial dimming responsiveness**: Removed drag detection to update overlays immediately on window position or size changes
  - Eliminated 150ms rapid change detection and 200ms stability threshold that delayed overlay updates
  - Overlays now resize and reposition instantly when windows are moved or resized (no waiting for drag to complete)
  - Removed ~100 lines of complex drag detection logic including timing variables and state tracking
  - Simpler, more responsive user experience with instant visual feedback during window manipulation
  - Performance improvement: no timing calculations or state checks on every frame

---

### Melhorado
- **Atualizações atômicas de sobreposição para renderização visual suave**: Todas as sobreposições agora atualizam simultaneamente sem fragmentação visual
  - Implementada API de posicionamento diferido do Windows (`BeginDeferWindowPos` / `DeferWindowPos` / `EndDeferWindowPos`)
  - Todas as sobreposições parciais (topo, fundo, esquerda, direita) E sobreposição ativa atualizam em uma única operação atômica
  - Windows renderiza todas as mudanças de sobreposição em um único quadro, eliminando "tearing" visual ou dessincronização
  - Sem mais atraso visível entre movimentos individuais de sobreposição durante arrasto de janela
  - Suavidade visual significativamente melhorada e aparência profissional
  - Técnico: Atualizações em lote reduzem pressão na fila de mensagens do Windows e melhoram consistência de renderização

- **Otimização de desempenho do escurecimento parcial**: Sobreposições agora reposicionam ao invés de recriar durante movimento de janela
  - Anteriormente: Destruía e recriava todas as sobreposições parciais em cada mudança de posição (ineficiente)
  - Agora: Reutiliza janelas de sobreposição existentes e as reposiciona via `SetWindowPos()` (muito mais eficiente)
  - Sobreposições só são recriadas quando a topologia muda (ex: janela toca borda da tela, mudança de display)
  - Redução significativa no uso de CPU e eliminação de engasgos visuais durante arrasto de janelas
  - Animações de sobreposição mais suaves e fluidas ao mover ou redimensionar janelas

- **Responsividade do escurecimento parcial**: Removida detecção de arrasto para atualizar sobreposições imediatamente em mudanças de posição ou tamanho da janela
  - Eliminado detecção de mudança rápida de 150ms e limiar de estabilidade de 200ms que atrasavam atualizações de sobreposição
  - Sobreposições agora redimensionam e reposicionam instantaneamente quando janelas são movidas ou redimensionadas (sem esperar conclusão do arrasto)
  - Removidas ~100 linhas de lógica complexa de detecção de arrasto incluindo variáveis de tempo e rastreamento de estado
  - Experiência de usuário mais simples e responsiva com feedback visual instantâneo durante manipulação de janelas
  - Melhoria de desempenho: sem cálculos de tempo ou verificações de estado a cada quadro

## [0.5.5-beta.5] - 2025-10-13

## [0.5.5-beta.4] - 2025-10-13

## [0.5.5-beta.3] - 2025-10-13

### Fixed
- **DPI scaling overlay mismatch**: Fixed overlay borders not matching window borders at non-100% display scaling (125%, 150%, etc.)
  - Root cause: Application wasn't declaring DPI awareness to Windows, causing automatic coordinate scaling
  - At 125% scale: Windows applied automatic scaling, but overlays used already-scaled coordinates → double-scaling
  - Solution: Added `SetProcessDpiAwarenessContext` with `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` mode
  - Per-Monitor V2 ensures application receives physical pixels directly from Windows APIs
  - Overlays now align perfectly with window borders at all DPI scales (100%, 125%, 150%, 175%, 200%)
  - Fix also enables per-monitor DPI awareness: each monitor can have different scaling independently
  - DPI awareness set at application startup before any Windows API calls
  - Technical reference: `src/main_new.rs:set_dpi_awareness()` and `src/main_new.rs:108`

---

### Corrigido
- **Incompatibilidade de escala DPI de sobreposição**: Corrigidas bordas de sobreposição não correspondendo às bordas da janela em escalas de display não-100% (125%, 150%, etc.)
  - Causa raiz: Aplicação não estava declarando consciência de DPI ao Windows, causando escalonamento automático de coordenadas
  - Em escala de 125%: Windows aplicava escalonamento automático, mas sobreposições usavam coordenadas já escalonadas → escalonamento duplo
  - Solução: Adicionado `SetProcessDpiAwarenessContext` com modo `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`
  - Per-Monitor V2 garante que aplicação receba pixels físicos diretamente das APIs do Windows
  - Sobreposições agora se alinham perfeitamente com bordas de janela em todas as escalas DPI (100%, 125%, 150%, 175%, 200%)
  - Correção também habilita consciência DPI por monitor: cada monitor pode ter escalonamento diferente independentemente
  - Consciência DPI definida no início da aplicação antes de quaisquer chamadas à API do Windows
  - Referência técnica: `src/main_new.rs:set_dpi_awareness()` e `src/main_new.rs:108`

## [0.5.5-beta.2] - 2025-10-11

## [0.5.5-beta.1] - 2025-10-11

### Improved
- **Message window infrastructure (Phase 2 of event-driven migration)**: Foundation for event-based Windows API communication
  - Created thread-safe HWND wrapper (`MessageWindowHandle`) for cross-thread messaging with safe Send implementation
  - Implemented message-only window using `HWND_MESSAGE` parent for efficient inter-thread communication
  - Message-only windows are more efficient than hidden windows: no z-order, painting, or input processing overhead
  - Added custom message constants (`WM_USER_TEST`) with reserved space for future event types
  - Window procedure logs all received messages for debugging and validation
  - Test infrastructure posts messages every 10 seconds to verify message delivery
  - Automatic cleanup via Drop trait prevents resource leaks
  - Runs alongside existing polling loop with zero functional impact
  - Preparation for event hooks in Phases 3-5 (display changes, foreground window, window movement)
  - Part of 7-phase incremental migration plan to achieve zero CPU usage when idle

---

### Melhorado
- **Infraestrutura de janela de mensagens (Fase 2 da migração orientada a eventos)**: Fundação para comunicação Windows API baseada em eventos
  - Criado wrapper HWND thread-safe (`MessageWindowHandle`) para mensagens cross-thread com implementação Send segura
  - Implementada janela somente de mensagens usando parent `HWND_MESSAGE` para comunicação inter-thread eficiente
  - Janelas somente de mensagens são mais eficientes que janelas ocultas: sem sobrecarga de z-order, pintura ou processamento de entrada
  - Adicionadas constantes de mensagens personalizadas (`WM_USER_TEST`) com espaço reservado para tipos de eventos futuros
  - Procedimento de janela registra todas as mensagens recebidas para depuração e validação
  - Infraestrutura de teste posta mensagens a cada 10 segundos para verificar entrega de mensagens
  - Limpeza automática via trait Drop previne vazamentos de recursos
  - Executa junto ao loop de polling existente com impacto funcional zero
  - Preparação para hooks de eventos nas Fases 3-5 (mudanças de display, janela em primeiro plano, movimento de janela)
  - Parte do plano de migração incremental de 7 fases para alcançar uso zero de CPU quando ocioso

### Improved
- **Message loop optimization (Phase 1 of event-driven migration)**: Enhanced polling efficiency with adaptive sleep and performance metrics
  - Implemented adaptive sleep: 50ms when active for better responsiveness, 200ms when idle for lower CPU usage
  - Added message processing counters to track Windows message frequency
  - Implemented activity tracking that detects user interactions (window changes, tray icon clicks)
  - System automatically adjusts polling interval based on recent activity (2-second window)
  - Added performance metrics logging every 10 seconds showing messages/second throughput
  - Lays foundation for complete event-driven migration in future phases
  - Expected CPU improvement: 5-10% reduction during idle periods
  - Part of 7-phase incremental migration plan to eliminate polling entirely

---

### Melhorado
- **Otimização do loop de mensagens (Fase 1 da migração orientada a eventos)**: Eficiência de polling aprimorada com sleep adaptativo e métricas de performance
  - Implementado sleep adaptativo: 50ms quando ativo para melhor responsividade, 200ms quando ocioso para menor uso de CPU
  - Adicionados contadores de processamento de mensagens para rastrear frequência de mensagens Windows
  - Implementado rastreamento de atividade que detecta interações do usuário (mudanças de janela, cliques no ícone da bandeja)
  - Sistema ajusta automaticamente intervalo de polling baseado em atividade recente (janela de 2 segundos)
  - Adicionado logging de métricas de performance a cada 10 segundos mostrando throughput de mensagens/segundo
  - Estabelece fundação para migração completa orientada a eventos em fases futuras
  - Melhoria esperada de CPU: redução de 5-10% durante períodos ociosos
  - Parte do plano de migração incremental de 7 fases para eliminar polling completamente

### Added
- **Event-driven architecture research and planning**: Comprehensive analysis of migrating from polling to event-driven Windows API
  - Created detailed polling-to-events migration report analyzing all 7 polling dependencies
  - Ranked migration feasibility for each feature (display hotplug: 5/5, foreground changes: 4/5, window movement: 3/5, etc.)
  - Documented expected performance improvements (100% CPU reduction when idle, up to 100x faster event latency)
  - Created incremental refactor plan with 7 phases to ensure zero-downtime migration
  - Identified HWND threading safety challenges and proposed solutions for Rust ownership model
  - Estimated total effort: 12-24 hours for complete migration, 2-4 hours for Phase 1 quick-wins
  - Reports saved in `.docs/` directory: `polling-to-events-migration-report.md` and `incremental-refactor-plan.md`

---

### Adicionado
- **Pesquisa e planejamento de arquitetura orientada a eventos**: Análise abrangente da migração de polling para Windows API orientado a eventos
  - Criado relatório detalhado de migração polling-para-eventos analisando todas as 7 dependências de polling
  - Classificada viabilidade de migração para cada funcionalidade (hotplug de display: 5/5, mudanças de foco: 4/5, movimento de janela: 3/5, etc.)
  - Documentadas melhorias de performance esperadas (redução de 100% de CPU quando ocioso, latência de evento até 100x mais rápida)
  - Criado plano de refatoração incremental com 7 fases para garantir migração sem tempo de inatividade
  - Identificados desafios de segurança de threading HWND e propostas de soluções para modelo de ownership do Rust
  - Esforço total estimado: 12-24 horas para migração completa, 2-4 horas para vitórias rápidas da Fase 1
  - Relatórios salvos no diretório `.docs/`: `polling-to-events-migration-report.md` e `incremental-refactor-plan.md`

## [0.5.4] - 2025-10-08

### Fixed
- **Cargo publish compilation failure**: Fixed publish-cargo job failing on Linux by switching to windows-latest runner
  - cargo publish must compile Windows-specific code during verification step
  - Cannot use pre-built artifacts since crates.io publishes source code, not binaries
  - Now builds successfully with Windows API dependencies (std::os::windows, winapi)

---

### Corrigido
- **Falha de compilação do cargo publish**: Corrigido job publish-cargo falhando no Linux ao mudar para runner windows-latest
  - cargo publish deve compilar código específico do Windows durante etapa de verificação
  - Não pode usar artefatos pré-construídos pois crates.io publica código-fonte, não binários
  - Agora compila com sucesso com dependências da API do Windows (std::os::windows, winapi)

## [0.5.3] - 2025-10-08

### Changed
- **Release workflow artifact sharing**: Binaries now built once and reused across all publishing targets
  - Build happens once in build-and-release job on windows-latest
  - Binaries uploaded as GitHub Actions artifacts with 1-day retention
  - publish-npm job downloads pre-built binaries instead of rebuilding
  - Guarantees identical binaries in GitHub Releases and npm package
  - Saves ~1-2 minutes of CI time per release by eliminating duplicate builds for npm
  - Removed redundant Rust toolchain installation from publish-npm job
  - Note: publish-cargo still rebuilds (required by crates.io verification)

---

### Alterado
- **Compartilhamento de artefatos no workflow de release**: Binários agora construídos uma vez e reutilizados em todos os alvos de publicação
  - Build acontece uma vez no job build-and-release em windows-latest
  - Binários enviados como artefatos GitHub Actions com retenção de 1 dia
  - Job publish-npm baixa binários pré-construídos ao invés de reconstruir
  - Garante binários idênticos em GitHub Releases e pacote npm
  - Economiza ~1-2 minutos de tempo de CI por release ao eliminar builds duplicados para npm
  - Removida instalação redundante de toolchain Rust do job publish-npm
  - Nota: publish-cargo ainda reconstrói (requerido pela verificação do crates.io)

## [0.5.2] - 2025-10-08

## [0.5.1] - 2025-10-08

### Changed
- **Release workflow fail-fast publishing**: Publishing to crates.io and npm now fails loudly instead of silently skipping
  - Removed `continue-on-error: true` from both publish-cargo and publish-npm jobs
  - Removed silent token checks that would skip publishing with warnings
  - Release will now fail immediately if CARGO_TOKEN or NPM_TOKEN secrets are missing
  - Ensures publishing issues are caught and fixed rather than silently ignored
  - No more mystery "why didn't this publish?" situations

### Added
- **Automatic git hooks with cargo-husky**: Zero-configuration pre-commit hooks that install themselves automatically
  - Added `cargo-husky` as dev dependency with "user-hooks" feature
  - Hooks stored in `.cargo-husky/hooks/` directory (version-controlled and committed to git)
  - Automatically installed to `.git/hooks/` when running `cargo test`, `cargo build`, or `cargo check`
  - **No manual setup required** - developers just clone and build as normal
  - All team members automatically get identical hooks from the repository
  - Hook updates propagate automatically on next build
- **Pre-commit code quality enforcement**: Automatic checks before every commit
  - Runs `cargo fmt --check` to verify code formatting compliance
  - Runs `cargo clippy --all-targets --all-features` to catch common mistakes and enforce best practices
  - Clear error messages guide developers to fix issues before committing
  - Fast local feedback loop - catch issues before pushing to CI
  - Prevents "fix formatting" and "fix clippy" commits by enforcing standards upfront

### Added
- **Automated multi-platform publishing**: Fully automated release workflow publishes to both crates.io and npm with a single git tag
  - Push git tag (e.g., `v0.4.8`) triggers automated publishing to crates.io, npm, and GitHub Releases
  - **crates.io automation**: `cargo publish` runs automatically on GitHub Actions for Rust users
  - **npm automation**: Pre-built binaries packaged and published to npm for Node.js users
  - Single source of truth: version managed in `package.json` and `Cargo.toml`, published everywhere automatically
  - Requires `CARGO_TOKEN` and `NPM_TOKEN` GitHub secrets (one-time setup)
- **npm package support with pre-built binaries**: Spotlight Dimmer can now be installed via npm with zero compilation required
  - Global installation via `npm install -g spotlight-dimmer` provides instant access to both binaries
  - Pre-built Windows executables included in package - no Rust toolchain required for users
  - Automated publishing via GitHub Actions: binaries built on CI and bundled into npm package
  - Command wrappers enable seamless execution: `spotlight-dimmer` and `spotlight-dimmer-config` work from any directory
  - Platform verification ensures package only installs on Windows x64 (with helpful error messages for other platforms)
  - Clean uninstallation via `npm uninstall -g spotlight-dimmer` automatically stops running instances
  - Icon files automatically bundled for proper system tray functionality
  - Package published to npm registry for worldwide distribution and easy discovery
  - Three distribution channels (npm, crates.io, GitHub Releases) all updated automatically

### Fixed
- **CI/CD pipeline failures**: Fixed all code quality issues causing build failures in GitHub Actions
  - Applied `cargo fmt` to all source files to comply with Rust standard formatting rules
  - Resolved formatting issues in `build.rs`, `src/config.rs`, `src/config_cli.rs`, `src/main_new.rs`, `src/overlay.rs`, `src/platform/mod.rs`, `src/platform/windows.rs`, and `src/tray.rs`
  - Fixed clippy warnings about `field_reassign_with_default` in test code by using struct initialization syntax
  - All CI checks now pass: formatting (`cargo fmt --check`), linting (`cargo clippy`), and tests (`cargo test`)

---

### Alterado
- **Publicação fail-fast no workflow de release**: Publicação para crates.io e npm agora falha imediatamente ao invés de pular silenciosamente
  - Removido `continue-on-error: true` dos jobs publish-cargo e publish-npm
  - Removidas verificações silenciosas de token que pulariam publicação com avisos
  - Release agora falhará imediatamente se secrets CARGO_TOKEN ou NPM_TOKEN estiverem ausentes
  - Garante que problemas de publicação sejam detectados e corrigidos ao invés de silenciosamente ignorados
  - Sem mais situações misteriosas de "por que isso não foi publicado?"

### Adicionado
- **Hooks git automáticos com cargo-husky**: Hooks pre-commit de configuração zero que se instalam automaticamente
  - Adicionado `cargo-husky` como dependência de desenvolvimento com feature "user-hooks"
  - Hooks armazenados no diretório `.cargo-husky/hooks/` (versionados e commitados no git)
  - Instalados automaticamente em `.git/hooks/` ao executar `cargo test`, `cargo build` ou `cargo check`
  - **Sem necessidade de configuração manual** - desenvolvedores apenas clonam e constroem normalmente
  - Todos os membros da equipe automaticamente recebem hooks idênticos do repositório
  - Atualizações de hooks propagam automaticamente no próximo build
- **Aplicação de qualidade de código pre-commit**: Verificações automáticas antes de cada commit
  - Executa `cargo fmt --check` para verificar conformidade de formatação do código
  - Executa `cargo clippy --all-targets --all-features` para detectar erros comuns e aplicar melhores práticas
  - Mensagens de erro claras orientam desenvolvedores a corrigir problemas antes de fazer commit
  - Loop de feedback local rápido - detectar problemas antes de enviar para CI
  - Previne commits "corrigir formatação" e "corrigir clippy" ao aplicar padrões antecipadamente

### Adicionado
- **Publicação automatizada multiplataforma**: Workflow de release totalmente automatizado publica para crates.io e npm com uma única tag git
  - Push de tag git (ex: `v0.4.8`) dispara publicação automatizada para crates.io, npm e GitHub Releases
  - **Automação crates.io**: `cargo publish` executado automaticamente no GitHub Actions para usuários Rust
  - **Automação npm**: Binários pré-compilados empacotados e publicados no npm para usuários Node.js
  - Fonte única de verdade: versão gerenciada em `package.json` e `Cargo.toml`, publicada em todo lugar automaticamente
  - Requer secrets GitHub `CARGO_TOKEN` e `NPM_TOKEN` (configuração única)
- **Suporte a pacote npm com binários pré-compilados**: Spotlight Dimmer agora pode ser instalado via npm sem necessidade de compilação
  - Instalação global via `npm install -g spotlight-dimmer` fornece acesso instantâneo a ambos os binários
  - Executáveis Windows pré-compilados incluídos no pacote - sem necessidade de ferramentas Rust para usuários
  - Publicação automatizada via GitHub Actions: binários compilados no CI e empacotados no pacote npm
  - Wrappers de comando permitem execução perfeita: `spotlight-dimmer` e `spotlight-dimmer-config` funcionam de qualquer diretório
  - Verificação de plataforma garante que o pacote só instale no Windows x64 (com mensagens de erro úteis para outras plataformas)
  - Desinstalação limpa via `npm uninstall -g spotlight-dimmer` para automaticamente instâncias em execução
  - Arquivos de ícone automaticamente empacotados para funcionalidade adequada da bandeja do sistema
  - Pacote publicado no registro npm para distribuição mundial e descoberta fácil
  - Três canais de distribuição (npm, crates.io, GitHub Releases) todos atualizados automaticamente

### Corrigido
- **Falhas no pipeline CI/CD**: Corrigidos todos os problemas de qualidade de código causando falhas de build no GitHub Actions
  - Aplicado `cargo fmt` a todos os arquivos fonte para conformidade com regras de formatação padrão do Rust
  - Resolvidos problemas de formatação em `build.rs`, `src/config.rs`, `src/config_cli.rs`, `src/main_new.rs`, `src/overlay.rs`, `src/platform/mod.rs`, `src/platform/windows.rs` e `src/tray.rs`
  - Corrigidos avisos do clippy sobre `field_reassign_with_default` no código de teste usando sintaxe de inicialização de struct
  - Todas as verificações CI agora passam: formatação (`cargo fmt --check`), linting (`cargo clippy`) e testes (`cargo test`)

## [0.4.5] - 2025-10-05

### Added
- **crates.io publication**: Published spotlight-dimmer to crates.io for easy installation via `cargo install spotlight-dimmer`
- **MIT LICENSE file**: Added standard MIT license file to the repository
- **Cross-platform compilation support**: Added `#[cfg(windows)]` guards to enable compilation on non-Windows platforms (binaries won't run, but package verification works)

### Changed
- **Cargo.toml metadata**: Added `readme` and `homepage` fields for better crates.io presentation
- **Platform-specific code organization**: Refactored to use module-level `#![cfg(windows)]` instead of item-level guards for cleaner code

### Improved
- **Code readability**: Replaced hundreds of individual `#[cfg(windows)]` attributes with clean module-level guards in `overlay.rs` and `tray.rs`

---

### Adicionado
- **Publicação no crates.io**: Publicado spotlight-dimmer no crates.io para instalação fácil via `cargo install spotlight-dimmer`
- **Arquivo LICENSE MIT**: Adicionado arquivo de licença MIT padrão ao repositório
- **Suporte a compilação multiplataforma**: Adicionadas guardas `#[cfg(windows)]` para habilitar compilação em plataformas não-Windows (binários não executam, mas verificação do pacote funciona)

### Alterado
- **Metadados do Cargo.toml**: Adicionados campos `readme` e `homepage` para melhor apresentação no crates.io
- **Organização de código específico de plataforma**: Refatorado para usar `#![cfg(windows)]` no nível do módulo ao invés de guardas no nível do item para código mais limpo

### Melhorado
- **Legibilidade do código**: Substituídas centenas de atributos `#[cfg(windows)]` individuais por guardas limpas no nível do módulo em `overlay.rs` e `tray.rs`

## [0.4.4] - 2025-10-05

### Fixed
- **Dev container configuration**: Updated development container setup to match current pure Rust architecture
  - Removed obsolete Tauri CLI installation that was causing build failures with edition2024 Rust features
  - Removed Node.js and npm installation (no longer needed after Tauri removal)
  - Removed PowerShell package (not available in Ubuntu 22.04 default repos, not needed for Rust-only development)
  - Updated Rust version from 1.77.2 to 1.83.0 for latest language features and edition2024 support
  - Removed all Tauri-specific dependencies (webkit2gtk, GTK, AppIndicator libraries)
  - Added rustfmt, clippy, and rust-analyzer components for better development experience
  - Updated port forwarding configuration (removed Tauri/frontend dev server ports 1420/1430)
  - Removed TypeScript/Prettier/ESLint extensions and replaced with Rust-focused extensions
  - Updated post-create script to build pure Rust binaries instead of Tauri application
  - Added convenient shell aliases: build, build-debug, test, lint, fmt
  - Configuration now matches the lightweight pure Windows API implementation

---

### Corrigido
- **Configuração do container de desenvolvimento**: Atualizada configuração do container de desenvolvimento para corresponder à arquitetura Rust pura atual
  - Removida instalação obsoleta do Tauri CLI que estava causando falhas de build com recursos edition2024 do Rust
  - Removida instalação de Node.js e npm (não mais necessária após remoção do Tauri)
  - Removido pacote PowerShell (não disponível nos repos padrão do Ubuntu 22.04, não necessário para desenvolvimento somente Rust)
  - Atualizada versão do Rust de 1.77.2 para 1.83.0 para recursos mais recentes da linguagem e suporte a edition2024
  - Removidas todas as dependências específicas do Tauri (webkit2gtk, GTK, bibliotecas AppIndicator)
  - Adicionados componentes rustfmt, clippy e rust-analyzer para melhor experiência de desenvolvimento
  - Atualizada configuração de encaminhamento de portas (removidas portas 1420/1430 do servidor de desenvolvimento Tauri/frontend)
  - Removidas extensões TypeScript/Prettier/ESLint e substituídas por extensões focadas em Rust
  - Atualizado script post-create para construir binários Rust puros ao invés de aplicação Tauri
  - Adicionados aliases convenientes de shell: build, build-debug, test, lint, fmt
  - Configuração agora corresponde à implementação leve com Windows API pura

## [0.4.3] - 2025-10-03

### Changed
- **Active overlay behavior with partial dimming**: Active overlay now intelligently resizes based on window state when partial dimming is enabled
  - Windowed mode: Active overlay resizes to match the exact size and position of the focused window (touching the edges of inactive overlays)
  - Maximized/Fullscreen mode: Active overlay covers the entire display as before
  - During drag operations: Active overlay temporarily returns to full screen for smooth performance
  - After drag ends: Active overlay automatically resizes to match final window position
  - Eliminates border darkening issue where both active and inactive overlays overlapped at window edges
  - Window state detection uses `IsZoomed()` API and monitor bounds checking with 10-pixel tolerance
  - Feature automatically activates when both partial dimming and active overlays are enabled

---

### Alterado
- **Comportamento de sobreposição ativa com escurecimento parcial**: Sobreposição ativa agora redimensiona inteligentemente baseado no estado da janela quando escurecimento parcial está habilitado
  - Modo janela: Sobreposição ativa redimensiona para corresponder exatamente ao tamanho e posição da janela focada (tocando as bordas das sobreposições inativas)
  - Modo maximizado/tela cheia: Sobreposição ativa cobre toda a tela como antes
  - Durante operações de arrasto: Sobreposição ativa retorna temporariamente para tela cheia para performance suave
  - Após o arrasto terminar: Sobreposição ativa redimensiona automaticamente para corresponder à posição final da janela
  - Elimina problema de escurecimento de borda onde sobreposições ativas e inativas se sobrepunham nas bordas da janela
  - Detecção de estado da janela usa API `IsZoomed()` e verificação de limites do monitor com tolerância de 10 pixels
  - Funcionalidade ativa automaticamente quando escurecimento parcial e sobreposições ativas estão habilitadas

## [0.4.2] - 2025-10-02

### Fixed
- **Release packaging**: Include icon files in GitHub release ZIP archives
  - Added `spotlight-dimmer-icon.ico` and `spotlight-dimmer-icon-paused.ico` to release bundle
  - Updated release notes to mention icon files are required
  - Icons must be in the same directory as executables for system tray to work

---

### Corrigido
- **Empacotamento de lançamento**: Incluir arquivos de ícone nos arquivos ZIP de lançamento do GitHub
  - Adicionados `spotlight-dimmer-icon.ico` e `spotlight-dimmer-icon-paused.ico` ao pacote de lançamento
  - Notas de lançamento atualizadas para mencionar que arquivos de ícone são necessários
  - Ícones devem estar no mesmo diretório que os executáveis para a bandeja do sistema funcionar

## [0.4.1] - 2025-10-02

### Fixed
- **Profile system stability**: Fixed critical bugs preventing profile switching from working correctly
  - Fixed config reload logic that used `else if` chains preventing multiple changes from being applied
  - Added active window detection fallback when `last_display_id` is None (at startup or early switches)
  - Overlays now properly update visibility after recreation, preventing them from appearing on active display
  - Individual config changes and `set-profile` command now work identically
  - Tray menu profile switching now works correctly without breaking overlays
- **Default profile configurations**: Corrected default profile settings for better user experience
  - light-mode: Now properly enables inactive overlays (was incorrectly disabled)
  - dark-mode: Now properly enables active overlays (was incorrectly disabled)
  - Both profiles now have sensible defaults that work out of the box

### Technical Details
- Added `update_inactive_color_only()` and `update_active_color_only()` methods to OverlayManager
- Visibility updates now query window manager when cached display_id is unavailable
- Config reload logic now handles multiple simultaneous changes correctly
- Added automatic default profile loading for configs from v0.3.0

---

### Corrigido
- **Estabilidade do sistema de perfis**: Corrigidos bugs críticos impedindo a troca de perfis de funcionar corretamente
  - Corrigida lógica de recarga de configuração que usava cadeias `else if` impedindo múltiplas mudanças de serem aplicadas
  - Adicionado fallback de detecção de janela ativa quando `last_display_id` é None (na inicialização ou trocas precoces)
  - Sobreposições agora atualizam visibilidade corretamente após recriação, prevenindo aparecimento na tela ativa
  - Mudanças individuais de configuração e comando `set-profile` agora funcionam identicamente
  - Troca de perfis via menu da bandeja agora funciona corretamente sem quebrar sobreposições
- **Configurações de perfis padrão**: Corrigidas configurações de perfis padrão para melhor experiência
  - light-mode: Agora habilita corretamente sobreposições inativas (estava incorretamente desabilitado)
  - dark-mode: Agora habilita corretamente sobreposições ativas (estava incorretamente desabilitado)
  - Ambos os perfis agora têm padrões sensatos que funcionam de imediato

## [0.4.0] - 2025-10-02

## [0.4.0] - 2025-10-01
### Added
- **Profile management system**: Save and switch between different overlay configurations
  - Save current settings as named profiles with `save-profile <name>` command
  - Load saved profiles instantly with `set-profile <name>` command
  - List all available profiles with `list-profiles` command
  - Delete unwanted profiles with `delete-profile <name>` command
  - System tray menu integration: Right-click tray icon to see and switch profiles
  - Two default profiles included: "light-mode" and "dark-mode"
  - Each profile stores: overlay color, dimming state, active overlay settings, and partial dimming preferences
  - Perfect for switching between different work modes, lighting conditions, or personal preferences
  - Zero performance impact: Profile storage uses lightweight TOML serialization in config file

---

### Adicionado
- **Sistema de gerenciamento de perfis**: Salve e alterne entre diferentes configurações de sobreposição
  - Salve configurações atuais como perfis nomeados com comando `save-profile <nome>`
  - Carregue perfis salvos instantaneamente com comando `set-profile <nome>`
  - Liste todos os perfis disponíveis com comando `list-profiles`
  - Exclua perfis indesejados com comando `delete-profile <nome>`
  - Integração com menu da bandeja do sistema: Clique com botão direito no ícone da bandeja para ver e alternar perfis
  - Dois perfis padrão incluídos: "light-mode" e "dark-mode"
  - Cada perfil armazena: cor de sobreposição, estado de escurecimento, configurações de sobreposição ativa e preferências de escurecimento parcial
  - Perfeito para alternar entre diferentes modos de trabalho, condições de iluminação ou preferências pessoais
  - Zero impacto de performance: Armazenamento de perfis usa serialização TOML leve no arquivo de configuração

## [0.3.0] - 2025-10-01

## [0.3.0] - 2025-10-01

### Added
- **Partial dimming mode**: New feature that dims empty areas around the focused window on the active display
  - Intelligently detects window position and creates up to 4 overlays (top, bottom, left, right) for gaps between window edges and display edges
  - Full-screen windows: No overlays (window covers entire display)
  - Docked windows: Single overlay on opposite side (e.g., window docked right → overlay on left)
  - Windowed mode: Multiple overlays for each exposed edge (e.g., centered window → 4 overlays)
  - Uses inactive overlay color for consistent visual experience
  - Edge detection with 5-pixel tolerance for precise alignment
  - Automatically updates overlays when window moves, resizes, or switches between displays
  - Smart corner handling prevents overlaps (horizontal overlays span full width, vertical overlays adjusted to prevent double-dimming)
  - Drag detection automatically hides overlays during window dragging to prevent flickering
  - Overlays recreate instantly when drag operation completes (200ms stability threshold)
  - Enable with `spotlight-dimmer-config partial-enable`
  - Can be combined with inactive display dimming and active display overlay for maximum focus
  - Zero performance impact when disabled

---

### Adicionado
- **Modo de escurecimento parcial**: Nova funcionalidade que escurece áreas vazias ao redor da janela focada no display ativo
  - Detecta inteligentemente a posição da janela e cria até 4 sobreposições (topo, fundo, esquerda, direita) para lacunas entre as bordas da janela e do display
  - Janelas em tela cheia: Sem sobreposições (janela cobre todo o display)
  - Janelas ancoradas: Sobreposição única no lado oposto (ex: janela ancorada à direita → sobreposição à esquerda)
  - Modo janela: Múltiplas sobreposições para cada borda exposta (ex: janela centralizada → 4 sobreposições)
  - Usa cor de sobreposição inativa para experiência visual consistente
  - Detecção de borda com tolerância de 5 pixels para alinhamento preciso
  - Atualiza automaticamente sobreposições quando a janela move, redimensiona ou muda entre displays
  - Gestão inteligente de cantos previne sobreposições (sobreposições horizontais cobrem largura total, verticais ajustadas para prevenir escurecimento duplo)
  - Detecção de arraste oculta automaticamente sobreposições durante movimento de janela para prevenir tremulação
  - Sobreposições recriam instantaneamente quando operação de arraste completa (limiar de estabilidade de 200ms)
  - Habilite com `spotlight-dimmer-config partial-enable`
  - Pode ser combinado com escurecimento de displays inativos e sobreposição de display ativo para máximo foco
  - Zero impacto de performance quando desabilitado

## [0.2.1] - 2025-10-01

## [0.2.0] - 2025-09-30

### Added
- **System tray icon**: Application now includes a system tray icon with context menu for easy management
  - Icon appears in Windows system tray when application is running
  - Right-click menu provides "Exit" option for graceful shutdown
  - Eliminates need to use Task Manager to close the application
  - Application exits cleanly, properly removing all overlays and tray icon
  - Uses pure Windows API (`Shell_NotifyIconW`) maintaining minimal binary size (~10 MB RAM)
  - Icon integrated with existing message loop architecture for seamless operation
  - Works with shortcuts and any launch method (uses `GetModuleFileNameW` to find icon relative to executable)

### Fixed
- **Ghost window and crash prevention**: Removed unnecessary drag detection workaround, ghost window issue resolved by message loop
  - Previous version (0.1.14) implemented complex mouse drag detection to prevent ghost windows and crashes
  - Discovery: The `process_windows_messages()` function added for tray icon support actually fixed the root cause
  - Proper Windows message processing (`PeekMessageW`/`DispatchMessageW`) prevents race conditions with overlay visibility changes
  - Removed all drag detection code (100+ lines including title bar hit testing) - no longer needed
  - Application now works flawlessly without any special handling for window dragging
  - Result: Simpler codebase, no ghost windows, no crashes, no display flashing, instant overlay updates during all operations
  - This demonstrates the importance of proper Windows message loop integration for GUI applications

---

### Adicionado
- **Ícone na bandeja do sistema**: A aplicação agora inclui um ícone na bandeja do sistema com menu de contexto para gerenciamento fácil
  - Ícone aparece na bandeja do sistema do Windows quando a aplicação está rodando
  - Menu de botão direito fornece opção "Exit" para encerramento gracioso
  - Elimina necessidade de usar o Gerenciador de Tarefas para fechar a aplicação
  - Aplicação encerra de forma limpa, removendo adequadamente todas as sobreposições e ícone da bandeja
  - Usa Windows API puro (`Shell_NotifyIconW`) mantendo tamanho binário mínimo (~10 MB RAM)
  - Ícone integrado com arquitetura de loop de mensagens existente para operação perfeita
  - Funciona com atalhos e qualquer método de inicialização (usa `GetModuleFileNameW` para encontrar ícone relativo ao executável)

### Corrigido
- **Prevenção de janelas fantasmas e crashes**: Removido workaround desnecessário de detecção de arraste, problema de janelas fantasmas resolvido pelo loop de mensagens
  - Versão anterior (0.1.14) implementou detecção complexa de arraste do mouse para prevenir janelas fantasmas e crashes
  - Descoberta: A função `process_windows_messages()` adicionada para suporte ao ícone da bandeja corrigiu a causa raiz
  - Processamento adequado de mensagens do Windows (`PeekMessageW`/`DispatchMessageW`) previne condições de corrida com mudanças de visibilidade de sobreposição
  - Removido todo código de detecção de arraste (100+ linhas incluindo teste de acerto de barra de título) - não mais necessário
  - Aplicação agora funciona perfeitamente sem qualquer tratamento especial para arraste de janelas
  - Resultado: Base de código mais simples, sem janelas fantasmas, sem crashes, sem piscadas de display, atualizações instantâneas de sobreposição durante todas operações
  - Isto demonstra a importância da integração adequada do loop de mensagens do Windows para aplicações GUI

## [0.1.14] - 2025-09-30

### Fixed
- **Critical stability fix**: Resolved system crash and eliminated black "ghost windows" when dragging windows between monitors with mouse
  - Root cause: Rapid `ShowWindow()` calls on layered topmost windows during active drag operations created race conditions with Windows' drag-and-drop message loop
  - Windows created black "ghost windows" when it couldn't deliver messages to overlays during drag operations
  - Solution: Implemented intelligent drag detection using `GetAsyncKeyState(VK_LBUTTON)` to detect when left mouse button is held down
  - All overlays are immediately hidden when mouse button is pressed (preventing ghost window creation)
  - Overlays remain hidden during entire drag operation (clean user experience, no black screens)
  - Overlays restore with correct visibility when mouse button is released (drag operation completes)
  - Normal focus changes and keyboard shortcuts (Win+Arrow) maintain instant responsiveness with immediate overlay updates
  - Result: Stable operation during window dragging, no crashes, no black ghost windows, instant response for all other operations

---

### Corrigido
- **Correção crítica de estabilidade**: Resolvido crash do sistema e eliminadas "janelas fantasmas" pretas ao arrastar janelas entre monitores com o mouse
  - Causa raiz: Chamadas rápidas de `ShowWindow()` em janelas topmost em camadas durante operações de arrastar ativas criavam condições de corrida com o loop de mensagens de arrastar e soltar do Windows
  - O Windows criava "janelas fantasmas" pretas quando não conseguia entregar mensagens às sobreposições durante operações de arraste
  - Solução: Implementada detecção inteligente de arraste usando `GetAsyncKeyState(VK_LBUTTON)` para detectar quando o botão esquerdo do mouse está pressionado
  - Todas as sobreposições são imediatamente ocultadas quando o botão do mouse é pressionado (prevenindo criação de janelas fantasmas)
  - Sobreposições permanecem ocultas durante toda a operação de arraste (experiência limpa do usuário, sem telas pretas)
  - Sobreposições restauram com visibilidade correta quando o botão do mouse é solto (operação de arraste é concluída)
  - Mudanças de foco normais e atalhos de teclado (Win+Seta) mantêm responsividade instantânea com atualizações imediatas de sobreposição
  - Resultado: Operação estável durante arraste de janelas, sem crashes, sem janelas fantasmas pretas, resposta instantânea para todas as outras operações

## [0.1.13] - 2025-09-30

## [0.1.12] - 2025-09-30

### Added
- **Smart console behavior**: Application now intelligently manages console window visibility
  - Shows console output with logs when launched from terminal (Command Prompt, PowerShell, etc.)
  - Hides console window when launched from Start Menu, desktop shortcuts, or GUI
  - Uses runtime detection via `GetConsoleProcessList()` to determine launch context
  - Seamless user experience: CLI-friendly when needed, GUI-silent otherwise
- **Active display overlay**: New optional overlay that highlights the active display instead of dimming inactive ones
  - Independent enable/disable control via `active-enable` and `active-disable` commands
  - Configurable color and opacity via `active-color <r> <g> <b> [a]` command
  - Can be used alone, with inactive dimming, or both simultaneously
  - Use cases: Highlight active display with subtle color, or dim both active/inactive with different intensities
  - Default subtle blue highlight (RGB 50, 100, 255, alpha 0.15) when enabled without color configuration
- **Dual overlay system**: Both inactive (dimming) and active (highlighting) overlays can now run simultaneously
  - Inactive overlays dim non-active displays (traditional behavior)
  - Active overlays highlight the currently active display
  - Each overlay type independently managed with separate Windows handles
  - Real-time visibility updates for both overlay types on window focus changes
- **CLI commands for active overlay management**:
  - `active-enable`: Enable active display overlay highlighting
  - `active-disable`: Disable active display overlay highlighting
  - `active-color <r> <g> <b> [a]`: Set active overlay color and opacity
  - Enhanced `status` command now displays both inactive and active overlay configurations
  - Enhanced help text with comprehensive usage examples for all overlay modes

### Fixed
- **Color bleeding between overlay types**: Fixed bug where moving windows between displays would cause inactive overlays to adopt active overlay colors
  - Root cause: Both overlay types shared the same Windows window class, causing `SetClassLongPtrW(GCLP_HBRBACKGROUND)` to affect all windows
  - Solution: Implemented separate window classes (`SpotlightDimmerInactiveOverlay` and `SpotlightDimmerActiveOverlay`) for each overlay type
  - Each window class now maintains its own independent background brush, preventing color cross-contamination
- Project structure: Removed duplicate `src/src/` nesting - now follows standard Cargo convention with `Cargo.toml` at root and source files in `src/`
- GitHub Actions workflow: Updated to use pure Cargo build instead of Tauri action
- GitHub Actions: Changed from NSIS installers to ZIP archives for simpler, more reliable distribution
- Documentation: Corrected all build paths to reflect new structure (removed `cd src` steps)

### Improved
- Configuration workflow: No longer need to restart the application after changing settings with `spotlight-dimmer-config.exe`
- User experience: Settings changes are now nearly instantaneous (2-second detection window)
- Resource efficiency: Config monitoring adds only 16 bytes of memory and <0.01% CPU overhead
- Overlay architecture: Refactored to support multiple overlay types with independent lifecycle management
- Configuration reload logic: Now handles both inactive and active overlay changes independently
- Display hotplug handling: Both overlay types properly recreated when displays are connected/disconnected

---

### Adicionado
- **Comportamento inteligente de console**: A aplicação agora gerencia inteligentemente a visibilidade da janela de console
  - Mostra saída do console com logs quando iniciado a partir do terminal (Prompt de Comando, PowerShell, etc.)
  - Oculta a janela do console quando iniciado a partir do Menu Iniciar, atalhos da área de trabalho ou GUI
  - Usa detecção em tempo de execução via `GetConsoleProcessList()` para determinar o contexto de inicialização
  - Experiência do usuário perfeita: amigável para CLI quando necessário, silencioso na GUI caso contrário
- **Sobreposição de display ativo**: Nova sobreposição opcional que destaca o display ativo ao invés de escurecer os inativos
  - Controle independente de ativar/desativar via comandos `active-enable` e `active-disable`
  - Cor e opacidade configuráveis via comando `active-color <r> <g> <b> [a]`
  - Pode ser usado sozinho, com escurecimento inativo, ou ambos simultaneamente
  - Casos de uso: Destacar display ativo com cor sutil, ou escurecer ambos ativo/inativo com intensidades diferentes
  - Destaque azul sutil padrão (RGB 50, 100, 255, alpha 0.15) quando habilitado sem configuração de cor
- **Sistema de sobreposição dupla**: Sobreposições inativas (escurecimento) e ativas (destaque) agora podem rodar simultaneamente
  - Sobreposições inativas escurecem displays não-ativos (comportamento tradicional)
  - Sobreposições ativas destacam o display atualmente ativo
  - Cada tipo de sobreposição gerenciado independentemente com handles Windows separados
  - Atualizações de visibilidade em tempo real para ambos os tipos de sobreposição em mudanças de foco de janela
- **Comandos CLI para gerenciamento de sobreposição ativa**:
  - `active-enable`: Habilitar destaque de sobreposição do display ativo
  - `active-disable`: Desabilitar destaque de sobreposição do display ativo
  - `active-color <r> <g> <b> [a]`: Definir cor e opacidade da sobreposição ativa
  - Comando `status` melhorado agora exibe configurações de sobreposições inativas e ativas
  - Texto de ajuda melhorado com exemplos abrangentes de uso para todos os modos de sobreposição

### Melhorado
- Fluxo de trabalho de configuração: Não é mais necessário reiniciar a aplicação após alterar configurações com `spotlight-dimmer-config.exe`
- Experiência do usuário: Mudanças de configuração agora são quase instantâneas (janela de detecção de 2 segundos)
- Eficiência de recursos: Monitoramento de configuração adiciona apenas 16 bytes de memória e <0,01% de sobrecarga de CPU
- Arquitetura de sobreposição: Refatorada para suportar múltiplos tipos de sobreposição com gerenciamento de ciclo de vida independente
- Lógica de recarregamento de configuração: Agora lida com mudanças de sobreposições inativas e ativas independentemente
- Tratamento de hotplug de display: Ambos os tipos de sobreposição adequadamente recriados quando displays são conectados/desconectados

### Corrigido
- **Sangramento de cor entre tipos de sobreposição**: Corrigido bug onde mover janelas entre displays fazia sobreposições inativas adotarem cores de sobreposições ativas
  - Causa raiz: Ambos os tipos de sobreposição compartilhavam a mesma classe de janela Windows, fazendo `SetClassLongPtrW(GCLP_HBRBACKGROUND)` afetar todas as janelas
  - Solução: Implementadas classes de janela separadas (`SpotlightDimmerInactiveOverlay` e `SpotlightDimmerActiveOverlay`) para cada tipo de sobreposição
  - Cada classe de janela agora mantém seu próprio pincel de fundo independente, prevenindo contaminação cruzada de cores
- Estrutura do projeto: Removido aninhamento duplicado `src/src/` - agora segue a convenção padrão do Cargo com `Cargo.toml` na raiz e arquivos fonte em `src/`
- Fluxo de trabalho GitHub Actions: Atualizado para usar build puro do Cargo ao invés da action Tauri
- GitHub Actions: Alterado de instaladores NSIS para arquivos ZIP para distribuição mais simples e confiável
- Documentação: Corrigidos todos os caminhos de build para refletir a nova estrutura (removidos passos `cd src`)

## [0.1.9] - 2025-09-30

### Changed
- **Complete architecture rewrite**: Removed Tauri dependency in favor of pure Windows API implementation
- Binary size reduced from 10.1 MB to 561 KB (~95% reduction)
- Memory footprint reduced from ~200 MB to ~7.6 MB (~96% reduction)
- Replaced webview-based overlays with native Windows layered windows for perfect transparency
- Configuration now persists in TOML format at `%APPDATA%\spotlight-dimmer\config.toml`
- Split functionality into two binaries: `spotlight-dimmer.exe` (core) and `spotlight-dimmer-config.exe` (settings)

### Added
- Native Windows API overlay implementation using `WS_EX_LAYERED` windows with alpha blending
- TOML-based configuration system for persistent settings
- CLI configuration tool (`spotlight-dimmer-config.exe`) for managing settings without GUI overhead
- Support for custom overlay colors via configuration (RGB + alpha)
- Automatic configuration loading from user AppData directory

### Removed
- Tauri framework and all web-based UI components
- WebView2 dependency (no longer needed)
- System tray icon (can be re-added with lightweight `trayicon` crate if needed)
- JavaScript/HTML/CSS frontend files (dist/ directory no longer required at runtime)
- ~25 MB of dependency crates (tokio full features, serde_json, tauri-plugin-log, etc.)

### Improved
- Startup time is now instantaneous (no webview initialization overhead)
- CPU usage reduced during idle monitoring
- Focus tracking remains at 100ms polling interval with enhanced detection
- Click-through overlays work flawlessly with native Windows API flags

---

### Alterado
- **Reescrita completa da arquitetura**: Removida dependência do Tauri em favor de implementação pura com Windows API
- Tamanho do binário reduzido de 10,1 MB para 561 KB (redução de ~95%)
- Pegada de memória reduzida de ~200 MB para ~7,6 MB (redução de ~96%)
- Substituídas sobreposições baseadas em webview por janelas em camadas nativas do Windows para transparência perfeita
- Configuração agora persiste em formato TOML em `%APPDATA%\spotlight-dimmer\config.toml`
- Funcionalidade dividida em dois binários: `spotlight-dimmer.exe` (núcleo) e `spotlight-dimmer-config.exe` (configurações)

### Adicionado
- Implementação de sobreposição nativa com Windows API usando janelas `WS_EX_LAYERED` com mistura alfa
- Sistema de configuração baseado em TOML para configurações persistentes
- Ferramenta de configuração CLI (`spotlight-dimmer-config.exe`) para gerenciar configurações sem sobrecarga de GUI
- Suporte para cores de sobreposição personalizadas via configuração (RGB + alfa)
- Carregamento automático de configuração do diretório AppData do usuário

### Removido
- Framework Tauri e todos os componentes de interface baseados em web
- Dependência do WebView2 (não mais necessário)
- Ícone da bandeja do sistema (pode ser readicionado com crate `trayicon` leve se necessário)
- Arquivos frontend JavaScript/HTML/CSS (diretório dist/ não mais necessário em tempo de execução)
- ~25 MB de crates de dependência (recursos completos do tokio, serde_json, tauri-plugin-log, etc.)

### Melhorado
- Tempo de inicialização agora é instantâneo (sem sobrecarga de inicialização de webview)
- Uso de CPU reduzido durante monitoramento ocioso
- Rastreamento de foco permanece em intervalo de polling de 100ms com detecção aprimorada
- Sobreposições click-through funcionam perfeitamente com flags nativas da Windows API

## [0.1.8] - 2025-09-29

### Added
- GitHub Codespaces development environment with complete Rust, Tauri, and Claude Code setup
- GitHub Actions workflow for automated releases with cross-platform binaries
- Comprehensive changelog maintenance system for AI agents
- Portuguese translation requirement for all changelog entries
- Automatic version increment rule for every prompt with code changes

### Improved
- GitHub Actions workflow optimization: Removed unnecessary Node.js setup and frontend build steps since project uses static assets only
- Reduced CI/CD build time and complexity by eliminating redundant npm operations

### Changed
- Switched from WiX MSI to NSIS installer for better GitHub Actions compatibility and reliability
- Windows installer now generates .exe setup files instead of .msi packages
- Improved cross-platform build support with NSIS (works on Linux/macOS hosts)

### Fixed
- GitHub Actions build failure: Switched to NSIS installer to resolve persistent WiX/MSI packaging errors
- Bundle identifier warning: Changed from com.spotlightdimmer.app to com.spotlightdimmer.desktop to avoid macOS conflicts
- Version synchronization: Fixed version mismatch between package.json, Cargo.toml, and tauri.conf.json
- WiX template configuration: Eliminated WiX dependency by switching to NSIS
- Added verbose logging to GitHub Actions for better build diagnostics

### Fixed
- GitHub Actions changelog update: Fixed detached HEAD issue in post-release workflow step
- GitHub Actions permissions: Added contents write permission to update-changelog job to fix 403 push errors

### Improved
- Build reliability: NSIS installer eliminates GitHub Actions WiX Toolset installation issues
- Installer size: NSIS compression achieves 20.8% size reduction (vs original binary)
- WinGet compatibility: NSIS installers fully support Windows Package Manager with 'nullsoft' type
- Cross-compilation: Can now build Windows installers on Linux/macOS runners

---

### Adicionado
- Ambiente de desenvolvimento GitHub Codespaces com configuração completa de Rust, Tauri e Claude Code
- Fluxo de trabalho do GitHub Actions para releases automatizadas com binários multiplataforma
- Sistema abrangente de manutenção de changelog para agentes de IA
- Requisito de tradução em português para todas as entradas do changelog
- Regra de incremento automático de versão para cada prompt com mudanças de código

### Alterado
- Mudança de instalador WiX MSI para NSIS para melhor compatibilidade e confiabilidade no GitHub Actions
- Instalador Windows agora gera arquivos setup .exe ao invés de pacotes .msi
- Melhor suporte de build multiplataforma com NSIS (funciona em hosts Linux/macOS)

### Corrigido
- Falha de build do GitHub Actions: Mudança para instalador NSIS resolve erros persistentes de empacotamento WiX/MSI
- Aviso de identificador de bundle: Alterado de com.spotlightdimmer.app para com.spotlightdimmer.desktop para evitar conflitos no macOS
- Sincronização de versão: Corrigida disparidade de versões entre package.json, Cargo.toml e tauri.conf.json
- Configuração de template WiX: Eliminada dependência do WiX ao mudar para NSIS
- Adicionado logging verboso ao GitHub Actions para melhor diagnóstico de builds
- Atualização de changelog do GitHub Actions: Corrigido problema de HEAD desanexado no passo de workflow pós-release
- Permissões do GitHub Actions: Adicionada permissão de escrita de conteúdo ao job update-changelog para corrigir erros 403 de push

### Melhorado
- Confiabilidade de build: Instalador NSIS elimina problemas de instalação do WiX Toolset no GitHub Actions
- Tamanho do instalador: Compressão NSIS alcança redução de 20,8% do tamanho (vs binário original)
- Compatibilidade WinGet: Instaladores NSIS suportam completamente o Windows Package Manager com tipo 'nullsoft'
- Compilação cruzada: Agora pode construir instaladores Windows em runners Linux/macOS

## [0.1.0] - 2024-09-28

### Added
- Initial release of Spotlight Dimmer
- Cross-platform display dimming functionality (Windows implementation)
- Real-time focus monitoring with 100ms polling interval
- Click-through transparent overlays on inactive displays
- System tray integration with toggle controls
- Hybrid overlay loading system supporting both Tauri builds and cargo install
- Enhanced window movement detection across multiple displays
- Auto-startup functionality (starts dimming by default)
- Professional Windows installer (MSI) support
- Custom application icons and branding
- Self-healing display layout change detection
- Minimized startup mode for background operation

### Technical Features
- Tauri 2.x framework integration
- Windows API integration for display and window management
- Thread-safe state management with Arc<Mutex<>>
- Async Tauri command system
- Embedded asset fallback system for portable installations
- Color picker integration for display identification
- Multiple installation methods (Tauri build vs cargo install)

### Development
- Comprehensive build system supporting multiple installation methods
- Cross-platform architecture ready for Linux/macOS expansion
- Extensive documentation in AGENTS.md for AI development
- Build time optimization guidelines
- Professional development tooling setup