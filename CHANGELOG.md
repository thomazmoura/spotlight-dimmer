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

---

### Adicionado
- Fluxo de trabalho do GitHub Actions para releases automatizadas com binários multiplataforma
- Sistema abrangente de manutenção de changelog para agentes de IA
- Requisito de tradução em português para todas as entradas do changelog
- Regra de incremento automático de versão para cada prompt com mudanças de código

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