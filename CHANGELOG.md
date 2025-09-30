# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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