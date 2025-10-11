# Spotlight Dimmer

## Visão Geral

Uma aplicação leve para Windows que escurece displays inativos para destacar o ativo. Construído com Rust puro e Windows API para máximo desempenho e uso mínimo de recursos.

> For the English version go to: [README.md](README.md)

Spotlight Dimmer é um programa para Windows que escurece todos os monitores exceto o monitor que possui o programa em foco no momento.

É destinado a ajudar pessoas que usam múltiplos monitores a focar e auxiliar em notar rapidamente qual janela tem o foco atual ao trocar de foco com atalhos como `alt + tab`. É especialmente útil para usuários que navegam principalmente com o teclado. Ajuda a evitar situações como digitar comandos de terminal no Teams porque você está olhando para uma tela enquanto o foco está em outra tela.

## Funcionalidades

- **Ultra-leve**: Apenas ~7.6 MB de uso de RAM, ~561 KB de tamanho binário
- **Windows API nativo**: Sem sobrecarga de mecanismo de navegador, inicialização instantânea
- **Transparência perfeita**: Escurecimento suave de 50% (customizável) em displays inativos
- **Sobreposições click-through**: Sobreposições não interferem com entrada de mouse/teclado
- **Rastreamento automático de foco**: Detecta mudanças de janela ativa e display em tempo real (polling de 100ms)
- **Suporte a hotplug de displays**: Recria automaticamente sobreposições quando displays são conectados/desconectados
- **Configuração persistente**: Configurações salvas em formato TOML em `%APPDATA%\spotlight-dimmer\config.toml`
- **Ferramenta CLI de configuração**: Gerencie configurações sem executar a aplicação principal

## Instalação

### A Partir do Código-Fonte

```bash
cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config
```

Os binários estarão em `target\release\`:
- `spotlight-dimmer.exe` - Aplicação principal
- `spotlight-dimmer-config.exe` - Ferramenta de configuração

### Uso

#### Executando a Aplicação

Simplesmente execute `spotlight-dimmer.exe`:

```cmd
spotlight-dimmer.exe
```

A aplicação irá:
1. Carregar configuração de `%APPDATA%\spotlight-dimmer\config.toml` (ou criar padrão)
2. Detectar todos os displays conectados
3. Criar janelas de sobreposição semi-transparentes em cada display
4. Monitorar foco da janela ativa e esconder sobreposição no display ativo
5. Executar indefinidamente até ser terminado

#### Parando a Aplicação

Para parar a aplicação em execução, use PowerShell (**não** Prompt de Comando ou bash):

```powershell
Get-Process spotlight-dimmer | Stop-Process
```

Ou use o Gerenciador de Tarefas para encerrar o processo `spotlight-dimmer.exe`.

#### Ferramenta de Configuração

Use `spotlight-dimmer-config.exe` para gerenciar configurações:

```cmd
# Mostrar configuração atual
spotlight-dimmer-config status

# Habilitar/desabilitar escurecimento
spotlight-dimmer-config enable
spotlight-dimmer-config disable

# Definir cor da sobreposição (RGB 0-255, alfa 0.0-1.0)
spotlight-dimmer-config color 0 0 0 0.7      # Sobreposição preta 70%
spotlight-dimmer-config color 50 50 50 0.3   # Sobreposição cinza 30%

# Resetar para padrões
spotlight-dimmer-config reset
```

**Nota**: Mudanças de configuração são detectadas e recarregadas automaticamente em até 2 segundos. Não é necessário reiniciar!

## Arquivo de Configuração

A configuração é armazenada em `%APPDATA%\spotlight-dimmer\config.toml`:

```toml
is_dimming_enabled = true

[overlay_color]
r = 0
g = 0
b = 0
a = 0.5
```

## Arquitetura

### Aplicação Principal (`spotlight-dimmer.exe`)

- **Uso de memória**: ~7.6 MB
- **Tamanho binário**: 561 KB
- **Implementação**: Windows API puro com crate Rust `winapi`
- **Tecnologia de sobreposição**: Janelas em camadas (`WS_EX_LAYERED`) com mistura alfa
- **Monitoramento de foco**: Polling de 100ms usando `GetForegroundWindow()` e `MonitorFromWindow()`

### Ferramenta de Configuração (`spotlight-dimmer-config.exe`)

- **Tamanho binário**: 627 KB
- **Implementação**: Ferramenta CLI usando `clap` para parsing de argumentos
- **Configuração**: Formato TOML via crate `toml`

### Detalhes Técnicos Principais

- **Click-through**: Flag `WS_EX_TRANSPARENT` garante que sobreposições não capturem entrada
- **Sempre no topo**: `WS_EX_TOPMOST` mantém sobreposições acima de outras janelas
- **Sem barra de tarefas**: `WS_EX_TOOLWINDOW` previne sobreposições de aparecerem no Alt+Tab
- **Sem foco**: `WS_EX_NOACTIVATE` previne sobreposições de roubar foco
- **Transparência**: `SetLayeredWindowAttributes()` com `LWA_ALPHA` para escurecimento suave

## Comparação com Versão Tauri

| Métrica | Tauri v0.1.8 | WinAPI v0.1.9 | Melhoria |
|---------|--------------|---------------|----------|
| Tamanho Binário | 10.1 MB | 561 KB | ~95% redução |
| Uso de Memória | ~200 MB | ~7.6 MB | ~96% redução |
| Tempo de Inicialização | ~400ms | Instantâneo | N/A |
| Dependências | 30+ crates | 3 crates | Mínimo |
| Deps. Runtime | WebView2 | Nenhuma | Auto-contido |

## Desenvolvimento

### Estrutura do Projeto

```
.
├── src/
│   ├── main_new.rs          # Ponto de entrada da aplicação principal
│   ├── config_cli.rs        # Ferramenta CLI de configuração
│   ├── config.rs            # Sistema de configuração (TOML)
│   ├── overlay.rs           # Implementação WinAPI de sobreposição
│   └── platform/
│       ├── mod.rs           # Traits multiplataforma
│       └── windows.rs       # Gerenciamento de display/janela do Windows
├── Cargo.toml               # Dependências Rust
└── target/release/          # Saída do build
```

### Construindo

```bash
cargo build --release
```

### Dependências

- `serde` - Serialização de configuração
- `toml` - Parsing de configuração TOML
- `winapi` - Bindings da Windows API

## Limitações Conhecidas

### Comportamento ao Arrastar Janelas

Ao arrastar janelas entre monitores com o mouse, as sobreposições são temporariamente ocultadas para prevenir instabilidade do sistema:

- **Durante o arraste**: Todas as sobreposições desaparecem quando o botão esquerdo do mouse é pressionado
- **Após o arraste**: Sobreposições reaparecem com visibilidade correta quando o botão do mouse é solto
- **Por quê**: O loop de mensagens de arrastar e soltar do Windows conflita com atualizações de visibilidade de sobreposição, causando instabilidade do sistema se as sobreposições permanecerem visíveis
- **Solução alternativa**: Use atalhos de teclado (teclas Win+Seta) para atualizações instantâneas de sobreposição sem ocultação

Esta é uma limitação da API do Windows, não um bug. Mudanças de foco e movimentação de janelas baseada em teclado funcionam instantaneamente sem ocultar sobreposições.

## Roadmap

- [ ] Ícone na bandeja do sistema (opcional, usando crate `trayicon`)
- [x] Hot reload de configuração sem reiniciar (janela de detecção de 2 segundos)
- [ ] Customização de cor por display
- [ ] Suporte a Linux (usando X11/Wayland)

## Licença

MIT

## Créditos

Construído com Rust e Windows API para máximo desempenho.
