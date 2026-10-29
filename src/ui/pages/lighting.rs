//! Explicit hardware actions, separate from the visual editor and its effects.
use super::Model;
use crate::app::session::Status;
use crate::core::{LightingTraits, Rgb, Rollback};
use crate::storage::backups;
use crate::ui::icons::Icon;
use crate::ui::strings as s;
use crate::ui::widgets::Button;
use eframe::egui::Ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Apply,
    Restore,
    FromBackup,
}

#[derive(Default)]
pub struct State {
    last: Option<Action>,
}

pub fn actions(
    ui: &mut Ui,
    model: &mut Model<'_>,
    slot: usize,
    traits: LightingTraits,
    solid: Option<Rgb>,
    state: &mut State,
) {
    let Some(entry) = model.session.entries().get(slot) else {
        return;
    };
    let status = entry.status().cloned();
    let running = status == Some(Status::Running);
    let configuration_backup = entry.backup_info().is_some();
    let mut action = None;
    if solid.is_some()
        && ui
            .add(
                Button::new(s::LIGHTING_APPLY)
                    .primary()
                    .icon(Icon::Brush)
                    .enabled(traits.writable && !running)
                    .busy(running && state.last == Some(Action::Apply)),
            )
            .clicked()
    {
        action = Some(Action::Apply);
    }
    if traits.rollback != Rollback::Unavailable
        && ui
            .add(
                Button::new(s::LIGHTING_RESTORE)
                    .icon(Icon::Undo)
                    .enabled(!running)
                    .busy(running && state.last == Some(Action::Restore)),
            )
            .clicked()
    {
        action = Some(Action::Restore);
    }
    if traits.rollback == Rollback::SessionAndFile
        && !configuration_backup
        && ui
            .add(
                Button::new(s::LIGHTING_FROM_BACKUP)
                    .icon(Icon::FileImport)
                    .enabled(!running)
                    .busy(running && state.last == Some(Action::FromBackup)),
            )
            .clicked()
    {
        action = Some(Action::FromBackup);
    }
    let Some(action) = action else { return };
    state.last = Some(action);
    match action {
        Action::Apply => {
            if let Some(rgb) = solid {
                model.session.apply_solid(slot, rgb, model.events);
            }
        }
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
