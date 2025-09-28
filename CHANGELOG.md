# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions workflow for automated releases with cross-platform binaries
- Comprehensive changelog maintenance system for AI agents
- Portuguese translation requirement for all changelog entries
- Automatic version increment rule for every prompt with code changes

### Improved
- GitHub Actions workflow optimization: Removed unnecessary Node.js setup and frontend build steps since project uses static assets only
- Reduced CI/CD build time and complexity by eliminating redundant npm operations

### Fixed
- GitHub Actions build failure: Added explicit WiX Toolset installation to resolve MSI packaging errors on Windows runners
- Bundle identifier warning: Changed from com.spotlightdimmer.app to com.spotlightdimmer.desktop to avoid macOS conflicts
- Version synchronization: Fixed version mismatch between package.json, Cargo.toml, and tauri.conf.json
- WiX template configuration: Removed missing main.wxs template reference to use Tauri's default template
- Added verbose logging to GitHub Actions for better build diagnostics

---

### Adicionado
- Fluxo de trabalho do GitHub Actions para releases automatizadas com binários multiplataforma
- Sistema abrangente de manutenção de changelog para agentes de IA
- Requisito de tradução em português para todas as entradas do changelog
- Regra de incremento automático de versão para cada prompt com mudanças de código

### Melhorado
- Otimização do fluxo de trabalho do GitHub Actions: Removidos passos desnecessários de configuração Node.js e build frontend já que o projeto usa apenas assets estáticos
- Redução do tempo de build CI/CD e complexidade ao eliminar operações npm redundantes

### Corrigido
- Falha de build do GitHub Actions: Adicionada instalação explícita do WiX Toolset para resolver erros de empacotamento MSI nos runners Windows
- Aviso de identificador de bundle: Alterado de com.spotlightdimmer.app para com.spotlightdimmer.desktop para evitar conflitos no macOS
- Sincronização de versão: Corrigida disparidade de versões entre package.json, Cargo.toml e tauri.conf.json
- Configuração de template WiX: Removida referência ao template main.wxs inexistente para usar o template padrão do Tauri
- Adicionado logging verboso ao GitHub Actions para melhor diagnóstico de builds

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