//! Only a current connection blocker or failure needs space below the model.
use super::{DevicePage, Mode, Model, Route, State};
use crate::app::session::Status;
use crate::ui::strings as s;
use crate::ui::theme::{Role, size, space};
use crate::ui::widgets::{Tone, badge};
use eframe::egui::{Rect, Ui, UiBuilder, pos2};

fn message(
    model: &Model<'_>,
    page: &DevicePage<'_>,
    state: &State,
) -> Option<(Tone, &'static str)> {
    if let Route::Device(slot) = page.route
        && let Some(Status::Failed(error)) =
            model.session.entries().get(slot).and_then(|e| e.status())
    {
        return Some((Tone::Critical, s::error(error)));
    }
    if state.preferences_failed {
        return Some((Tone::Critical, s::SETTINGS_SAVE_FAILED));
    }
    if state.preview_requires_usb {
        return Some((
            Tone::Info,
            if matches!(page.route, Route::Preview(_)) {
                s::LIGHTING_OFFLINE_CONTROLS
            } else if page.dormant {
                s::LIGHTING_ENABLE_DRIVER
            } else {
                s::LIGHTING_CONNECT_USB
            },
        ));
    }
    if page.traits.is_some_and(|t| !t.writable) {
        return Some((Tone::Info, s::LIGHTING_WRITE_BLOCKED));
    }
    None
}

pub(super) fn show(
    ui: &mut Ui,
    stage: Rect,
    destination: Rect,
    model: &Model<'_>,
    page: &DevicePage<'_>,
    state: &State,
) -> (Rect, Rect) {
    if state.mode != Mode::Lighting {
        return (stage, destination);
    }
    let Some((tone, text)) = message(model, page, state) else {
        return (stage, destination);
    };
    let destination_height = ui
        .painter()
        .layout(
            text.to_owned(),
            Role::Caption.font(),
            tone.color(),
            (destination.width() - space::S * 3.0 - size::ICON_SMALL).max(space::L),
        )
        .size()
        .y
        .max(size::ICON_SMALL)
        + space::M;
    let wrap = (stage.width() - space::S * 3.0 - size::ICON_SMALL).max(space::L);
    let height = ui
        .painter()
        .layout(text.to_owned(), Role::Caption.font(), tone.color(), wrap)
        .size()
        .y
        .max(size::ICON_SMALL)
        + space::M;
    let top = stage.bottom() - height;
    let rect = Rect::from_min_max(pos2(stage.left(), top), stage.max);
    let mut strip = ui.new_child(UiBuilder::new().id_salt("studio-notices").max_rect(rect));
    strip.set_clip_rect(rect.intersect(ui.clip_rect()));
    badge(&mut strip, tone, text);
    (
        Rect::from_min_max(stage.min, pos2(stage.right(), top - space::S)),
        Rect::from_min_max(
            destination.min,
            pos2(
                destination.right(),
                destination.bottom() - destination_height - space::S,
            ),
        ),
    )
}
