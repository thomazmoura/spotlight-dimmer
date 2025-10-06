// Library exports for Spotlight Dimmer
// This allows us to test the code without running the binaries

pub mod config;
pub mod platform;

#[cfg(windows)]
pub mod overlay;

#[cfg(windows)]
pub mod tray;
