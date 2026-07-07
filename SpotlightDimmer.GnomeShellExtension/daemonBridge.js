/**
 * SpotlightDimmer - Daemon Bridge
 *
 * D-Bus client for the spotlight-dimmer-daemon. The extension is a thin
 * adapter: it reports focus/geometry/monitor changes to the daemon (which
 * owns configuration, overlay calculation and the wezterm/tmux integration)
 * and renders the overlay definitions the daemon publishes back, since GNOME
 * has no layer-shell for the daemon to render with itself.
 *
 * The daemon is D-Bus-activatable: watching its name with AUTO_START (and
 * every adapter method call) starts it on demand, so no manual ordering is
 * needed. If it crashes, systemd restarts it and the name-watch re-registers.
 */

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

const DAEMON_NAME = 'org.spotlightdimmer.Daemon';
const DAEMON_PATH = '/org/spotlightdimmer/Daemon';

const ADAPTER_INTERFACE_XML = `
<node>
  <interface name="org.spotlightdimmer.Adapter1">
    <method name="RegisterAdapter">
      <arg type="s" name="compositor" direction="in"/>
      <arg type="a{sv}" name="capabilities" direction="in"/>
      <arg type="u" name="protocolVersion" direction="out"/>
    </method>
    <method name="UpdateMonitors">
      <arg type="s" name="monitorsJson" direction="in"/>
    </method>
    <method name="FocusChanged">
      <arg type="s" name="wmClass" direction="in"/>
      <arg type="s" name="title" direction="in"/>
      <arg type="i" name="x" direction="in"/>
      <arg type="i" name="y" direction="in"/>
      <arg type="i" name="width" direction="in"/>
      <arg type="i" name="height" direction="in"/>
    </method>
    <method name="FocusCleared"/>
    <method name="GeometryChanged">
      <arg type="i" name="x" direction="in"/>
      <arg type="i" name="y" direction="in"/>
      <arg type="i" name="width" direction="in"/>
      <arg type="i" name="height" direction="in"/>
    </method>
    <method name="TitleChanged">
      <arg type="s" name="title" direction="in"/>
    </method>
  </interface>
</node>`;

const RENDERER_INTERFACE_XML = `
<node>
  <interface name="org.spotlightdimmer.Renderer1">
    <method name="RegisterRenderer">
      <arg type="s" name="currentOverlaysJson" direction="out"/>
    </method>
    <signal name="OverlaysChanged">
      <arg type="u" name="serial"/>
      <arg type="s" name="overlaysJson"/>
    </signal>
  </interface>
</node>`;

const DAEMON_INTERFACE_XML = `
<node>
  <interface name="org.spotlightdimmer.Daemon1">
    <method name="Toggle">
      <arg type="b" name="enabled" direction="out"/>
    </method>
  </interface>
</node>`;

const AdapterProxy = Gio.DBusProxy.makeProxyWrapper(ADAPTER_INTERFACE_XML);
const RendererProxy = Gio.DBusProxy.makeProxyWrapper(RENDERER_INTERFACE_XML);
const DaemonProxy = Gio.DBusProxy.makeProxyWrapper(DAEMON_INTERFACE_XML);

/**
 * DaemonBridge owns the D-Bus proxies and the daemon name watch.
 *
 * Callbacks (set before init()):
 * - onOverlays(payload): parsed OverlaysPayload to render (fresh serial only)
 * - onDaemonAppeared(): daemon (re)appeared; caller should re-send monitors
 *   and current focus after registration completes
 * - onDaemonVanished(): daemon gone; caller should hide all overlays
 */
export class DaemonBridge {
    constructor() {
        this.onOverlays = null;
        this.onDaemonAppeared = null;
        this.onDaemonVanished = null;

        this._adapter = null;
        this._renderer = null;
        this._daemon = null;
        this._watchId = null;
        this._overlaysSignalId = null;
        this._lastSerial = 0;
        this._destroyed = false;
    }

    /**
     * Create proxies and start watching the daemon name.
     * AUTO_START activates the daemon if it is installed but not running.
     */
    init() {
        // Proxies are created eagerly and survive daemon restarts; method
        // calls on them auto-start the (activatable) daemon as needed.
        this._adapter = new AdapterProxy(
            Gio.DBus.session, DAEMON_NAME, DAEMON_PATH,
            (proxy, error) => {
                if (error) {
                    console.warn(`SpotlightDimmer: adapter proxy init failed: ${error.message}`);
                }
            }
        );

        this._renderer = new RendererProxy(
            Gio.DBus.session, DAEMON_NAME, DAEMON_PATH,
            (proxy, error) => {
                if (error) {
                    console.warn(`SpotlightDimmer: renderer proxy init failed: ${error.message}`);
                    return;
                }
                this._overlaysSignalId = proxy.connectSignal(
                    'OverlaysChanged',
                    (_proxy, _sender, [serial, overlaysJson]) => {
                        this._handleOverlaysJson(overlaysJson, serial);
                    }
                );
            }
        );

        this._daemon = new DaemonProxy(
            Gio.DBus.session, DAEMON_NAME, DAEMON_PATH,
            (proxy, error) => {
                if (error) {
                    console.warn(`SpotlightDimmer: daemon proxy init failed: ${error.message}`);
                }
            }
        );

        this._watchId = Gio.bus_watch_name(
            Gio.BusType.SESSION,
            DAEMON_NAME,
            Gio.BusNameWatcherFlags.AUTO_START,
            () => {
                console.log('SpotlightDimmer: daemon appeared');
                // A (re)started daemon has no adapter state and serial starts
                // over; reset the stale-serial guard before re-registering.
                this._lastSerial = 0;
                this.onDaemonAppeared?.();
            },
            () => {
                console.warn('SpotlightDimmer: daemon vanished (crashed or not installed)');
                this.onDaemonVanished?.();
            }
        );
    }

    /**
     * Register this extension as the "gnome" adapter that renders overlays
     * itself, then fetch the current overlays snapshot and render it.
     * Called on every daemon (re)appearance.
     */
    async register() {
        try {
            const capabilities = {
                renders_overlays: GLib.Variant.new_boolean(true),
            };
            const [version] = await this._adapter.RegisterAdapterAsync('gnome', capabilities);
            console.log(`SpotlightDimmer: registered with daemon (protocol v${version})`);
        } catch (e) {
            console.warn(`SpotlightDimmer: RegisterAdapter failed: ${e.message}`);
            return;
        }

        try {
            const [snapshot] = await this._renderer.RegisterRendererAsync();
            this._handleOverlaysJson(snapshot, null);
        } catch (e) {
            console.warn(`SpotlightDimmer: RegisterRenderer failed: ${e.message}`);
        }
    }

    /**
     * Send the full monitor list (as JSON — the contract's KWin-safe format).
     * @param {Array} monitors - [{key, geometry, workArea, scale}] with rects
     *   as {x, y, width, height} in logical global coordinates
     */
    updateMonitors(monitors) {
        this._adapter?.UpdateMonitorsAsync(JSON.stringify(monitors)).catch(
            e => console.warn(`SpotlightDimmer: UpdateMonitors failed: ${e.message}`));
    }

    focusChanged(wmClass, title, rect) {
        this._adapter?.FocusChangedAsync(
            wmClass, title, rect.x, rect.y, rect.width, rect.height
        ).catch(e => console.warn(`SpotlightDimmer: FocusChanged failed: ${e.message}`));
    }

    focusCleared() {
        this._adapter?.FocusClearedAsync().catch(
            e => console.warn(`SpotlightDimmer: FocusCleared failed: ${e.message}`));
    }

    geometryChanged(rect) {
        this._adapter?.GeometryChangedAsync(
            rect.x, rect.y, rect.width, rect.height
        ).catch(e => console.warn(`SpotlightDimmer: GeometryChanged failed: ${e.message}`));
    }

    titleChanged(title) {
        this._adapter?.TitleChangedAsync(title).catch(
            e => console.warn(`SpotlightDimmer: TitleChanged failed: ${e.message}`));
    }

    /** Toggle dimming on/off (bound to the keyboard shortcut). */
    toggle() {
        this._daemon?.ToggleAsync().catch(
            e => console.warn(`SpotlightDimmer: Toggle failed: ${e.message}`));
    }

    /**
     * Parse a payload and forward it when fresh. `serial` from the signal is
     * checked against the payload for cheap stale filtering; a null serial
     * (RegisterRenderer snapshot) always applies and re-bases the guard.
     * @private
     */
    _handleOverlaysJson(overlaysJson, serial) {
        if (this._destroyed) {
            return;
        }

        let payload;
        try {
            payload = JSON.parse(overlaysJson);
        } catch (e) {
            console.warn(`SpotlightDimmer: invalid overlays payload: ${e.message}`);
            return;
        }

        if (serial !== null && payload.serial <= this._lastSerial) {
            return; // stale signal from before a reconnect
        }
        this._lastSerial = payload.serial;

        this.onOverlays?.(payload);
    }

    destroy() {
        this._destroyed = true;

        if (this._watchId) {
            Gio.bus_unwatch_name(this._watchId);
            this._watchId = null;
        }

        if (this._overlaysSignalId && this._renderer) {
            this._renderer.disconnectSignal(this._overlaysSignalId);
            this._overlaysSignalId = null;
        }

        this._adapter = null;
        this._renderer = null;
        this._daemon = null;
    }
}
