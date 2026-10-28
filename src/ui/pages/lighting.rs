//! The lighting section of a device page.

use super::Model;
use super::layout::{section_title, settled_width};
use crate::app::session::Status;
use crate::core::{LightingTraits, Maturity, Rgb, Rollback};
use crate::storage::backups;
use crate::ui::icons::Icon;
use crate::ui::motion::{self, tokens};
use crate::ui::strings as s;
use crate::ui::theme::{Role, breakpoint, color, label, radius, size, space};
use crate::ui::widgets::color_picker::{Hsv, color_picker};
use crate::ui::widgets::{Button, Tone, badge, card, spinner, text_field};
use eframe::egui::{Color32, CornerRadius, Id, Sense, Ui, Vec2};

const PREVIEW: f32 = 64.0;
const HEX_WIDTH: f32 = 120.0;
const PICKER_WIDTH: f32 = 320.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Apply,
    Restore,
    FromBackup,
}

/// Per-device editing state: the chosen color survives leaving the page.
pub struct State {
    hsv: Hsv,
    hex: String,
    last: Option<Action>,
}

impl Default for State {
    fn default() -> Self {
        let start = Rgb::new(0x4d, 0xb2, 0xe8);
        Self {
            hsv: Hsv::from_rgb(start, 0.0),
            hex: start.hex(),
            last: None,
        }
    }
}

pub fn section(
    ui: &mut Ui,
    model: &mut Model<'_>,
    slot: usize,
    traits: LightingTraits,
    state: &mut State,
) {
    let Some(entry) = model.session.entries().get(slot) else {
        return;
    };
    let running = entry.status() == Some(&Status::Running);
    let status = entry.status().cloned();
    let experimental = entry.descriptor().driver.maturity == Maturity::Experimental;
    let mut action = None;

    card(ui, |ui| {
        section_title(ui, Icon::Lighting, s::LIGHTING);
        let narrow = settled_width(ui) < breakpoint::SECTION_COLUMNS;
        let picker = |ui: &mut Ui, state: &mut State| {
            ui.vertical(|ui| {
                ui.set_width(PICKER_WIDTH.min(ui.available_width()));
                if color_picker(ui, Id::new(("picker", slot)), &mut state.hsv) {
                    state.hex = state.hsv.to_rgb().hex();
                }
            });
        };
        let mut controls = |ui: &mut Ui, state: &mut State| {
            ui.vertical(|ui| {
                preview(ui, slot, state.hsv.to_rgb());
                ui.add_space(space::M);
                label(
                    ui,
                    s::LIGHTING_COLOR,
                    Role::Caption,
                    color::TEXT_SECONDARY.get(),
                );
                let field = text_field(ui, Id::new(("hex", slot)), &mut state.hex, HEX_WIDTH, true);
                if field.changed()
                    && let Some(rgb) = Rgb::parse_hex(&state.hex)
                {
                    state.hsv = Hsv::from_rgb(rgb, state.hsv.h);
                }
                if field.lost_focus() {
                    state.hex = state.hsv.to_rgb().hex();
                }
                ui.add_space(space::L);
                let apply = Button::new(s::LIGHTING_APPLY)
                    .primary()
                    .icon(Icon::Brush)
                    .enabled(traits.writable && !running)
                    .busy(running && state.last == Some(Action::Apply));
                if ui.add(apply).clicked() {
                    action = Some(Action::Apply);
                }
                if traits.rollback != Rollback::Unavailable {
                    let restore = Button::new(s::LIGHTING_RESTORE)
                        .icon(Icon::Undo)
                        .enabled(!running)
                        .busy(running && state.last == Some(Action::Restore));
                    if ui.add(restore).clicked() {
                        action = Some(Action::Restore);
                    }
                }
                if traits.rollback == Rollback::SessionAndFile {
                    let file = Button::new(s::LIGHTING_FROM_BACKUP)
                        .icon(Icon::FileImport)
                        .enabled(!running)
                        .busy(running && state.last == Some(Action::FromBackup));
                    if ui.add(file).clicked() {
                        action = Some(Action::FromBackup);
                    }
                }
            });
        };
        if narrow {
            picker(ui, state);
            ui.add_space(space::L);
            controls(ui, state);
        } else {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = space::XL;
                picker(ui, state);
                controls(ui, state);
            });
        }

        ui.add_space(space::L);
        if experimental {
            badge(ui, Tone::Caution, s::LIGHTING_EXPERIMENTAL);
        }
        if !traits.writable {
            badge(ui, Tone::Info, s::LIGHTING_NOT_WRITABLE);
        }
        if traits.persistent {
            badge(ui, Tone::Caution, s::LIGHTING_PERSISTENT);
        } else {
            badge(ui, Tone::Info, s::LIGHTING_SESSION_ONLY);
        }
        if traits.rollback == Rollback::SessionAndFile {
            badge(ui, Tone::Info, s::LIGHTING_BACKUP_FIRST);
        }
        if traits.rollback == Rollback::Unavailable {
            badge(ui, Tone::Caution, s::LIGHTING_NO_ROLLBACK);
        }
        match &status {
            Some(Status::Running) => {
                ui.horizontal(|ui| {
                    spinner::show(ui, color::TEXT_SECONDARY.get());
                    label(
                        ui,
                        s::LIGHTING_RUNNING,
                        Role::Caption,
                        color::TEXT_SECONDARY.get(),
                    );
                });
            }
            Some(Status::Done(outcome)) => badge(ui, Tone::Success, &s::outcome(*outcome)),
            Some(Status::Failed(error)) => badge(ui, Tone::Critical, s::error(error)),
            None => {}
        }
    });

    let Some(action) = action else {
        return;
    };
    state.last = Some(action);
    match action {
        Action::Apply => model
            .session
            .apply_solid(slot, state.hsv.to_rgb(), model.events),
        Action::Restore => model.session.restore(slot, model.events),
        Action::FromBackup => {
            let mut dialog =
                rfd::FileDialog::new().add_filter(s::LIGHTING_BACKUP_FILTER, &["json"]);
            if let Some(dir) = backups::directory().filter(|d| d.exists()) {
                dialog = dialog.set_directory(dir);
            }
            if let Some(path) = dialog.pick_file() {
                model.session.restore_file(slot, path, model.events);
            }
        }
    }
}

/// The chosen color, flowing to each new value instead of jumping.
fn preview(ui: &mut Ui, slot: usize, rgb: Rgb) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(PREVIEW), Sense::hover());
    let shown = motion::animate_color(
        ui.ctx(),
        Id::new(("preview", slot)),
        Color32::from_rgb(rgb.r, rgb.g, rgb.b),
        tokens::FAST,
    );
    ui.painter()
        .rect_filled(rect, CornerRadius::same(radius::OVERLAY), shown);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(radius::OVERLAY),
        eframe::egui::Stroke::new(size::STROKE, color::STROKE_DEFAULT.get()),
        eframe::egui::StrokeKind::Inside,
    );
}
