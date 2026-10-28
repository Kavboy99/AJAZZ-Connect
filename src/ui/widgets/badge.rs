//! Status badge: icon + text + status color on `surface.sunken`. Color alone never carries the
//! meaning. Appears with a fade and an 8 px shift.

use crate::ui::icons::Icon;
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{Role, color, motion_tokens, radius, size, space};
use eframe::egui::{Color32, CornerRadius, Id, Rect, Sense, Ui, Vec2, pos2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Success,
    Caution,
    Critical,
    Info,
}

impl Tone {
    pub fn color(self) -> Color32 {
        match self {
            Tone::Success => color::STATUS_SUCCESS.get(),
            Tone::Caution => color::STATUS_CAUTION.get(),
            Tone::Critical => color::STATUS_CRITICAL.get(),
            Tone::Info => color::STATUS_INFO.get(),
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Tone::Success => Icon::Success,
            Tone::Caution => Icon::Caution,
            Tone::Critical => Icon::Critical,
            Tone::Info => Icon::Info,
        }
    }
}

const PADDING: Vec2 = Vec2::new(space::S, space::XS + 2.0);
/// Below this the text would wrap after every word.
const MIN_TEXT_WIDTH: f32 = 120.0;

pub fn badge(ui: &mut Ui, tone: Tone, text: &str) {
    let id = Id::new(("badge", tone as u8, text));
    let ctx = ui.ctx().clone();
    let shown = motion::animate_from(&ctx, id, 0.0, 1.0, tokens::STANDARD, Channel::Fade);
    let shift = motion::animate_from(
        &ctx,
        id.with("shift"),
        motion_tokens::APPEAR_SHIFT,
        0.0,
        tokens::STANDARD,
        Channel::Offset,
    );
    let ink = tone.color();
    let wrap =
        (ui.available_width() - PADDING.x * 2.0 - size::ICON_SMALL - space::S).max(MIN_TEXT_WIDTH);
    let galley = ui
        .painter()
        .layout(text.to_owned(), Role::Caption.font(), ink, wrap);
    let inner = vec2(
        size::ICON_SMALL + space::S + galley.size().x,
        galley.size().y.max(size::ICON_SMALL),
    );
    let (rect, _) = ui.allocate_exact_size(inner + PADDING * 2.0, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let rect = rect.translate(vec2(0.0, shift));
    let mut painter = ui.painter().clone();
    painter.multiply_opacity(shown);
    painter.rect_filled(
        rect,
        CornerRadius::same(radius::CONTROL),
        color::SURFACE_SUNKEN.get(),
    );
    let top = rect.top() + PADDING.y;
    let icon = Rect::from_min_size(
        pos2(
            rect.left() + PADDING.x,
            top + (inner.y - size::ICON_SMALL) / 2.0,
        ),
        Vec2::splat(size::ICON_SMALL),
    );
    tone.icon().paint(&painter, icon, ink);
    painter.galley(
        pos2(
            icon.right() + space::S,
            top + (inner.y - galley.size().y) / 2.0,
        ),
        galley,
        ink,
    );
}
