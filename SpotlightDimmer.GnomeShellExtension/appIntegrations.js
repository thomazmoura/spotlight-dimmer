/**
 * SpotlightDimmer - App Integrations
 *
 * Allows the spotlight to target a region INSIDE the focused window for
 * configured applications. The first provider is "tmux": when the focused
 * window is a terminal (e.g. WezTerm) whose visible content is a tmux client,
 * the spotlight rect becomes the focused tmux pane instead of the whole window.
 *
 * Data flow:
 * - tmux hooks run tools/spotlight-dimmer-tmux-report.sh, which pushes the
 *   focused pane's pixel rect (relative to the terminal content origin) to the
 *   D-Bus interface exported here, keyed by the tmux client's tty.
 * - On focus/title changes, extension.js calls setFocusedWindow(). If the
 *   window's WM_CLASS matches a configured integration, `wezterm cli` is
 *   queried asynchronously to find the focused wezterm pane's tty, and
 *   `tmux list-clients` verifies a live tmux client is attached to that tty.
 * - getPaneRect() joins both sides: reported pane rect + window frame rect +
 *   configured content offsets. It returns null whenever anything is missing,
 *   so callers always fall back to the whole-window behavior.
 */

import GObject from 'gi://GObject';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';

const DBUS_NAME = 'org.spotlightdimmer.PaneTracker';
const DBUS_PATH = '/org/spotlightdimmer/PaneTracker';

const DBUS_INTERFACE_XML = `
<node>
  <interface name="org.spotlightdimmer.PaneTracker">
    <method name="UpdatePaneGeometry">
      <arg type="s" name="tty" direction="in"/>
      <arg type="i" name="x" direction="in"/>
      <arg type="i" name="y" direction="in"/>
      <arg type="i" name="width" direction="in"/>
      <arg type="i" name="height" direction="in"/>
    </method>
    <method name="ClearPane">
      <arg type="s" name="tty" direction="in"/>
    </method>
  </interface>
</node>`;

// Debounce for window title changes (terminals update titles frequently)
const TITLE_DEBOUNCE_MS = 150;

/**
 * AppIntegrations exports the PaneTracker D-Bus interface and resolves the
 * focused inner-region rect for windows with a configured integration.
 * Emits 'pane-rect-changed' (no payload) whenever the resolved rect may have
 * changed; callers should re-read via getPaneRect().
 */
export const AppIntegrations = GObject.registerClass({
    Signals: {
        'pane-rect-changed': {},
    },
}, class AppIntegrations extends GObject.Object {
    _init(configBridge) {
        super._init();

        this._configBridge = configBridge;

        // Latest pane rect per tmux client tty, pixels relative to the
        // terminal content origin: Map<tty, {x, y, width, height}>
        this._paneDataByTty = new Map();

        // Focused-window integration state
        this._focusedWindow = null;
        this._integration = null;     // matched config entry or null
        this._activePane = null;      // {tty, offsetX, offsetY} from wezterm or null

        // Invalidates in-flight async queries when focus moves on
        this._queryGeneration = 0;

        this._titleSignalId = null;
        this._titleDebounceId = null;

        this._ownerId = null;
        this._dbusObject = null;
        this._exportDbus();
    }

    /**
     * Export the PaneTracker D-Bus interface on the session bus.
     * @private
     */
    _exportDbus() {
        try {
            this._dbusObject = Gio.DBusExportedObject.wrapJSObject(DBUS_INTERFACE_XML, this);
            this._ownerId = Gio.bus_own_name(
                Gio.BusType.SESSION,
                DBUS_NAME,
                Gio.BusNameOwnerFlags.NONE,
                (connection) => {
                    this._dbusObject.export(connection, DBUS_PATH);
                    console.log(`SpotlightDimmer: D-Bus service exported at ${DBUS_NAME}`);
                },
                null,
                () => console.warn(`SpotlightDimmer: Failed to acquire D-Bus name ${DBUS_NAME}`)
            );
        } catch (e) {
            console.error(`SpotlightDimmer: Error exporting D-Bus interface: ${e.message}`);
        }
    }

    /**
     * D-Bus method: receive the focused pane rect for a tmux client.
     * Coordinates are pixels relative to the terminal content origin.
     */
    UpdatePaneGeometry(tty, x, y, width, height) {
        if (!tty || width <= 0 || height <= 0) {
            return;
        }

        this._paneDataByTty.set(tty, { x, y, width, height });
        this.emit('pane-rect-changed');
    }

    /**
     * D-Bus method: forget the pane rect for a tmux client.
     */
    ClearPane(tty) {
        if (this._paneDataByTty.delete(tty)) {
            this.emit('pane-rect-changed');
        }
    }

    /**
     * Update integration state for a newly focused window.
     * Called by extension.js on focus changes.
     * @param {Meta.Window|null} window - The focused window
     */
    setFocusedWindow(window) {
        this._queryGeneration++;
        this._disconnectTitleSignal();

        this._focusedWindow = window;
        this._integration = this._matchIntegration(window);
        this._activePane = null;

        if (!this._integration) {
            this.emit('pane-rect-changed');
            return;
        }

        // Re-resolve when the terminal title changes: wezterm tab switches and
        // tmux attach/detach/exit all change the title without any Mutter
        // focus or geometry event.
        this._titleSignalId = window.connect('notify::title', () => {
            this._onTitleChanged();
        });

        this._queryWezTerm();
    }

    /**
     * Compute the screen-space rect of the focused inner region (tmux pane).
     * @param {Object} frameRect - Focused window frame rect {x, y, width, height}
     * @returns {Object|null} Screen-space rect, or null to use the whole window
     */
    getPaneRect(frameRect) {
        if (!this._integration || !this._activePane || !frameRect) {
            return null;
        }

        const paneData = this._paneDataByTty.get(this._activePane.tty);
        if (!paneData) {
            return null;
        }

        const originX = frameRect.x + this._integration.contentOffsetX + this._activePane.offsetX;
        const originY = frameRect.y + this._integration.contentOffsetY + this._activePane.offsetY;

        // Clamp to the window frame; a dimming overlay does not need to be
        // pixel-perfect, but it must never highlight outside the window.
        const left = Math.max(originX + paneData.x, frameRect.x);
        const top = Math.max(originY + paneData.y, frameRect.y);
        const right = Math.min(originX + paneData.x + paneData.width, frameRect.x + frameRect.width);
        const bottom = Math.min(originY + paneData.y + paneData.height, frameRect.y + frameRect.height);

        if (right - left <= 0 || bottom - top <= 0) {
            return null;
        }

        return { x: left, y: top, width: right - left, height: bottom - top };
    }

    /**
     * Find the configured integration for a window's WM_CLASS.
     * @private
     */
    _matchIntegration(window) {
        if (!window) {
            return null;
        }

        let wmClass = null;
        try {
            wmClass = window.get_wm_class();
        } catch (e) {
            return null;
        }

        if (!wmClass) {
            return null;
        }

        const integrations = this._configBridge.getConfig().appIntegrations || [];
        return integrations.find(i => i.wmClass === wmClass && i.provider === 'tmux') || null;
    }

    /**
     * Handle focused-window title changes with debouncing.
     * @private
     */
    _onTitleChanged() {
        if (this._titleDebounceId) {
            GLib.source_remove(this._titleDebounceId);
        }

        this._titleDebounceId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, TITLE_DEBOUNCE_MS, () => {
            this._titleDebounceId = null;
            if (this._integration) {
                this._queryWezTerm();
            }
            return GLib.SOURCE_REMOVE;
        });
    }

    /**
     * Resolve the focused wezterm pane and verify it hosts a live tmux client.
     *
     * Three async subprocesses, chained; all failures resolve to "no active
     * pane" so the caller falls back to whole-window highlighting:
     * 1. `wezterm cli list-clients` -> focused_pane_id
     * 2. `wezterm cli list`         -> that pane's tty_name and cell origin
     * 3. `tmux list-clients`        -> tty must belong to a live tmux client
     * @private
     */
    _queryWezTerm() {
        const generation = ++this._queryGeneration;

        this._spawnJson(['wezterm', 'cli', 'list-clients', '--format', 'json'], clients => {
            if (generation !== this._queryGeneration) return;

            const client = Array.isArray(clients)
                ? clients.find(c => c.focused_pane_id !== null && c.focused_pane_id !== undefined)
                : null;
            if (!client) {
                this._setActivePane(null);
                return;
            }

            this._spawnJson(['wezterm', 'cli', 'list', '--format', 'json'], panes => {
                if (generation !== this._queryGeneration) return;

                const pane = Array.isArray(panes)
                    ? panes.find(p => p.pane_id === client.focused_pane_id)
                    : null;
                if (!pane || !pane.tty_name) {
                    this._setActivePane(null);
                    return;
                }

                // Offset of this wezterm pane's cell grid within the window,
                // for wezterm-native splits (0 when tmux fills the tab).
                // Cell pixel size is derived from the pane's own reported size.
                const cellW = pane.size?.cols ? pane.size.pixel_width / pane.size.cols : 0;
                const cellH = pane.size?.rows ? pane.size.pixel_height / pane.size.rows : 0;
                const offsetX = Math.round((pane.left_col || 0) * cellW);
                const offsetY = Math.round((pane.top_row || 0) * cellH);

                this._spawnLines(['tmux', 'list-clients', '-F', '#{client_tty}'], ttys => {
                    if (generation !== this._queryGeneration) return;

                    if (ttys && ttys.includes(pane.tty_name)) {
                        this._setActivePane({ tty: pane.tty_name, offsetX, offsetY });
                    } else {
                        // Focused wezterm pane is not a tmux client (e.g. a
                        // plain shell tab, or the tmux server is gone).
                        this._setActivePane(null);
                    }
                });
            });
        });
    }

    /**
     * Set the resolved active pane state and notify listeners.
     * @private
     */
    _setActivePane(pane) {
        this._activePane = pane;
        this.emit('pane-rect-changed');
    }

    /**
     * Spawn a subprocess and parse its stdout as JSON (null on any failure).
     * @private
     */
    _spawnJson(argv, callback) {
        this._spawn(argv, stdout => {
            if (stdout === null) {
                callback(null);
                return;
            }
            try {
                callback(JSON.parse(stdout));
            } catch (e) {
                callback(null);
            }
        });
    }

    /**
     * Spawn a subprocess and split its stdout into trimmed lines (null on failure).
     * @private
     */
    _spawnLines(argv, callback) {
        this._spawn(argv, stdout => {
            if (stdout === null) {
                callback(null);
                return;
            }
            callback(stdout.split('\n').map(l => l.trim()).filter(l => l.length > 0));
        });
    }

    /**
     * Spawn a subprocess asynchronously; callback receives stdout or null.
     * Never blocks the shell.
     * @private
     */
    _spawn(argv, callback) {
        let proc;
        try {
            proc = Gio.Subprocess.new(
                argv,
                Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_SILENCE
            );
        } catch (e) {
            // Binary not installed / not in PATH
            callback(null);
            return;
        }

        proc.communicate_utf8_async(null, null, (p, result) => {
            try {
                const [, stdout] = p.communicate_utf8_finish(result);
                callback(p.get_successful() ? stdout : null);
            } catch (e) {
                callback(null);
            }
        });
    }

    /**
     * Disconnect the notify::title handler from the previous window.
     * @private
     */
    _disconnectTitleSignal() {
        if (this._titleDebounceId) {
            GLib.source_remove(this._titleDebounceId);
            this._titleDebounceId = null;
        }

        if (this._titleSignalId && this._focusedWindow) {
            try {
                this._focusedWindow.disconnect(this._titleSignalId);
            } catch (e) {
                // Window may already be destroyed - this is normal
            }
        }
        this._titleSignalId = null;
    }

    /**
     * Clean up resources.
     */
    destroy() {
        this._queryGeneration++;
        this._disconnectTitleSignal();
        this._focusedWindow = null;

        if (this._ownerId) {
            Gio.bus_unown_name(this._ownerId);
            this._ownerId = null;
        }

        if (this._dbusObject) {
            try {
                this._dbusObject.unexport();
            } catch (e) {
                // Already unexported when the bus name was released
            }
            this._dbusObject = null;
        }

        this._paneDataByTty.clear();
    }
});
