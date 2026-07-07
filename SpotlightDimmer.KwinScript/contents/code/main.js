/**
 * SpotlightDimmer - KWin Script (Plasma 6)
 *
 * Compositor adapter for the spotlight-dimmer-daemon: reports monitors,
 * focus, geometry and title changes over D-Bus. The daemon computes overlay
 * geometry and renders the dimming itself via layer-shell (KWin supports
 * wlr-layer-shell, so no compositor-side rendering is needed here).
 *
 * KWin scripts can only make D-Bus calls, not own bus names, so the whole
 * adapter contract is call-based. The very first callDBus D-Bus-activates
 * the daemon, which also solves startup ordering; if the daemon crashes and
 * restarts, the next call re-activates it and the daemon treats calls from
 * an unknown sender as implicit re-registration.
 *
 * Wire-format note: all argument types are basic (strings, int32) because
 * KWin's callDBus cannot marshal nested D-Bus structs — monitors travel as
 * a JSON string, rects as four ints. `|0` forces integral JS numbers so the
 * QJS engine stores them as int32 and QDBus marshals them as 'i'.
 */

const SERVICE = "org.spotlightdimmer.Daemon";
const OBJECT_PATH = "/org/spotlightdimmer/Daemon";
const ADAPTER_IFACE = "org.spotlightdimmer.Adapter1";

let trackedWindow = null;
let trackedHandlers = null;

function adapterCall(method, ...args) {
    try {
        callDBus(SERVICE, OBJECT_PATH, ADAPTER_IFACE, method, ...args);
    } catch (e) {
        print(`SpotlightDimmer: D-Bus call ${method} failed: ${e}`);
    }
}

function roundRect(g) {
    return {
        x: Math.round(g.x) | 0,
        y: Math.round(g.y) | 0,
        width: Math.round(g.width) | 0,
        height: Math.round(g.height) | 0,
    };
}

function sendMonitors() {
    const monitors = [];

    for (const output of workspace.screens) {
        const workArea = workspace.clientArea(
            KWin.PlacementArea,
            output,
            workspace.currentDesktop
        );

        monitors.push({
            key: output.name,
            geometry: roundRect(output.geometry),
            workArea: roundRect(workArea),
            scale: output.scale,
        });
    }

    adapterCall("UpdateMonitors", JSON.stringify(monitors));
}

/**
 * Only windows KWin actually activates reach sendFocus/trackWindow, so a
 * small blocklist beats enumerating popup types: desktop and dock
 * activations mean "no spotlight target", everything else (the Kickoff
 * applet popup, KRunner, dialogs...) is where the user's attention is.
 */
function isSpotlightTarget(window) {
    return window && !window.desktopWindow && !window.dock;
}

function sendFocus(window) {
    if (!isSpotlightTarget(window)) {
        adapterCall("FocusCleared");
        return;
    }

    const frame = roundRect(window.frameGeometry);
    adapterCall(
        "FocusChanged",
        window.resourceClass ?? "",
        window.caption ?? "",
        frame.x, frame.y, frame.width, frame.height
    );
}

function sendGeometry(window) {
    const frame = roundRect(window.frameGeometry);
    adapterCall("GeometryChanged", frame.x, frame.y, frame.width, frame.height);
}

/**
 * Track only the active window: geometry for overlay placement, caption for
 * the daemon's wezterm/tmux requery (tmux attach/detach and tab switches
 * change the caption without any focus/geometry event), fullscreen because
 * frame geometry may settle after the state flips.
 */
function trackWindow(window) {
    untrackWindow();

    if (!isSpotlightTarget(window)) {
        return;
    }

    const handlers = {
        geometry: () => sendGeometry(window),
        caption: () => adapterCall("TitleChanged", window.caption ?? ""),
        fullScreen: () => sendGeometry(window),
    };

    window.frameGeometryChanged.connect(handlers.geometry);
    window.captionChanged.connect(handlers.caption);
    window.fullScreenChanged.connect(handlers.fullScreen);

    trackedWindow = window;
    trackedHandlers = handlers;
}

function untrackWindow() {
    if (trackedWindow && trackedHandlers) {
        try {
            trackedWindow.frameGeometryChanged.disconnect(trackedHandlers.geometry);
            trackedWindow.captionChanged.disconnect(trackedHandlers.caption);
            trackedWindow.fullScreenChanged.disconnect(trackedHandlers.fullScreen);
        } catch (e) {
            // Window may already be gone - this is normal
        }
    }

    trackedWindow = null;
    trackedHandlers = null;
}

function register() {
    // The reply callback confirms the daemon is up before the initial sync
    try {
        callDBus(
            SERVICE, OBJECT_PATH, ADAPTER_IFACE,
            "RegisterAdapter", "kwin", {},
            (protocolVersion) => {
                print(`SpotlightDimmer: registered with daemon (protocol v${protocolVersion})`);
                sendMonitors();
                const active = workspace.activeWindow;
                trackWindow(active);
                sendFocus(active);
            }
        );
    } catch (e) {
        print(`SpotlightDimmer: RegisterAdapter failed: ${e}`);
    }
}

workspace.windowActivated.connect((window) => {
    trackWindow(window);
    sendFocus(window);
});

workspace.screensChanged.connect(sendMonitors);

// Work areas change when panels move/resize even if screens don't
if (workspace.virtualScreenGeometryChanged !== undefined) {
    workspace.virtualScreenGeometryChanged.connect(sendMonitors);
}

register();
print("SpotlightDimmer: KWin adapter loaded");
