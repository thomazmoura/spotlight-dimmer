# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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