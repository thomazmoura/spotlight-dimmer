//! Basic geometry and color types shared across the daemon.

use serde::{Deserialize, Serialize};

/// RGB color. Serialized as `{"r":0,"g":0,"b":0}` in overlay payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
}

/// Rectangle in logical global compositor coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    /// Clamp this rectangle to fit within `bounds`.
    /// Port of `_clampToMonitor` (calculator.js) / `AppState.ClampToDisplay` (C#).
    /// Degenerate results have width/height 0, never negative.
    pub fn clamp_to(&self, bounds: &Rect) -> Rect {
        let left = self.x.max(bounds.x);
        let top = self.y.max(bounds.y);
        let right = self.right().min(bounds.right());
        let bottom = self.bottom().min(bounds.bottom());

        Rect {
            x: left,
            y: top,
            width: (right - left).max(0),
            height: (bottom - top).max(0),
        }
    }

    /// Area of the intersection with `other`, in square logical pixels.
    pub fn overlap_area(&self, other: &Rect) -> i64 {
        let c = self.clamp_to(other);
        c.width as i64 * c.height as i64
    }
}
