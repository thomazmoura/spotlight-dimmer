// Function to create a working mock Tauri API for development
function createMockTauriAPI() {
    console.log('Creating mock Tauri API for development...');

    // Mock window manager that returns sample data
    const mockWindowManager = {
        async getActiveWindow() {
            return {
                handle: 12345,
                window_title: "Development Browser",
                process_name: "chrome.exe",
                display_id: "DISPLAY1"
            };
        },

        async getDisplays() {
            return [
                {
                    id: "DISPLAY1",
                    name: "Primary Display",
                    width: 1920,
                    height: 1080,
                    x: 0,
                    y: 0,
                    is_primary: true
                },
                {
                    id: "DISPLAY2",
                    name: "Secondary Display",
                    width: 1680,
                    height: 1050,
                    x: 1920,
                    y: 0,
                    is_primary: false
                }
            ];
        },

        dimmingEnabled: false,
        overlayColor: { r: 0, g: 0, b: 0, a: 0.5 },

        async toggleDimming() {
            this.dimmingEnabled = !this.dimmingEnabled;
            console.log('Mock: Dimming toggled to', this.dimmingEnabled);
            return this.dimmingEnabled;
        },

        async isDimmingEnabled() {
            return this.dimmingEnabled;
        },

        async getOverlayColor() {
            console.log('Mock: Getting overlay color', this.overlayColor);
            return this.overlayColor;
        },

        async setOverlayColor(color) {
            console.log('Mock: Setting overlay color to', color);
            this.overlayColor = { ...color };
            return;
        },

        async resetOverlayColor() {
            const defaultColor = { r: 0, g: 0, b: 0, a: 0.5 };
            console.log('Mock: Resetting overlay color to default', defaultColor);
            this.overlayColor = { ...defaultColor };
            return defaultColor;
        }
    };

    return {
        invoke: async function(command, args) {
            console.log('Mock Tauri invoke:', command, args);

            switch(command) {
                case 'get_displays':
                    return mockWindowManager.getDisplays();
                case 'get_active_window':
                    return mockWindowManager.getActiveWindow();
                case 'toggle_dimming':
                    return mockWindowManager.toggleDimming();
                case 'is_dimming_enabled':
                    return mockWindowManager.isDimmingEnabled();
                case 'get_overlay_color':
                    return mockWindowManager.getOverlayColor();
                case 'set_overlay_color':
                    return mockWindowManager.setOverlayColor(args.color);
                case 'reset_overlay_color':
                    return mockWindowManager.resetOverlayColor();
                default:
                    console.warn('Mock: Unknown command:', command);
                    return null;
            }
        },
        listen: async function(event, handler) {
            console.log('Mock Tauri listen:', event);
            // Simulate focus change events every 5 seconds for testing
            if (event === 'focus-changed') {
                setInterval(() => {
                    const mockData = {
                        active_window: {
                            handle: Math.floor(Math.random() * 10000),
                            window_title: "Mock Window " + Math.floor(Math.random() * 10),
                            process_name: "mock.exe",
                            display_id: Math.random() > 0.5 ? "DISPLAY1" : "DISPLAY2"
                        },
                        active_display: {
                            id: Math.random() > 0.5 ? "DISPLAY1" : "DISPLAY2",
                            name: Math.random() > 0.5 ? "Primary Display" : "Secondary Display",
                            width: 1920,
                            height: 1080,
                            x: 0,
                            y: 0,
                            is_primary: true
                        }
                    };
                    handler({ payload: mockData });
                }, 5000);
            }
            return function() { console.log('Mock: Unsubscribed from', event); };
        }
    };
}

// Function to wait for Tauri API to be available
function waitForTauri() {
    return new Promise((resolve, reject) => {
        let attempts = 0;
        const maxAttempts = 10; // Reduced wait time

        const checkTauri = () => {
            attempts++;

            if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.event) {
                console.log('Real Tauri API found after', attempts, 'attempts');
                resolve({
                    invoke: window.__TAURI__.core.invoke,
                    listen: window.__TAURI__.event.listen
                });
            } else if (attempts >= maxAttempts) {
                console.log('Real Tauri API not available, using mock API for development');
                resolve(createMockTauriAPI());
            } else {
                setTimeout(checkTauri, 100);
            }
        };

        checkTauri();
    });
}

// Global variables to store Tauri functions
let invoke, listen;

class SpotlightDimmerApp {
    constructor() {
        this.displays = [];
        this.activeWindow = null;
        this.activeDisplay = null;
        this.isDimmingEnabled = false;
        this.overlayColor = { r: 0, g: 0, b: 0, a: 0.5 };

        this.elements = {
            activeWindow: document.getElementById('activeWindow'),
            activeDisplay: document.getElementById('activeDisplay'),
            dimmingStatus: document.getElementById('dimmingStatus'),
            toggleBtn: document.getElementById('toggleBtn'),
            toggleText: document.getElementById('toggleText'),
            refreshBtn: document.getElementById('refreshBtn'),
            displaysList: document.getElementById('displaysList'),
            colorPicker: document.getElementById('colorPicker'),
            colorPreview: document.getElementById('colorPreview'),
            opacitySlider: document.getElementById('opacitySlider'),
            opacityValue: document.getElementById('opacityValue'),
            resetColorBtn: document.getElementById('resetColorBtn')
        };

        this.init();
    }

    async init() {
        try {
            console.log('Initializing Spotlight Dimmer...');
            console.log('Waiting for Tauri API...');

            // Wait for Tauri API to be available
            const tauriApi = await waitForTauri();
            invoke = tauriApi.invoke;
            listen = tauriApi.listen;

            console.log('Tauri API loaded successfully!');

            // Add detailed debugging for missing elements
            const requiredElements = ['activeWindow', 'activeDisplay', 'dimmingStatus', 'toggleBtn', 'refreshBtn', 'displaysList'];
            const foundElements = {};
            requiredElements.forEach(id => {
                const element = document.getElementById(id);
                foundElements[id] = !!element;
                if (!element) {
                    console.error(`Missing element: ${id}`);
                } else {
                    console.log(`Found element: ${id}`);
                }
            });
            console.log('Element status:', foundElements);

            // Set up event listeners
            this.setupEventListeners();

            // Listen for focus changes from the backend
            await this.setupFocusListener();

            // Test basic Tauri connectivity first
            await this.testTauriConnectivity();

            // Load initial state
            await this.loadDimmingStatus();
            await this.loadOverlayColor();
            await this.loadDisplays();
            await this.updateActiveWindow();

            console.log('App initialized successfully');
        } catch (error) {
            console.error('Failed to initialize app:', error);
            console.error('Error details:', error.message, error.stack);
            this.showError('Failed to initialize application: ' + error.message);
        }
    }

    setupEventListeners() {
        this.elements.toggleBtn.addEventListener('click', () => this.toggleDimming());
        this.elements.refreshBtn.addEventListener('click', () => this.refreshDisplays());

        // Color picker event listeners
        this.elements.colorPicker.addEventListener('input', (e) => this.onColorChange(e.target.value));
        this.elements.opacitySlider.addEventListener('input', (e) => this.onOpacityChange(e.target.value));
        this.elements.resetColorBtn.addEventListener('click', () => this.resetColor());
    }

    async testTauriConnectivity() {
        try {
            console.log('Testing Tauri connectivity...');

            if (!invoke) {
                throw new Error('Tauri invoke function not available');
            }

            // Test a simple command
            console.log('Attempting to call is_dimming_enabled...');
            const result = await invoke('is_dimming_enabled');
            console.log('Tauri connectivity test successful:', result);
            return true;
        } catch (error) {
            console.error('Tauri connectivity test failed:', error);
            console.error('Error details:', error.message, error.stack);
            this.showError('Failed to connect to Tauri backend: ' + error.message);
            return false;
        }
    }

    async setupFocusListener() {
        try {
            await listen('focus-changed', (event) => {
                console.log('Focus changed:', event.payload);
                this.handleFocusChange(event.payload);
            });
        } catch (error) {
            console.error('Failed to set up focus listener:', error);
        }
    }

    handleFocusChange(data) {
        this.activeWindow = data.active_window;
        this.activeDisplay = data.active_display;
        this.updateUI();
    }

    async loadDimmingStatus() {
        try {
            console.log('Loading dimming status...');
            this.isDimmingEnabled = await invoke('is_dimming_enabled');
            console.log('Dimming status loaded:', this.isDimmingEnabled);
            this.updateDimmingUI();
        } catch (error) {
            console.error('Failed to load dimming status:', error);
            console.error('Error details:', error.message);
        }
    }

    async loadDisplays() {
        try {
            console.log('Loading displays...');
            this.elements.displaysList.innerHTML = '<div class="loading">Loading displays...</div>';
            this.displays = await invoke('get_displays');
            console.log('Displays loaded:', this.displays);
            this.renderDisplays();
        } catch (error) {
            console.error('Failed to load displays:', error);
            console.error('Error details:', error.message);
            this.elements.displaysList.innerHTML =
                '<div style="color: var(--danger-color);">Failed to load displays: ' + error.message + '</div>';
        }
    }

    async updateActiveWindow() {
        try {
            console.log('Getting active window...');
            this.activeWindow = await invoke('get_active_window');
            console.log('Active window:', this.activeWindow);

            // Find the display for this window
            if (this.displays.length > 0) {
                this.activeDisplay = this.displays.find(d => d.id === this.activeWindow.display_id);
                console.log('Active display:', this.activeDisplay);
            }

            this.updateUI();
        } catch (error) {
            console.error('Failed to get active window:', error);
            console.error('Error details:', error.message);
            this.elements.activeWindow.textContent = 'Error: ' + error.message;
            this.elements.activeDisplay.textContent = 'Unknown';
        }
    }

    async toggleDimming() {
        try {
            this.elements.toggleBtn.disabled = true;
            this.elements.toggleText.textContent = 'Please wait...';

            this.isDimmingEnabled = await invoke('toggle_dimming');
            this.updateDimmingUI();

            // Refresh displays to update their status
            await this.loadDisplays();

        } catch (error) {
            console.error('Failed to toggle dimming:', error);
            this.showError('Failed to toggle dimming: ' + error);
        } finally {
            this.elements.toggleBtn.disabled = false;
        }
    }

    async refreshDisplays() {
        try {
            this.elements.refreshBtn.disabled = true;
            this.elements.refreshBtn.textContent = 'Refreshing...';

            await this.loadDisplays();
            await this.updateActiveWindow();

        } catch (error) {
            console.error('Failed to refresh displays:', error);
            this.showError('Failed to refresh displays: ' + error);
        } finally {
            this.elements.refreshBtn.disabled = false;
            this.elements.refreshBtn.textContent = 'Refresh Displays';
        }
    }

    updateUI() {
        // Update active window info
        if (this.activeWindow) {
            const windowText = this.activeWindow.window_title || 'Unknown Window';
            const processText = this.activeWindow.process_name || 'Unknown Process';
            this.elements.activeWindow.textContent = `${windowText} (${processText})`;
        } else {
            this.elements.activeWindow.textContent = 'No active window detected';
        }

        // Update active display info
        if (this.activeDisplay) {
            this.elements.activeDisplay.textContent =
                `${this.activeDisplay.name} (${this.activeDisplay.width}x${this.activeDisplay.height})`;
        } else {
            this.elements.activeDisplay.textContent = 'Unknown display';
        }

        // Re-render displays to update their status
        this.renderDisplays();
    }

    updateDimmingUI() {
        const status = this.elements.dimmingStatus;
        const btn = this.elements.toggleBtn;
        const text = this.elements.toggleText;

        if (this.isDimmingEnabled) {
            status.textContent = 'Enabled';
            status.className = 'value status-enabled';
            btn.className = 'primary-btn enabled';
            text.textContent = 'Disable Dimming';
        } else {
            status.textContent = 'Disabled';
            status.className = 'value status-disabled';
            btn.className = 'primary-btn';
            text.textContent = 'Enable Dimming';
        }
    }

    renderDisplays() {
        if (this.displays.length === 0) {
            this.elements.displaysList.innerHTML = '<div>No displays found</div>';
            return;
        }

        const displaysHTML = this.displays.map(display => {
            const isActive = this.activeDisplay && display.id === this.activeDisplay.id;
            const isDimmed = this.isDimmingEnabled && !isActive;

            let statusClass = 'normal';
            let statusText = 'Normal';

            if (isActive) {
                statusClass = 'active';
                statusText = 'Active (Focused)';
            } else if (isDimmed) {
                statusClass = 'dimmed';
                statusText = 'Dimmed';
            }

            const displayClass = isActive ? 'display-item active' :
                                 isDimmed ? 'display-item dimmed' : 'display-item';

            return `
                <div class="${displayClass}">
                    <div class="display-name">${display.name}</div>
                    <div class="display-details">Resolution: ${display.width} � ${display.height}</div>
                    <div class="display-details">Position: (${display.x}, ${display.y})</div>
                    ${display.is_primary ? '<div class="display-details">Primary Display</div>' : ''}
                    <div class="display-status ${statusClass}">${statusText}</div>
                </div>
            `;
        }).join('');

        this.elements.displaysList.innerHTML = displaysHTML;
    }

    showError(message) {
        // TODO: Implement proper error notification system
        console.error(message);
        alert(message); // Temporary solution
    }

    // Color management methods
    async loadOverlayColor() {
        try {
            console.log('Loading overlay color...');
            this.overlayColor = await invoke('get_overlay_color');
            console.log('Overlay color loaded:', this.overlayColor);
            this.updateColorUI();
        } catch (error) {
            console.error('Failed to load overlay color:', error);
            console.error('Error details:', error.message);
        }
    }

    updateColorUI() {
        // Update color picker
        const hexColor = this.rgbToHex(this.overlayColor.r, this.overlayColor.g, this.overlayColor.b);
        this.elements.colorPicker.value = hexColor;

        // Update opacity slider (convert from 0.0-1.0 to 0-100)
        const opacityPercent = Math.round(this.overlayColor.a * 100);
        this.elements.opacitySlider.value = opacityPercent;
        this.elements.opacityValue.textContent = opacityPercent + '%';

        // Update color preview
        const rgbaString = `rgba(${this.overlayColor.r}, ${this.overlayColor.g}, ${this.overlayColor.b}, ${this.overlayColor.a})`;
        this.elements.colorPreview.style.backgroundColor = rgbaString;

        console.log('Updated color UI with:', rgbaString);
    }

    async onColorChange(hexColor) {
        try {
            console.log('Color picker changed to:', hexColor);

            // Convert hex to RGB
            const rgb = this.hexToRgb(hexColor);
            if (rgb) {
                this.overlayColor.r = rgb.r;
                this.overlayColor.g = rgb.g;
                this.overlayColor.b = rgb.b;

                // Update preview immediately
                this.updateColorPreview();

                // Send to backend
                await invoke('set_overlay_color', { color: this.overlayColor });
                console.log('Successfully updated overlay color');
            }
        } catch (error) {
            console.error('Failed to update overlay color:', error);
            this.showError('Failed to update overlay color: ' + error.message);
        }
    }

    async onOpacityChange(value) {
        try {
            console.log('Opacity slider changed to:', value);

            // Convert from 0-100 to 0.0-1.0
            this.overlayColor.a = parseFloat(value) / 100;

            // Update UI
            this.elements.opacityValue.textContent = value + '%';
            this.updateColorPreview();

            // Send to backend
            await invoke('set_overlay_color', { color: this.overlayColor });
            console.log('Successfully updated overlay opacity');
        } catch (error) {
            console.error('Failed to update overlay opacity:', error);
            this.showError('Failed to update overlay opacity: ' + error.message);
        }
    }

    updateColorPreview() {
        const rgbaString = `rgba(${this.overlayColor.r}, ${this.overlayColor.g}, ${this.overlayColor.b}, ${this.overlayColor.a})`;
        this.elements.colorPreview.style.backgroundColor = rgbaString;
    }

    async resetColor() {
        try {
            console.log('Resetting color to default...');
            this.overlayColor = await invoke('reset_overlay_color');
            this.updateColorUI();
            console.log('Successfully reset overlay color');
        } catch (error) {
            console.error('Failed to reset overlay color:', error);
            this.showError('Failed to reset overlay color: ' + error.message);
        }
    }

    // Utility functions
    hexToRgb(hex) {
        const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result ? {
            r: parseInt(result[1], 16),
            g: parseInt(result[2], 16),
            b: parseInt(result[3], 16)
        } : null;
    }

    rgbToHex(r, g, b) {
        return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1);
    }
}

// Initialize the app when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    console.log('DOM loaded, initializing app...');

    // Check if all required elements are present
    const requiredElements = ['activeWindow', 'activeDisplay', 'dimmingStatus', 'toggleBtn', 'refreshBtn', 'displaysList'];
    const missingElements = requiredElements.filter(id => !document.getElementById(id));

    if (missingElements.length > 0) {
        console.error('Missing required HTML elements:', missingElements);
    } else {
        console.log('All required elements found, creating app...');
    }

    new SpotlightDimmerApp();
});

// Handle app errors
window.addEventListener('error', (event) => {
    console.error('Global error:', event.error);
});

window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason);
});