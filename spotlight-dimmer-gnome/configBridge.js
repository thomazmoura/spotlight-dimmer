/**
 * SpotlightDimmer - Configuration Bridge
 *
 * Loads configuration from the shared config file and watches for changes.
 * Config location: ~/.config/SpotlightDimmer/config.json
 */

import GObject from 'gi://GObject';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';

const CONFIG_DIR = 'SpotlightDimmer';
const CONFIG_FILE = 'config.json';

/**
 * ConfigBridge loads and watches the SpotlightDimmer configuration file.
 * Emits 'config-changed' signal when configuration is updated.
 */
export const ConfigBridge = GObject.registerClass({
    Signals: {
        'config-changed': {},
    },
}, class ConfigBridge extends GObject.Object {
    _init() {
        super._init();

        // Build config file path: ~/.config/SpotlightDimmer/config.json
        this._configPath = GLib.build_filenamev([
            GLib.get_user_config_dir(),
            CONFIG_DIR,
            CONFIG_FILE,
        ]);

        this._config = this._getDefaultConfig();
        this._monitor = null;
        this._debounceSourceId = null;

        this._loadConfig();
        this._watchConfig();
    }

    /**
     * Get default configuration values.
     * Matches C# AppConfig defaults.
     * @private
     */
    _getDefaultConfig() {
        return {
            mode: 'FullScreen',
            inactiveColor: { r: 0, g: 0, b: 0 },
            inactiveOpacity: 153, // ~60%
            activeColor: { r: 0, g: 0, b: 0 },
            activeOpacity: 102, // ~40%
        };
    }

    /**
     * Load configuration from file.
     * @private
     */
    _loadConfig() {
        try {
            const file = Gio.File.new_for_path(this._configPath);

            if (!file.query_exists(null)) {
                console.log('SpotlightDimmer: Config file not found, using defaults');
                console.log(`SpotlightDimmer: Expected path: ${this._configPath}`);
                return;
            }

            const [success, contents] = file.load_contents(null);
            if (!success) {
                console.log('SpotlightDimmer: Failed to read config file');
                return;
            }

            const decoder = new TextDecoder('utf-8');
            const json = decoder.decode(contents);
            const data = JSON.parse(json);

            this._parseConfig(data);
            console.log(`SpotlightDimmer: Config loaded - Mode: ${this._config.mode}`);
        } catch (e) {
            console.error(`SpotlightDimmer: Error loading config: ${e.message}`);
        }
    }

    /**
     * Parse configuration data from JSON object.
     * Matches C# AppConfig.Overlay structure.
     * @private
     */
    _parseConfig(data) {
        if (!data.Overlay) {
            return;
        }

        const overlay = data.Overlay;

        // Mode (FullScreen, Partial, PartialWithActive)
        if (overlay.Mode) {
            this._config.mode = overlay.Mode;
        }

        // Inactive color (hex string like "#000000")
        if (overlay.InactiveColor) {
            this._config.inactiveColor = this._parseHexColor(overlay.InactiveColor);
        }

        // Inactive opacity (0-255)
        if (typeof overlay.InactiveOpacity === 'number') {
            this._config.inactiveOpacity = this._clampOpacity(overlay.InactiveOpacity);
        }

        // Active color (hex string)
        if (overlay.ActiveColor) {
            this._config.activeColor = this._parseHexColor(overlay.ActiveColor);
        }

        // Active opacity (0-255)
        if (typeof overlay.ActiveOpacity === 'number') {
            this._config.activeOpacity = this._clampOpacity(overlay.ActiveOpacity);
        }
    }

    /**
     * Parse a hex color string (#RRGGBB) to {r, g, b} object.
     * @private
     */
    _parseHexColor(hex) {
        // Remove # prefix if present
        hex = hex.replace('#', '');

        // Validate length
        if (hex.length !== 6) {
            console.warn(`SpotlightDimmer: Invalid color format: ${hex}`);
            return { r: 0, g: 0, b: 0 };
        }

        return {
            r: parseInt(hex.substring(0, 2), 16) || 0,
            g: parseInt(hex.substring(2, 4), 16) || 0,
            b: parseInt(hex.substring(4, 6), 16) || 0,
        };
    }

    /**
     * Clamp opacity value to valid range.
     * @private
     */
    _clampOpacity(value) {
        return Math.min(255, Math.max(0, Math.round(value)));
    }

    /**
     * Watch config file for changes.
     * @private
     */
    _watchConfig() {
        try {
            const file = Gio.File.new_for_path(this._configPath);
            const parent = file.get_parent();

            // Watch the directory (file might not exist yet)
            // This allows us to detect when the file is created
            if (parent) {
                this._monitor = parent.monitor_directory(
                    Gio.FileMonitorFlags.NONE,
                    null
                );
            } else {
                this._monitor = file.monitor_file(
                    Gio.FileMonitorFlags.NONE,
                    null
                );
            }

            this._monitor.connect('changed', (monitor, changedFile, otherFile, eventType) => {
                // Only react to our config file
                const basename = changedFile.get_basename();
                if (basename !== CONFIG_FILE) {
                    return;
                }

                if (eventType === Gio.FileMonitorEvent.CHANGED ||
                    eventType === Gio.FileMonitorEvent.CREATED) {
                    this._onConfigFileChanged();
                }
            });

            console.log('SpotlightDimmer: Watching config directory for changes');
        } catch (e) {
            console.error(`SpotlightDimmer: Error watching config: ${e.message}`);
        }
    }

    /**
     * Handle config file change event with debouncing.
     * @private
     */
    _onConfigFileChanged() {
        // Debounce rapid changes (editors may save multiple times)
        if (this._debounceSourceId) {
            GLib.source_remove(this._debounceSourceId);
        }

        this._debounceSourceId = GLib.timeout_add(
            GLib.PRIORITY_DEFAULT,
            100, // 100ms debounce
            () => {
                this._debounceSourceId = null;
                this._loadConfig();
                this.emit('config-changed');
                return GLib.SOURCE_REMOVE;
            }
        );
    }

    /**
     * Get the current configuration.
     * @returns {Object} Configuration object
     */
    getConfig() {
        return this._config;
    }

    /**
     * Get the config file path.
     * @returns {string} Config file path
     */
    getConfigPath() {
        return this._configPath;
    }

    /**
     * Clean up resources.
     */
    destroy() {
        if (this._debounceSourceId) {
            GLib.source_remove(this._debounceSourceId);
            this._debounceSourceId = null;
        }

        if (this._monitor) {
            this._monitor.cancel();
            this._monitor = null;
        }
    }
});
