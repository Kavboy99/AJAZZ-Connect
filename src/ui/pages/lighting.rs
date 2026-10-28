//! Explicit hardware actions, separate from the visual editor and its effects.
use super::Model;
use crate::app::session::Status;
use crate::core::{LightingTraits, Maturity, Rgb, Rollback};
use crate::storage::backups;
use crate::ui::icons::Icon;
use crate::ui::strings as s;
use crate::ui::theme::{Role, color, label, space};
use crate::ui::widgets::{Button, Tone, badge};
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
    rgb: Rgb,
    settings: Option<crate::core::LightingSettings>,
    state: &mut State,
) {
    let Some(entry) = model.session.entries().get(slot) else {
        return;
    };
    let status = entry.status().cloned();
    let running = status == Some(Status::Running);
    let experimental = entry.descriptor().driver.maturity == Maturity::Experimental;
    label(
        ui,
        s::STUDIO_DEVICE_WRITE,
        Role::BodyStrong,
        color::TEXT_PRIMARY.get(),
    );
    ui.add_space(space::S);
    if experimental {
        badge(ui, Tone::Caution, s::LIGHTING_EXPERIMENTAL);
    }
    if !traits.writable {
        badge(ui, Tone::Info, s::LIGHTING_NOT_WRITABLE);
    }
    badge(
        ui,
        if traits.persistent {
            Tone::Caution
        } else {
            Tone::Info
        },
        if traits.persistent {
            s::LIGHTING_PERSISTENT
        } else {
            s::LIGHTING_SESSION_ONLY
        },
    );
    match traits.rollback {
        Rollback::SessionAndFile => badge(ui, Tone::Info, s::LIGHTING_BACKUP_FIRST),
        Rollback::Unavailable => badge(ui, Tone::Caution, s::LIGHTING_NO_ROLLBACK),
        Rollback::Session => {}
    }
    ui.add_space(space::S);
    let mut action = None;
    if ui
        .add(
            Button::new(if settings.is_some() {
                s::LIGHTING_APPLY_PROGRAM
            } else {
                s::LIGHTING_APPLY
            })
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
    match &status {
        Some(Status::Running) => {
            label(
                ui,
                s::LIGHTING_RUNNING,
                Role::Caption,
                color::TEXT_SECONDARY.get(),
            );
        }
        Some(Status::Done(outcome)) => badge(ui, Tone::Success, &s::outcome(*outcome)),
        Some(Status::Failed(error)) => badge(ui, Tone::Critical, s::error(error)),
        None => {}
    }
    let Some(action) = action else { return };
    state.last = Some(action);
    match action {
        Action::Apply => {
            if let Some(settings) = settings {
                model.session.apply_lighting(slot, settings, model.events);
            } else {
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
