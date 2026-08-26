//! SpotlightDimmer core: platform-agnostic overlay geometry, configuration
//! parsing and application state.
//!
//! This crate mirrors `SpotlightDimmer.Core` (C#) and the pure modules of the
//! GNOME Shell extension (`calculator.js`, the parsing half of
//! `configBridge.js`, and the geometry join of `appIntegrations.js`). It has
//! no GTK/Wayland/D-Bus dependencies so it can be unit-tested anywhere.

pub mod calculator;
pub mod config;
pub mod pane;
pub mod primitives;
pub mod state;
pub mod title;
