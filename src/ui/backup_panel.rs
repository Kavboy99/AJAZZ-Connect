use super::{
    backup_actions,
    icons::Icon,
    strings as s,
    theme::{Role, color, label, space},
    widgets::Button,
};
use crate::{app::backup_session::BackupSession, storage::backups};
use eframe::egui::Ui;

pub(crate) fn show(ui: &mut Ui, state: &mut BackupSession) {
    state.drain();
    label(
        ui,
        s::BACKUP_PANEL_LEAD,
        Role::Body,
        color::TEXT_SECONDARY.get(),
    );
    ui.add_space(space::M);
    if ui
        .add(
            Button::new(s::BACKUP_REFRESH)
                .icon(Icon::Search)
                .enabled(!state.session().busy())
                .busy(state.session().discovering()),
        )
        .clicked()
    {
        state.refresh();
    }
    let mut selected = state.selected().unwrap_or(0);
    let name = state
        .session()
        .entries()
        .get(selected)
        .map_or(s::BACKUP_SELECT, |e| e.descriptor().name.as_str());
    ui.add_space(space::M);
    ui.add_enabled_ui(!state.session().busy(), |ui| {
        eframe::egui::ComboBox::from_id_salt("backup-device")
            .selected_text(name)
            .width(ui.available_width().min(420.0))
            .show_ui(ui, |ui| {
                for (index, entry) in state.session().entries().iter().enumerate() {
                    ui.selectable_value(&mut selected, index, &entry.descriptor().name);
                }
            });
    });
    state.select(selected);
    ui.add_space(space::M);
    if let Some(slot) = state.selected() {
        let (session, events) = state.parts();
        backup_actions::actions(ui, session, events, slot);
    } else {
        ui.horizontal_wrapped(|ui| {
            ui.add(Button::new(s::BACKUP_SAVE).icon(Icon::Save).enabled(false));
            ui.add(
                Button::new(s::BACKUP_LOAD)
                    .icon(Icon::FileImport)
                    .enabled(false),
            );
        });
        label(
            ui,
            if state.session().discovering() {
                s::DEVICES_FINDING
            } else {
                s::BACKUP_EMPTY
            },
            Role::Body,
            color::TEXT_SECONDARY.get(),
        );
    }
    for note in state.session().notes().iter().take(3) {
        label(
            ui,
            s::error(&note.error),
            Role::Caption,
            color::STATUS_CAUTION.get(),
        );
    }
    ui.add_space(space::M);
    if ui
        .add(Button::new(s::SETTINGS_OPEN_FOLDER).icon(Icon::Folder))
        .clicked()
        && let Some(dir) = backups::export_directory().or_else(backups::directory)
        && std::fs::create_dir_all(&dir).is_ok()
    {
        let _ = std::process::Command::new("explorer").arg(dir).spawn();
    }
}
