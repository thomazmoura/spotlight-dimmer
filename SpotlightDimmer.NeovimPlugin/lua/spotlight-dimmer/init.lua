-- SpotlightDimmer neovim integration: narrow the tmux pane spotlight down to
-- the focused neovim split.
--
-- neovim never talks to SpotlightDimmer directly. It publishes the focused
-- window's cell rect as a tmux pane option and asks tmux to re-run the pane
-- report script, which folds the rect into the pane geometry it already
-- sends. Because tmux evaluates that option against the ACTIVE pane, and the
-- daemon only honours pane geometry for a focused terminal attached to a live
-- tmux client, the neovim rect only matters when the whole chain is focused:
-- terminal window > tmux pane > neovim split. With a single split in the tab
-- nothing is published, and the whole pane stays lit. So does typing on the
-- command line, which is where prompts like nvim-tree's "create file" land.
--
-- Payload (all integers, cells, 0-based):
--   "<grid_cols>,<grid_rows>,<col>,<row>,<width>,<height>"
-- The grid size is neovim's own &columns/&lines, which the report script
-- compares against the pane size to drop a rect made stale by a resize.
--
-- Two transports carry it, chosen by where neovim runs:
--   inside tmux ($TMUX set)  the @spotlight_dimmer_nvim pane option, set with
--                            the tmux CLI, which then re-runs the report
--   over ssh (no $TMUX)      the terminal title, as "sd-nvim=<payload>": the
--                            local tmux keeps it as the pane title and copies
--                            it into the window title (#T in set-titles-string),
--                            and the daemon refreshes the pane geometry when it
--                            changes. The report script reads it back out of
--                            #{pane_title}.
--
-- See docs/TMUX_INTEGRATION.md ("Neovim splits").

local M = {}

M.OPTION = "@spotlight_dimmer_nvim"
M.TITLE_MARKER = "sd-nvim="

local defaults = {
  enabled = true,
  report_script = "~/.config/SpotlightDimmer/tools/spotlight-dimmer-tmux-report.sh",
  -- Over ssh: set 'titlestring' to the title segment. Set to false when
  -- other code owns the title, and append M.title_segment() there instead.
  manage_title = true,
}

local config = vim.deepcopy(defaults)
-- Last value handed to tmux ("" = option unset), to skip redundant spawns.
local last_value = nil
local scheduled = false

local function is_floating(win)
  return vim.api.nvim_win_get_config(win).relative ~= ""
end

--- Cell rect of a window within the neovim grid, framed the way the tmux
--- report script frames tmux panes: the window's own statusline (or the
--- laststatus=3 separator) below it, and the vertical separators on both
--- sides, are included so the lines around the focused split light up too.
--- @param win integer|nil window handle (default: current window)
--- @return table|nil {col, row, width, height}, nil for floating windows
function M.compute_rect(win)
  win = win or vim.api.nvim_get_current_win()

  -- Floating windows (Telescope, pickers, ...) sit on top of the splits;
  -- spotlighting the whole pane keeps them from being dimmed.
  if is_floating(win) then
    return nil
  end

  local info = vim.fn.getwininfo(win)[1]
  if not info then
    return nil
  end

  local col = info.wincol - 1
  local row = info.winrow - 1
  local width = info.width
  -- winrow is the winbar row when there is one; height excludes it
  local height = info.height + (info.winbar or 0)

  -- The editor area ends above the command line, and above the global
  -- statusline with laststatus=3. Any window ending above that has a row of
  -- its own below it: its statusline, or the laststatus=3 separator.
  local area_bottom = vim.o.lines - vim.o.cmdheight
  if vim.o.laststatus == 3 then
    area_bottom = area_bottom - 1
  end
  if row + height < area_bottom then
    height = height + 1
  end

  -- Vertical separators: the one to the right belongs to this window, the
  -- one to the left to its neighbour, but both frame the focused split.
  if col + width < vim.o.columns then
    width = width + 1
  end
  if col > 0 then
    col = col - 1
    width = width + 1
  end

  return { col = col, row = row, width = width, height = height }
end

--- Number of splits (non-floating windows) in the current tabpage.
local function count_splits()
  local count = 0
  for _, win in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
    if not is_floating(win) then
      count = count + 1
    end
  end
  return count
end

--- Typing on the command line: ":", "/", or a prompt from input() (which is
--- what vim.ui.input() uses unless a plugin replaces it).
local function in_cmdline()
  return vim.api.nvim_get_mode().mode:sub(1, 1) == "c"
end

--- The pane option payload for the current window, or "" to unset it.
--- A tab with a single split has nothing to single out, so it unsets the
--- option too and the whole pane (command line included) stays lit.
--- So does command-line mode: the focused window does not change, but what
--- is being typed is on the bottom row, outside the split, or in a floating
--- popup anywhere over the editor (noice's cmdline_popup).
function M.payload()
  if count_splits() <= 1 or in_cmdline() then
    return ""
  end
  local rect = M.compute_rect()
  if not rect or rect.width <= 0 or rect.height <= 0 then
    return ""
  end
  return string.format(
    "%d,%d,%d,%d,%d,%d",
    vim.o.columns, vim.o.lines, rect.col, rect.row, rect.width, rect.height
  )
end

--- The terminal-title transport's segment ("sd-nvim=<payload>"), or "" when
--- there is nothing to narrow (single split, floating window, command line).
--- For code that owns 'titlestring' and composes it: append this on the
--- events in M.EVENTS.
function M.title_segment()
  local value = M.payload()
  if value == "" then
    return ""
  end
  return M.TITLE_MARKER .. value
end

--- Store `value` in this pane's option and re-run the report script.
--- The report only runs while this pane is the active pane of its session's
--- active window: from anywhere else the script would report whichever pane
--- tmux considers current, and the tmux hooks already cover switching back.
local function send(value, sync)
  local pane = vim.env.TMUX_PANE
  local script = vim.fn.shellescape(vim.fn.expand(config.report_script))

  local cmd = { "tmux", "set-option", "-p", "-t", pane }
  if value == "" then
    vim.list_extend(cmd, { "-u", M.OPTION })
  else
    vim.list_extend(cmd, { M.OPTION, value })
  end
  vim.list_extend(cmd, {
    ";", "if-shell", "-F", "-t", pane,
    "#{&&:#{pane_active},#{window_active}}",
    "run-shell -b " .. script,
  })

  local ok, job = pcall(vim.system, cmd, { text = true })
  if ok and sync then
    pcall(job.wait, job, 500)
  end
end

function M.report(force)
  if not config.enabled then
    return
  end
  local value = M.payload()
  if value == last_value and not force then
    return
  end
  last_value = value
  send(value, false)
end

--- Coalesce bursts (a :vsplit fires WinNew, WinEnter, WinResized, ...) into
--- one report on the next event-loop tick, once the layout has settled.
local function schedule_report()
  if scheduled then
    return
  end
  scheduled = true
  vim.schedule(function()
    scheduled = false
    M.report(false)
  end)
end

local function clear(sync)
  last_value = ""
  send("", sync)
end

local LAYOUT_EVENTS = {
  "VimEnter", "WinEnter", "BufWinEnter", "WinResized", "VimResized", "TabEnter",
  -- fires before the window is gone; the deferred report sees the new layout
  "WinClosed",
  -- the command line lights the whole pane while it is open (see payload())
  "CmdlineEnter", "CmdlineLeave",
}
M.EVENTS = LAYOUT_EVENTS
local LAYOUT_OPTIONS = { "laststatus", "showtabline", "winbar", "cmdheight" }

--- Over ssh: keep the title segment current. neovim restores the terminal
--- title when it exits or is suspended, which takes the segment with it.
local function setup_title(group)
  vim.o.title = true
  local update = function()
    vim.schedule(function()
      vim.o.titlestring = M.title_segment()
    end)
  end
  vim.api.nvim_create_autocmd(LAYOUT_EVENTS, { group = group, callback = update })
  vim.api.nvim_create_autocmd("OptionSet", {
    group = group, pattern = LAYOUT_OPTIONS, callback = update,
  })
  if vim.v.vim_did_enter == 1 then
    update()
  end
end

function M.setup(opts)
  config = vim.tbl_deep_extend("force", vim.deepcopy(defaults), opts or {})
  if not config.enabled then
    return
  end

  local group = vim.api.nvim_create_augroup("SpotlightDimmer", { clear = true })

  -- Not inside tmux: over ssh, the local tmux can still be reached through
  -- the terminal title. Anywhere else there is no pane to narrow.
  if not vim.env.TMUX or not vim.env.TMUX_PANE then
    if vim.env.SSH_TTY and config.manage_title then
      setup_title(group)
    end
    return
  end

  vim.api.nvim_create_autocmd(LAYOUT_EVENTS, { group = group, callback = schedule_report })
  vim.api.nvim_create_autocmd("OptionSet", {
    group = group, pattern = LAYOUT_OPTIONS, callback = schedule_report,
  })

  -- Suspended (Ctrl-Z) or gone: the shell underneath owns the pane again.
  vim.api.nvim_create_autocmd("VimSuspend", {
    group = group,
    callback = function() clear(true) end,
  })
  vim.api.nvim_create_autocmd("VimLeavePre", {
    group = group,
    callback = function() clear(true) end,
  })
  vim.api.nvim_create_autocmd("VimResume", {
    group = group,
    callback = function() M.report(true) end,
  })

  -- setup() may run after VimEnter (lazy loading)
  if vim.v.vim_did_enter == 1 then
    schedule_report()
  end
end

return M
