# SpotlightDimmer neovim plugin

Narrows SpotlightDimmer's tmux pane spotlight down to the focused neovim
split, so the other splits are dimmed the same way inactive panes and windows
are (a replacement for inactive-window plugins like `tint.nvim`).

Requires the Linux tmux integration to be set up first. See
[docs/TMUX_INTEGRATION.md](../docs/TMUX_INTEGRATION.md#neovim-splits).

```vim
" vim-plug
Plug 'thomazmoura/spotlight-dimmer', { 'rtp': 'SpotlightDimmer.NeovimPlugin' }
```

```lua
require("spotlight-dimmer").setup()
```

Inside tmux it publishes the split through a tmux pane option. Over ssh (a
local tmux pane running `ssh host`, neovim on the host) it uses the terminal
title instead, which the desktop's daemon watches. Elsewhere `setup()` does
nothing.

Tests (no tmux needed): `nvim --headless -u NONE -l SpotlightDimmer.NeovimPlugin/tests/rect_spec.lua`
