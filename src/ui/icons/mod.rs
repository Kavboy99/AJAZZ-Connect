//! The only icon source: Hugeicons Stroke Rounded, referenced through `Icon`. An icon takes the
//! color of its text or status token.

pub mod app_icon;
mod glyphs;
mod path;
mod raster;

use crate::core::DeviceKind;
use eframe::egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, vec2};
use glyphs::Element;
use path::Polyline;
use std::sync::OnceLock;

const GRID: f32 = 24.0;
const STROKE: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    /// The application itself (also the window and EXE icon, `app_icon.rs`).
    Touch,
    App,
    Keyboard,
    Mouse,
    Headset,
    Gamepad,
    Lighting,
    Component,
    Plug,
    Battery,
    Devices,
    Diagnostics,
    Settings,
    Success,
    Caution,
    Critical,
    Info,
    Search,
    Undo,
    FileImport,
    Save,
    Folder,
    ChevronRight,
    ChevronLeft,
    Menu,
    Brush,
    Privacy,
    /// Switch to the light theme.
    Sun,
    /// Switch to the dark theme.
    Moon,
    Close,
}

impl Icon {
    const ALL: [Icon; 30] = [
        Icon::Touch,
        Icon::App,
        Icon::Keyboard,
        Icon::Mouse,
        Icon::Headset,
        Icon::Gamepad,
        Icon::Lighting,
        Icon::Component,
        Icon::Plug,
        Icon::Battery,
        Icon::Devices,
        Icon::Diagnostics,
        Icon::Settings,
        Icon::Success,
        Icon::Caution,
        Icon::Critical,
        Icon::Info,
        Icon::Search,
        Icon::Undo,
        Icon::FileImport,
        Icon::Save,
        Icon::Folder,
        Icon::ChevronRight,
        Icon::ChevronLeft,
        Icon::Menu,
        Icon::Brush,
        Icon::Privacy,
        Icon::Sun,
        Icon::Moon,
        Icon::Close,
    ];

    fn elements(self) -> &'static [Element] {
        match self {
            Icon::Touch => glyphs::TOUCH_01,
            Icon::App => glyphs::GAME_CONTROLLER_03,
            Icon::Keyboard => glyphs::KEYBOARD,
            Icon::Mouse => glyphs::MOUSE,
            Icon::Headset => glyphs::HEADSET,
            Icon::Gamepad => glyphs::GAMEPAD,
            Icon::Lighting => glyphs::BULB,
            Icon::Component => glyphs::CPU,
            Icon::Plug => glyphs::PLUG,
            Icon::Battery => glyphs::BATTERY_EMPTY,
            Icon::Devices => glyphs::USB,
            Icon::Diagnostics => glyphs::ACTIVITY,
            Icon::Settings => glyphs::SETTINGS,
            Icon::Success => glyphs::CHECKMARK_CIRCLE,
            Icon::Caution => glyphs::ALERT_TRIANGLE,
            Icon::Critical => glyphs::ALERT_CIRCLE,
            Icon::Info => glyphs::INFORMATION_CIRCLE,
            Icon::Search => glyphs::SEARCH,
            Icon::Undo => glyphs::UNDO,
            Icon::FileImport => glyphs::FILE_IMPORT,
            Icon::Save => glyphs::FLOPPY_DISK,
            Icon::Folder => glyphs::FOLDER,
            Icon::ChevronRight => glyphs::ARROW_RIGHT,
            Icon::ChevronLeft => glyphs::ARROW_LEFT,
            Icon::Menu => glyphs::MENU,
            Icon::Brush => glyphs::PAINT_BRUSH,
            Icon::Privacy => glyphs::SHIELD,
            Icon::Sun => glyphs::SUN_03,
            Icon::Moon => glyphs::MOON_02,
            Icon::Close => glyphs::CANCEL_01,
        }
    }

    pub fn for_kind(kind: DeviceKind) -> Self {
        match kind {
            DeviceKind::Keyboard => Icon::Keyboard,
            DeviceKind::Mouse => Icon::Mouse,
            DeviceKind::Headset => Icon::Headset,
            DeviceKind::Gamepad => Icon::Gamepad,
            DeviceKind::Lighting => Icon::Lighting,
            DeviceKind::Component => Icon::Component,
            DeviceKind::Other => Icon::Plug,
        }
    }

    fn outline(self) -> &'static [Polyline] {
        static CACHE: OnceLock<Vec<Vec<Polyline>>> = OnceLock::new();
        let cache = CACHE.get_or_init(|| {
            Icon::ALL
                .iter()
                .map(|icon| path::outline(icon.elements()))
                .collect()
        });
        let index = Icon::ALL.iter().position(|i| *i == self).unwrap_or(0);
        &cache[index]
    }

    /// Draws the icon centered in `rect`, rotated by `angle` radians around its center.
    pub fn paint_rotated(self, painter: &Painter, rect: Rect, color: Color32, angle: f32) {
        let size = rect.width().min(rect.height());
        let scale = size / GRID;
        let center = rect.center();
        let (sin, cos) = angle.sin_cos();
        let map = |p: path::Point| {
            let v = (vec2(p.x, p.y) - Vec2::splat(GRID / 2.0)) * scale;
            center + vec2(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
        };
        let stroke = Stroke::new(STROKE * scale, color);
        for line in self.outline() {
            let points: Vec<Pos2> = line.points.iter().map(|p| map(*p)).collect();
            if line.closed {
                painter.add(Shape::closed_line(points, stroke));
                continue;
            }
            // Round caps, as in the source set; they also draw the zero-length dots.
            for end in [points.first(), points.last()].into_iter().flatten() {
                painter.circle_filled(*end, stroke.width / 2.0, color);
            }
            painter.add(Shape::line(points, stroke));
        }
    }

    pub fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        self.paint_rotated(painter, rect, color, 0.0);
    }

    /// Allocates a square of `size` and draws the icon in it.
    pub fn show(self, ui: &mut Ui, size: f32, color: Color32) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        self.paint(ui.painter(), rect, color);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_icon_colors_follow_the_tokens() {
        use crate::ui::theme::color;
        assert_eq!(app_icon::TILE, color::SURFACE_CARD.dark().to_array());
        assert_eq!(app_icon::INK, color::TEXT_PRIMARY.dark().to_array());
        let pixels = app_icon::rgba(32);
        assert_eq!(pixels.len(), 32 * 32 * 4);
        // Transparent corner, opaque tile center.
        assert_eq!(pixels[3], 0);
        assert_eq!(pixels[(16 * 32 + 2) * 4 + 3], 255);
    }

    #[test]
    fn every_glyph_parses_into_the_grid() {
        for icon in Icon::ALL {
            for element in icon.elements() {
                if let Element::Path(d) = element {
                    let lines = path::parse(d).unwrap_or_else(|| panic!("{icon:?}"));
                    assert!(!lines.is_empty(), "{icon:?}");
                    for p in lines.iter().flat_map(|l| &l.points) {
                        assert!(
                            (0.0..=GRID).contains(&p.x) && (0.0..=GRID).contains(&p.y),
                            "{icon:?}"
                        );
                    }
                }
            }
        }
    }
}
