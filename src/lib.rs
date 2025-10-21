// Library exports for Spotlight Dimmer
// This allows us to test the code without running the binaries

pub mod autostart;
pub mod config;
pub mod platform;
pub mod tmux_watcher;
pub mod windows_terminal;

#[cfg(windows)]
pub mod overlay;

#[cfg(windows)]
pub mod tmux_overlay;

#[cfg(windows)]
pub mod tray;

#[cfg(windows)]
pub mod message_window;
