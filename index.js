#!/usr/bin/env node

/**
 * Spotlight Dimmer - Main entry point
 *
 * This package provides native Windows binaries accessible via:
 * - spotlight-dimmer (main application)
 * - spotlight-dimmer-config (configuration tool)
 *
 * This file is the npm package main entry point but doesn't execute anything.
 * Users should run the binaries directly via the commands above.
 */

console.log(`
Spotlight Dimmer - Windows Display Dimming Tool

This package provides two commands:

  spotlight-dimmer              Start the application
  spotlight-dimmer-config       Configure settings

Usage:
  spotlight-dimmer                      # Start dimming inactive displays
  spotlight-dimmer-config status        # Show current configuration
  spotlight-dimmer-config enable        # Enable dimming
  spotlight-dimmer-config disable       # Disable dimming
  spotlight-dimmer-config color R G B A # Set overlay color (e.g., color 0 0 0 0.5)
  spotlight-dimmer-config reset         # Reset to defaults

For more information, visit:
https://github.com/thomazmoura/spotlight-dimmer
`);
