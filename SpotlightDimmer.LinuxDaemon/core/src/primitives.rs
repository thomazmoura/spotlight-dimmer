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

    /// This rectangle with `hole` cut out of it, as at most four
    /// non-overlapping pieces whose union plus the hole is the original.
    ///
    /// The split is band-above / band-below / left / right, so the two
    /// horizontal bands span the full width and the side pieces only span
    /// the hole's height. A hole that misses this rectangle entirely gives
    /// back the rectangle unchanged; a hole that covers it gives back
    /// nothing.
    ///
    /// Used to exempt floating surfaces from the dimming: a transparent
    /// overlay painted *on top* cannot undo the dim beneath it, and a second
    /// dim overlay on top would composite into a darker patch, so the region
    /// underneath has to actually be removed.
    pub fn subtract(&self, hole: &Rect) -> Vec<Rect> {
        if self.width <= 0 || self.height <= 0 {
            return Vec::new();
        }

        let hole = hole.clamp_to(self);
        if hole.width <= 0 || hole.height <= 0 {
            return vec![*self];
        }

        let mut pieces = Vec::with_capacity(4);

        if hole.y > self.y {
            pieces.push(Rect::new(self.x, self.y, self.width, hole.y - self.y));
        }

        if hole.bottom() < self.bottom() {
            pieces.push(Rect::new(
                self.x,
                hole.bottom(),
                self.width,
                self.bottom() - hole.bottom(),
            ));
        }

        if hole.x > self.x {
            pieces.push(Rect::new(self.x, hole.y, hole.x - self.x, hole.height));
        }

        if hole.right() < self.right() {
            pieces.push(Rect::new(
                hole.right(),
                hole.y,
                self.right() - hole.right(),
                hole.height,
            ));
        }

        pieces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: Rect = Rect::new(0, 0, 100, 100);

    /// Total area of a piece list.
    fn area(pieces: &[Rect]) -> i64 {
        pieces
            .iter()
            .map(|p| p.width as i64 * p.height as i64)
            .sum()
    }

    fn assert_partition(base: &Rect, hole: &Rect) {
        let pieces = base.subtract(hole);
        let covered = hole.clamp_to(base);

        // The pieces plus the hole tile the base exactly...
        assert_eq!(
            area(&pieces) + covered.width as i64 * covered.height as i64,
            base.width as i64 * base.height as i64,
            "pieces {pieces:?} do not tile {base:?} around {hole:?}"
        );

        // ...and never overlap each other or the hole.
        for (i, a) in pieces.iter().enumerate() {
            assert_eq!(a.overlap_area(&covered), 0, "piece {i} overlaps the hole");
            for b in pieces.iter().skip(i + 1) {
                assert_eq!(a.overlap_area(b), 0, "pieces overlap: {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn a_hole_in_the_middle_splits_into_four_pieces() {
        let pieces = BASE.subtract(&Rect::new(40, 40, 20, 20));
        assert_eq!(pieces.len(), 4);
        assert_partition(&BASE, &Rect::new(40, 40, 20, 20));
    }

    #[test]
    fn a_hole_touching_edges_and_corners_drops_those_pieces() {
        // Full-width band across the middle: only above and below survive.
        assert_eq!(BASE.subtract(&Rect::new(0, 40, 100, 20)).len(), 2);
        // Corner: one band plus one side.
        assert_eq!(BASE.subtract(&Rect::new(0, 0, 20, 20)).len(), 2);
        // Full edge strip: a single piece.
        assert_eq!(BASE.subtract(&Rect::new(0, 0, 100, 20)).len(), 1);
    }

    #[test]
    fn a_hole_covering_everything_leaves_nothing() {
        assert!(BASE.subtract(&BASE).is_empty());
        assert!(BASE.subtract(&Rect::new(-50, -50, 400, 400)).is_empty());
    }

    #[test]
    fn a_hole_that_misses_returns_the_rectangle_unchanged() {
        assert_eq!(BASE.subtract(&Rect::new(200, 200, 50, 50)), vec![BASE]);
        // Touching edge-to-edge is not an overlap.
        assert_eq!(BASE.subtract(&Rect::new(100, 0, 50, 100)), vec![BASE]);
        // Degenerate holes are no-ops.
        assert_eq!(BASE.subtract(&Rect::new(10, 10, 0, 50)), vec![BASE]);
    }

    #[test]
    fn pieces_always_tile_the_original_for_every_hole_position() {
        let base = Rect::new(-30, 17, 140, 90);
        for x in (-60..140).step_by(17) {
            for y in (-10..130).step_by(13) {
                for size in [1, 11, 60, 200] {
                    assert_partition(&base, &Rect::new(x, y, size, size));
                }
            }
        }
    }
}
