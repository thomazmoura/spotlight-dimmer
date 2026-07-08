# Changelog — Linux

All notable changes to the Linux version of SpotlightDimmer — the `spotlight-dimmer-daemon`, the GNOME Shell extension and the KWin script — are documented in this file. Windows changes are tracked separately in [CHANGELOG.md](CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Linux releases are tagged `vX.Y.Z-linux` and versioned independently from the Windows releases.

## [Unreleased]

## [0.2.1] - 2026-07-08

### Fixed
- **Linux release pipeline failure**: The v0.2.0 release pipeline failed before publishing anything because the CI runners (Ubuntu 24.04) do not ship the `libgtk4-layer-shell-dev` package required to build the KDE daemon — that library only exists in Ubuntu 25.10+ repositories. The build now runs per package variant: the GNOME .deb is built on Ubuntu 24.04 so it stays installable on Ubuntu 24.04 LTS and newer, while the KDE .deb is built on Ubuntu 26.04 LTS (it requires Ubuntu 25.10+ either way, since its gtk4-layer-shell runtime dependency is not available on older releases). This makes v0.2.1 the first Linux release with published .deb packages — see the [0.2.0] section below for the full feature list it delivers (the v0.2.0 tag never produced a release).

---

### Corrigido
- **Falha na pipeline de release do Linux**: A pipeline de release da v0.2.0 falhou antes de publicar qualquer coisa porque os runners de CI (Ubuntu 24.04) não fornecem o pacote `libgtk4-layer-shell-dev` necessário para compilar o daemon do KDE — essa biblioteca só existe nos repositórios do Ubuntu 25.10+. O build agora roda por variante de pacote: o .deb do GNOME é compilado no Ubuntu 24.04 para continuar instalável no Ubuntu 24.04 LTS e mais recentes, enquanto o .deb do KDE é compilado no Ubuntu 26.04 LTS (ele exige Ubuntu 25.10+ de qualquer forma, já que sua dependência de runtime gtk4-layer-shell não está disponível em versões mais antigas). Isso faz da v0.2.1 a primeira release do Linux com pacotes .deb publicados — veja a seção [0.2.0] abaixo para a lista completa de funcionalidades que ela entrega (a tag v0.2.0 nunca gerou uma release).

## [0.2.0] - 2026-07-07

### Added
- **Official .deb packages for Ubuntu/Kubuntu (amd64 and arm64)**: Linux releases now ship ready-to-install packages instead of requiring a build from source
  - `spotlight-dimmer-gnome`: the headless daemon plus the GNOME Shell extension installed system-wide — after installing, log out and back in, then run `gnome-extensions enable spotlightdimmer@thomazmoura.github.io`
  - `spotlight-dimmer-kde`: the daemon with the layer-shell renderer plus the KWin script, which KWin loads automatically after install
  - Install with `sudo apt install ./<package>.deb` — runtime dependencies (GTK4 and gtk4-layer-shell for the KDE package) are resolved automatically
  - The two packages intentionally conflict with each other; install the one matching your desktop
  - First run: copy `/usr/share/doc/<package>/examples/config.example.json` to `~/.config/SpotlightDimmer/config.json` to start with a visible PartialWithActive setup
- **Automated Linux release pipeline**: pushing a `vX.Y.Z-linux` tag now builds and tests the daemon, packages the four .deb files (GNOME/KDE × amd64/arm64) and publishes them to a GitHub release with these notes
- **Linux CI**: pushes and pull requests touching the Linux code now automatically run the Rust test suite and build both daemon profiles (headless and layer-shell)
- **KDE Plasma 6 (Wayland) support**: SpotlightDimmer now dims inactive displays and regions on KDE Plasma
  - New shared Rust daemon (`spotlight-dimmer-daemon`) owns configuration, overlay calculation and the wezterm/tmux integration for all Linux compositors
  - A KWin script (`SpotlightDimmer.KwinScript`) reports focus, geometry and monitor changes to the daemon over D-Bus; the daemon renders click-through overlays via layer-shell
  - All three dimming modes (FullScreen, Partial, PartialWithActive), multi-monitor with hot-plug, config hot-reload and the tmux pane spotlight work on KDE
  - Meta+Shift+D toggles dimming (configurable in System Settings → Shortcuts)
  - Install with `make install-linux-kde` from `SpotlightDimmer.LinuxDaemon/`; see `docs/LINUX_DAEMON.md`
- **tmux pane highlighting inside WezTerm (GNOME)**: The spotlight can now follow the focused tmux pane instead of the whole terminal window
  - When WezTerm is focused and running tmux, sibling panes and everything outside the focused pane are dimmed; the highlight follows pane switches, splits, and resizes instantly
  - New `AppIntegrations` config section maps a window class to an integration provider (`"tmux"`), with per-app `ContentOffsetX`/`ContentOffsetY` to account for terminal padding and tab bars
  - Event-driven via tmux hooks and a new D-Bus service (`org.spotlightdimmer.PaneTracker`) — no polling
  - The extension verifies the focused terminal content is a live tmux client (tty matching via `wezterm cli` and `tmux list-clients`) and falls back to whole-window highlighting whenever pane data is unavailable
  - Ships ready-to-use setup files: `tools/spotlight-dimmer-tmux-report.sh` (geometry reporter run by tmux hooks) and `tools/spotlight-dimmer.tmux.conf` (hook definitions to source from `~/.tmux.conf`)
  - Full setup guide in `docs/TMUX_INTEGRATION.md`
- **Global keyboard shortcut (Super+Shift+D)**: Toggle all SpotlightDimmer overlays on/off without disabling the extension
  - Press once to pause: all overlays disappear and focus/window changes won't bring them back
  - Press again to resume: overlays recalculate and appear correctly based on current state
  - Shortcut works in both normal and overview modes
  - Extension stays fully enabled while paused (signals remain connected for instant resume)
- **GNOME Shell Extension for Linux**: Initial implementation of SpotlightDimmer for GNOME Wayland
  - Supports GNOME Shell 45, 46, 47, and 48
  - Three dimming modes: FullScreen, Partial, and PartialWithActive (feature parity with Windows)
  - Reads shared configuration from `~/.config/SpotlightDimmer/config.json`
  - Hot-reload support: Changes to config file apply instantly without restart
  - Click-through overlays: Interact with windows beneath the dimming overlays
  - Multi-monitor support with automatic detection and hot-plug handling
  - Event-driven focus tracking using GNOME Shell's Meta.Display signals
  - New `spotlight-dimmer-gnome/` directory with JavaScript (GJS) implementation:
    - `extension.js` - Main orchestration and lifecycle management
    - `calculator.js` - Port of C# overlay calculation logic
    - `configBridge.js` - Configuration loading with GLib.FileMonitor
    - `overlayManager.js` - St.Widget overlay management
    - `focusTracker.js` - Focus and window position tracking

### Changed
- **Linux changes now live in their own changelog**: this file (`CHANGELOG.linux.md`) tracks the Linux version, which is released and versioned independently from Windows using `vX.Y.Z-linux` tags
- **GNOME Shell extension is now a thin adapter for the shared daemon**: overlay calculation, configuration loading and the wezterm/tmux integration moved from the extension into `spotlight-dimmer-daemon`, eliminating duplicated logic between GNOME and KDE
  - The daemon must now be installed for dimming to work on GNOME: run `make install-linux-gnome` from `SpotlightDimmer.LinuxDaemon/` (the daemon is D-Bus activated and restarts automatically; no manual start needed)
  - Behavior is unchanged: same modes, same `~/.config/SpotlightDimmer/config.json`, same Super+Shift+D toggle, same tmux pane spotlight (existing tmux hook installs keep working — the `org.spotlightdimmer.PaneTracker` D-Bus interface is identical)
  - The tmux setup files moved from `SpotlightDimmer.GnomeShellExtension/tools/` to `SpotlightDimmer.LinuxDaemon/tools/`
- **README now covers both Windows and Linux**: Restructured with platform-specific installation instructions — winget/installer for Windows, and step-by-step local install guides for Ubuntu (GNOME) and Kubuntu (KDE Plasma 6), including requirements, uninstall steps, and the optional tmux pane spotlight setup
- **Linux install commands now work from the repository root**: Added a root-level Makefile that forwards `make install-linux-gnome`, `make install-linux-kde` and related targets to `SpotlightDimmer.LinuxDaemon/`, so the README instructions work without changing into a subdirectory
- **Linux install seeds a starter configuration**: When `~/.config/SpotlightDimmer/config.json` doesn't exist, the install now copies the example config (PartialWithActive mode), so dimming is visible immediately after installing — previously the daemon defaulted to FullScreen mode, which shows nothing on single-monitor setups

### Fixed
- **Release changelog tooling**: `Move-UnreleasedToVersion.ps1` no longer duplicates the old `[Unreleased]` section on every release, and `Extract-Changelog.ps1` can extract a released version's section via `-Version`, so Linux release notes are built from the correct changelog section
- **tmux pane highlight no longer goes stale in the session/window chooser**: Opening tmux's session chooser (`prefix+s`) or window chooser (`prefix+w`) now moves the highlight to cover the chooser instead of leaving it on the previously focused pane — the default `choose-tree -Z` bindings zoom the active pane, and the integration now re-reports geometry on mode and layout changes (new `pane-mode-changed` and `window-layout-changed` tmux hooks). Re-copy `tools/spotlight-dimmer.tmux.conf` to `~/.config/SpotlightDimmer/tools/` and reload tmux to get the fix
- **Overlays now cover panels and taskbars on Linux**: Dimming is computed against the full monitor geometry instead of the work area, so the panel/taskbar strip is dimmed like the rest of the screen (matching the Windows behavior) — previously an undimmed gap remained over the panel even with fullscreen applications
- **The KDE application launcher and other popups are now spotlighted**: Opening the start menu (Kickoff), KRunner or other focused popups now highlights them like any window instead of dimming the entire screen — previously only "normal" windows were treated as focus targets
- **Dimming no longer stops after a daemon restart on KDE**: The daemon now caches the reported monitor layout in the session runtime directory and restores it on startup, so upgrades and crash recoveries keep dimming without waiting for a monitor hotplug event (the KWin script only reports monitors on load and on screen changes)
- **KDE install now reloads a running KWin script**: `make install-kwin` unloads the previous script instance via KWin's scripting D-Bus API before reconfiguring, so upgrades actually run the new script version and re-register with the daemon — previously the old script kept running until logout
- **GNOME extension fullscreen application support**: Overlays now work correctly with fullscreen applications
  - Other monitors are properly dimmed when one monitor has a fullscreen window
  - Active overlay (PartialWithActive mode) now renders above fullscreen content
  - Edge overlays automatically hidden when window is maximized or fullscreen (no visible gap to dim)
  - Added explicit `trackFullscreen: false` to ensure overlays stay visible across GNOME Shell versions
  - Added `in-fullscreen-changed` signal handler for system-wide fullscreen state changes
- **GNOME extension dock coverage in Partial modes**: Fixed overlays covering dock area and preventing drag-and-drop operations
  - Overlays now use work area geometry (via `Meta.Workspace.get_work_area_for_monitor`) which excludes dock and panel struts
  - Dock remains fully interactive in Partial and PartialWithActive modes
- **GNOME extension fullscreen flickering**: Fixed overlay flickering with fullscreen applications
  - Extension disables compositor unredirect via `global.compositor.disable_unredirect()` on enable
  - Ensures overlays remain compositor-managed and visible when fullscreen apps are running
  - Restores default unredirect behavior via `global.compositor.enable_unredirect()` on disable

---

### Adicionado
- **Pacotes .deb oficiais para Ubuntu/Kubuntu (amd64 e arm64)**: As versões Linux agora são distribuídas como pacotes prontos para instalar, em vez de exigir compilação a partir do código-fonte
  - `spotlight-dimmer-gnome`: o daemon headless mais a extensão do GNOME Shell instalada para todo o sistema — após instalar, saia e entre na sessão novamente e execute `gnome-extensions enable spotlightdimmer@thomazmoura.github.io`
  - `spotlight-dimmer-kde`: o daemon com o renderizador layer-shell mais o script do KWin, que o KWin carrega automaticamente após a instalação
  - Instale com `sudo apt install ./<pacote>.deb` — as dependências de runtime (GTK4 e gtk4-layer-shell para o pacote KDE) são resolvidas automaticamente
  - Os dois pacotes conflitam entre si propositalmente; instale o que corresponde ao seu desktop
  - Primeira execução: copie `/usr/share/doc/<pacote>/examples/config.example.json` para `~/.config/SpotlightDimmer/config.json` para começar com uma configuração PartialWithActive visível
- **Pipeline automatizado de releases Linux**: enviar uma tag `vX.Y.Z-linux` agora compila e testa o daemon, empacota os quatro arquivos .deb (GNOME/KDE × amd64/arm64) e os publica em um release do GitHub com estas notas
- **CI para Linux**: pushes e pull requests que alteram o código Linux agora executam automaticamente a suíte de testes Rust e compilam os dois perfis do daemon (headless e layer-shell)
- **Suporte ao KDE Plasma 6 (Wayland)**: O SpotlightDimmer agora escurece displays e regiões inativas no KDE Plasma
  - Novo daemon compartilhado em Rust (`spotlight-dimmer-daemon`) é dono da configuração, do cálculo de sobreposições e da integração wezterm/tmux para todos os compositores Linux
  - Um script do KWin (`SpotlightDimmer.KwinScript`) reporta mudanças de foco, geometria e monitores ao daemon via D-Bus; o daemon renderiza sobreposições click-through via layer-shell
  - Os três modos de escurecimento (FullScreen, Partial, PartialWithActive), multi-monitor com hot-plug, hot-reload de configuração e o spotlight de painel tmux funcionam no KDE
  - Meta+Shift+D alterna o escurecimento (configurável em Configurações do Sistema → Atalhos)
  - Instale com `make install-linux-kde` a partir de `SpotlightDimmer.LinuxDaemon/`; veja `docs/LINUX_DAEMON.md`
- **Destaque de painel tmux dentro do WezTerm (GNOME)**: O spotlight agora pode seguir o painel tmux focado em vez da janela inteira do terminal
  - Quando o WezTerm está focado e executando tmux, os painéis irmãos e tudo fora do painel focado são escurecidos; o destaque acompanha trocas de painel, divisões e redimensionamentos instantaneamente
  - Nova seção de configuração `AppIntegrations` mapeia uma classe de janela para um provedor de integração (`"tmux"`), com `ContentOffsetX`/`ContentOffsetY` por aplicativo para compensar padding e barra de abas do terminal
  - Orientado a eventos via hooks do tmux e um novo serviço D-Bus (`org.spotlightdimmer.PaneTracker`) — sem polling
  - A extensão verifica se o conteúdo focado do terminal é um cliente tmux ativo (correspondência de tty via `wezterm cli` e `tmux list-clients`) e retorna ao destaque de janela inteira sempre que os dados do painel estiverem indisponíveis
  - Inclui arquivos de configuração prontos para uso: `tools/spotlight-dimmer-tmux-report.sh` (reportador de geometria executado pelos hooks do tmux) e `tools/spotlight-dimmer.tmux.conf` (definições de hooks para carregar no `~/.tmux.conf`)
  - Guia completo de instalação em `docs/TMUX_INTEGRATION.md`
- **Atalho de teclado global (Super+Shift+D)**: Alterne todas as sobreposições do SpotlightDimmer ligadas/desligadas sem desabilitar a extensão
  - Pressione uma vez para pausar: todas as sobreposições desaparecem e mudanças de foco/janela não as trazem de volta
  - Pressione novamente para retomar: sobreposições recalculam e aparecem corretamente com base no estado atual
  - Atalho funciona tanto no modo normal quanto no modo de visão geral
  - Extensão permanece totalmente habilitada enquanto pausada (sinais permanecem conectados para retomada instantânea)
- **Extensão GNOME Shell para Linux**: Implementação inicial do SpotlightDimmer para GNOME Wayland
  - Suporta GNOME Shell 45, 46, 47 e 48
  - Três modos de escurecimento: FullScreen, Partial e PartialWithActive (paridade de funcionalidades com Windows)
  - Lê configuração compartilhada de `~/.config/SpotlightDimmer/config.json`
  - Suporte a hot-reload: Alterações no arquivo de configuração aplicam instantaneamente sem reiniciar
  - Overlays click-through: Interaja com janelas abaixo das sobreposições de escurecimento
  - Suporte multi-monitor com detecção automática e tratamento de hot-plug
  - Rastreamento de foco orientado a eventos usando sinais Meta.Display do GNOME Shell
  - Novo diretório `spotlight-dimmer-gnome/` com implementação JavaScript (GJS):
    - `extension.js` - Orquestração principal e gerenciamento de ciclo de vida
    - `calculator.js` - Port da lógica de cálculo de overlay do C#
    - `configBridge.js` - Carregamento de configuração com GLib.FileMonitor
    - `overlayManager.js` - Gerenciamento de overlay St.Widget
    - `focusTracker.js` - Rastreamento de foco e posição de janela

### Alterado
- **Mudanças do Linux agora ficam em um changelog próprio**: este arquivo (`CHANGELOG.linux.md`) acompanha a versão Linux, que é lançada e versionada de forma independente do Windows usando tags `vX.Y.Z-linux`
- **A extensão GNOME Shell agora é um adaptador leve para o daemon compartilhado**: o cálculo de sobreposições, o carregamento de configuração e a integração wezterm/tmux foram movidos da extensão para o `spotlight-dimmer-daemon`, eliminando lógica duplicada entre GNOME e KDE
  - O daemon agora precisa estar instalado para o escurecimento funcionar no GNOME: execute `make install-linux-gnome` a partir de `SpotlightDimmer.LinuxDaemon/` (o daemon é ativado via D-Bus e reinicia automaticamente; não é preciso iniciá-lo manualmente)
  - O comportamento permanece o mesmo: mesmos modos, mesmo `~/.config/SpotlightDimmer/config.json`, mesmo atalho Super+Shift+D, mesmo spotlight de painel tmux (instalações existentes dos hooks do tmux continuam funcionando — a interface D-Bus `org.spotlightdimmer.PaneTracker` é idêntica)
  - Os arquivos de configuração do tmux foram movidos de `SpotlightDimmer.GnomeShellExtension/tools/` para `SpotlightDimmer.LinuxDaemon/tools/`
- **O README agora cobre Windows e Linux**: Reestruturado com instruções de instalação específicas por plataforma — winget/instalador para Windows e guias passo a passo de instalação local para Ubuntu (GNOME) e Kubuntu (KDE Plasma 6), incluindo requisitos, passos de desinstalação e a configuração opcional do spotlight de painel tmux
- **Comandos de instalação Linux agora funcionam a partir da raiz do repositório**: Adicionado um Makefile na raiz que encaminha `make install-linux-gnome`, `make install-linux-kde` e alvos relacionados para `SpotlightDimmer.LinuxDaemon/`, de forma que as instruções do README funcionem sem precisar entrar em um subdiretório
- **Instalação Linux cria uma configuração inicial**: Quando `~/.config/SpotlightDimmer/config.json` não existe, a instalação agora copia a configuração de exemplo (modo PartialWithActive), tornando o escurecimento visível imediatamente após a instalação — antes o daemon usava o modo FullScreen por padrão, que não mostra nada em configurações de monitor único

### Corrigido
- **Ferramentas de changelog de release**: O `Move-UnreleasedToVersion.ps1` não duplica mais a seção `[Unreleased]` antiga a cada release, e o `Extract-Changelog.ps1` pode extrair a seção de uma versão lançada via `-Version`, então as notas de release do Linux são montadas a partir da seção correta do changelog
- **O destaque do painel tmux não fica mais desatualizado no seletor de sessões/janelas**: Abrir o seletor de sessões do tmux (`prefix+s`) ou o seletor de janelas (`prefix+w`) agora move o destaque para cobrir o seletor em vez de deixá-lo no painel focado anteriormente — os atalhos padrão `choose-tree -Z` fazem zoom no painel ativo, e a integração agora reenvia a geometria em mudanças de modo e layout (novos hooks tmux `pane-mode-changed` e `window-layout-changed`). Copie novamente `tools/spotlight-dimmer.tmux.conf` para `~/.config/SpotlightDimmer/tools/` e recarregue o tmux para receber a correção
- **Sobreposições agora cobrem painéis e barras de tarefas no Linux**: O escurecimento é calculado sobre a geometria completa do monitor em vez da área de trabalho, então a faixa do painel/barra de tarefas é escurecida como o resto da tela (igual ao comportamento no Windows) — antes restava uma lacuna sem escurecimento sobre o painel mesmo com aplicativos em tela cheia
- **O lançador de aplicativos do KDE e outros popups agora recebem o spotlight**: Abrir o menu iniciar (Kickoff), o KRunner ou outros popups focados agora os destaca como qualquer janela em vez de escurecer a tela inteira — antes apenas janelas "normais" eram tratadas como alvos de foco
- **O escurecimento não para mais após reinício do daemon no KDE**: O daemon agora guarda o layout de monitores reportado no diretório de runtime da sessão e o restaura ao iniciar, então atualizações e recuperações de falhas mantêm o escurecimento sem esperar por um evento de conexão de monitor (o script do KWin só reporta monitores ao carregar e em mudanças de tela)
- **Instalação no KDE agora recarrega um script KWin em execução**: `make install-kwin` descarrega a instância anterior do script via API D-Bus de scripting do KWin antes de reconfigurar, então atualizações realmente executam a nova versão do script e se registram novamente com o daemon — antes o script antigo continuava rodando até o logout
- **Suporte a aplicativos em tela cheia na extensão GNOME**: Sobreposições agora funcionam corretamente com aplicativos em tela cheia
  - Outros monitores são adequadamente escurecidos quando um monitor tem uma janela em tela cheia
  - Sobreposição ativa (modo PartialWithActive) agora renderiza acima do conteúdo em tela cheia
  - Sobreposições de borda automaticamente ocultas quando janela está maximizada ou em tela cheia (sem lacuna visível para escurecer)
  - Adicionado `trackFullscreen: false` explícito para garantir que sobreposições permaneçam visíveis entre versões do GNOME Shell
  - Adicionado manipulador de sinal `in-fullscreen-changed` para mudanças de estado de tela cheia em todo o sistema
- **Cobertura da dock pela extensão GNOME em modos Partial**: Corrigida sobreposição cobrindo área da dock e impedindo operações de arrastar e soltar
  - Sobreposições agora usam geometria de área de trabalho (via `Meta.Workspace.get_work_area_for_monitor`) que exclui struts da dock e painel
  - Dock permanece totalmente interativa nos modos Partial e PartialWithActive
- **Cintilação em tela cheia na extensão GNOME**: Corrigida cintilação de sobreposição com aplicativos em tela cheia
  - Extensão desabilita unredirect do compositor via `global.compositor.disable_unredirect()` ao habilitar
  - Garante que sobreposições permaneçam gerenciadas pelo compositor e visíveis quando aplicativos em tela cheia estão rodando
  - Restaura comportamento padrão de unredirect via `global.compositor.enable_unredirect()` ao desabilitar
