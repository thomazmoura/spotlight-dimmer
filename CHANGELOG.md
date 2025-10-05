# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.5] - 2025-10-05

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