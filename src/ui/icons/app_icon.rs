//! The application icon: Hugeicons "game-controller-03" in near-white on a dark grey rounded tile.
//! Drawn for the window (`ui/mod.rs`) and embedded into the executable (`build.rs`). Free of egui;
//! the colors repeat `surface.card` and `text.primary` from `ui/theme/tokens.rs` because the build
//! script cannot depend on egui, and a test keeps them equal.

use super::glyphs::GAME_CONTROLLER_03;
use super::path::{self, Point};
use super::raster::Canvas;

pub const TILE: [u8; 4] = [0x2B, 0x2B, 0x2B, 0xFF];
pub const INK: [u8; 4] = [0xF2, 0xF2, 0xF2, 0xFF];

/// Corner radius as a share of the tile, like Windows 11 app tiles.
const CORNER: f32 = 0.22;
/// The glyph's share of the tile.
const GLYPH: f32 = 0.70;
/// Keeps strokes legible at 16 px.
const MIN_STROKE: f32 = 1.1;

/// Square RGBA image of `size` pixels, straight alpha, top-down.
pub fn rgba(size: u32) -> Vec<u8> {
    let s = size as f32;
    let mut canvas = Canvas::new(size as usize);
    canvas.fill_rounded_square(s * CORNER, TILE);
    let glyph = s * GLYPH;
    let scale = glyph / 24.0;
    let origin = Point {
        x: (s - glyph) / 2.0,
        y: (s - glyph) / 2.0,
    };
    let width = (1.5 * scale).max(MIN_STROKE);
    canvas.stroke(
        &path::outline(GAME_CONTROLLER_03),
        origin,
        scale,
        width,
        INK,
    );
    canvas.into_rgba()
}
