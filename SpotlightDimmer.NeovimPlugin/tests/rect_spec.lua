-- Headless tests for compute_rect() / payload().
--
-- Run from the repository root:
--   nvim --headless -u NONE -l SpotlightDimmer.NeovimPlugin/tests/rect_spec.lua

vim.opt.rtp:prepend(vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":h:h"))
local sd = require("spotlight-dimmer")

local failures = 0

local function check(name, got, want)
  local ok = vim.deep_equal(got, want)
  if not ok then
    failures = failures + 1
  end
  print(string.format("%s %s: got %s, want %s",
    ok and "ok  " or "FAIL", name, vim.inspect(got), vim.inspect(want)))
end

local function rect(col, row, width, height)
  return { col = col, row = row, width = width, height = height }
end

-- Without an attached UI, nvim keeps its default 80x24 grid; resizing it
-- headless is unreliable (windows resize lazily), so the expectations below
-- are written for that fixed grid.
assert(vim.o.columns == 80 and vim.o.lines == 24,
  "expected the headless default 80x24 grid, got " .. vim.o.columns .. "x" .. vim.o.lines)

local function reset(laststatus)
  -- :only cannot run from a floating window; close those first
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    if vim.api.nvim_win_get_config(win).relative ~= "" then
      vim.api.nvim_win_close(win, true)
    end
  end
  vim.cmd("silent! only | silent! tabonly")
  vim.o.cmdheight = 1
  vim.o.showtabline = 0
  vim.o.laststatus = laststatus
  vim.o.winbar = ""
  vim.o.equalalways = true
end

-- laststatus=2 -----------------------------------------------------------
-- 24 lines = 22 text rows + statusline + cmdline

reset(2)
check("single window, ls=2", sd.compute_rect(), rect(0, 0, 80, 23))
check("single split payload unsets the option", sd.payload(), "")

reset(2)
vim.cmd("vsplit")
-- left 40 cols, separator at col 40, right 39 cols from col 41
check("vsplit left takes its right separator", sd.compute_rect(), rect(0, 0, 41, 23))
vim.cmd("wincmd l")
check("vsplit right takes its left separator", sd.compute_rect(), rect(40, 0, 40, 23))

-- Split count: only 2+ splits in the current tab narrow the pane ---------

reset(2)
vim.cmd("vsplit")
check("payload with two splits", sd.payload(), "80,24,0,0,41,23")
vim.cmd("only")
check("payload back to one split", sd.payload(), "")
vim.cmd("vsplit | tabnew")
check("splits in another tab do not count", sd.payload(), "")
vim.cmd("tabprevious")
check("splits in the current tab do", sd.payload(), "80,24,0,0,41,23")
vim.cmd("only")
vim.api.nvim_open_win(vim.api.nvim_create_buf(false, true), false, {
  relative = "editor", row = 5, col = 10, width = 40, height = 10,
})
check("floats are not splits", sd.payload(), "")

reset(2)
vim.cmd("split")
-- top 11 rows + statusline, bottom 10 rows + statusline (the odd row goes
-- to the top window)
check("split top includes its statusline", sd.compute_rect(), rect(0, 0, 80, 12))
vim.cmd("wincmd j")
check("split bottom includes its statusline", sd.compute_rect(), rect(0, 12, 80, 11))

-- laststatus=3 (global statusline) -------------------------------------
-- 24 lines = 22 rows of windows + global statusline + cmdline

reset(3)
check("single window, ls=3", sd.compute_rect(), rect(0, 0, 80, 22))

reset(3)
vim.cmd("split")
-- top 11 rows + separator, bottom 10 rows (nothing below but the global
-- statusline, which belongs to no window)
check("ls=3 split top includes the separator", sd.compute_rect(), rect(0, 0, 80, 12))
vim.cmd("wincmd j")
check("ls=3 split bottom stops above the global statusline", sd.compute_rect(), rect(0, 12, 80, 10))

-- laststatus=0/1: no statusline on a lone last window --------------------

reset(1)
check("single window, ls=1 has no statusline", sd.compute_rect(), rect(0, 0, 80, 23))
reset(0)
vim.cmd("split")
vim.cmd("wincmd j")
check("ls=0 last window has no statusline", sd.compute_rect(), rect(0, 12, 80, 11))

-- Mixed layout: the rect is an interior cell of the grid ---------------

reset(2)
vim.cmd("vsplit | split")
check("mixed top-left", sd.compute_rect(), rect(0, 0, 41, 12))
vim.cmd("wincmd j")
check("mixed bottom-left", sd.compute_rect(), rect(0, 12, 41, 11))

-- winbar and tabline shift the window down; both belong in the rect -------

reset(2)
vim.o.winbar = "%f"
check("winbar row is part of the window", sd.compute_rect(), rect(0, 0, 80, 23))
vim.o.winbar = ""
vim.o.showtabline = 2
check("tabline row is excluded", sd.compute_rect(), rect(0, 1, 80, 22))

-- Floating windows clear the option (whole pane spotlighted) -------------

reset(2)
local buf = vim.api.nvim_create_buf(false, true)
vim.api.nvim_open_win(buf, true, {
  relative = "editor", row = 5, col = 10, width = 40, height = 10,
})
check("floating window returns nil", sd.compute_rect(), nil)
check("floating window payload unsets the option", sd.payload(), "")

-- Command-line mode lights the whole pane ---------------------------------
-- A prompt (nvim-tree's "create file", via vim.ui.input) keeps the focused
-- window but draws on the command line, or in noice's floating popup.

local function payload_in_cmdline()
  local got
  -- <C-r>= evaluates while still in command-line mode
  _G.sd_capture = function()
    got = sd.payload()
    return ""
  end
  vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes(
    [[:<C-r>=v:lua.sd_capture()<CR><Esc>]], true, false, true), "x", false)
  return got
end

reset(2)
vim.cmd("vsplit")
check("command line payload unsets the option", payload_in_cmdline(), "")
check("payload back after the command line", sd.payload(), "80,24,0,0,41,23")

-- Title transport segment -------------------------------------------------

reset(2)
check("title segment empty on a single split", sd.title_segment(), "")
vim.cmd("vsplit")
check("title segment", sd.title_segment(), "sd-nvim=80,24,0,0,41,23")
vim.api.nvim_open_win(vim.api.nvim_create_buf(false, true), true, {
  relative = "editor", row = 5, col = 10, width = 40, height = 10,
})
check("title segment empty on a float", sd.title_segment(), "")

if failures > 0 then
  print(string.format("%d failure(s)", failures))
  os.exit(1)
end
print("all passed")
os.exit(0)
