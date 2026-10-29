use crate::app::{Events, Session};
use crate::ui::theme::{Role, color, label, space};
use crate::ui::{icons::Icon, strings as s, widgets::Button};
use crate::{
    app::identity,
    core::{BackupCoverage, RestoreScope},
    storage::{archive, backups},
};
use eframe::egui::Ui;

pub(crate) fn actions(ui: &mut Ui, session: &mut Session, events: &Events, slot: usize) {
    let Some(entry) = session.entries().get(slot) else {
        return;
    };
    let Some(info) = entry.backup_info() else {
        ui.horizontal_wrapped(|ui| {
            ui.add(Button::new(s::BACKUP_SAVE).icon(Icon::Save).enabled(false));
            ui.add(
                Button::new(s::BACKUP_LOAD)
                    .icon(Icon::FileImport)
                    .enabled(false),
            );
        });
        {
            label(
                ui,
                s::BACKUP_UNSUPPORTED,
                Role::Caption,
                color::TEXT_SECONDARY.get(),
            );
        }
        return;
    };
    let enabled = !entry.busy() && !entry.is_dormant();
    let name = entry.descriptor().name.clone();
    let mut export = false;
    let mut restore = false;
    ui.horizontal_wrapped(|ui| {
        export = ui
            .add(
                Button::new(s::BACKUP_SAVE)
                    .icon(Icon::Save)
                    .enabled(enabled),
            )
            .clicked();
        restore = ui
            .add(
                Button::new(s::BACKUP_LOAD)
                    .icon(Icon::FileImport)
                    .enabled(enabled && info.restore.is_some()),
            )
            .clicked();
        label(
            ui,
            coverage(info.coverage),
            Role::Caption,
            color::TEXT_SECONDARY.get(),
        );
    });
    if let Some(status) = entry.status() {
        let text = match status {
            crate::app::session::Status::Running => s::LIGHTING_RUNNING.into(),
            crate::app::session::Status::Done(outcome) => s::outcome(*outcome),
            crate::app::session::Status::Failed(error) => s::error(error).into(),
        };
        label(ui, &text, Role::Caption, color::TEXT_SECONDARY.get());
    }
    ui.add_space(space::S);
    if export {
        let mut dialog = rfd::FileDialog::new()
            .add_filter(s::BACKUP_ARCHIVE, &["lsbackup"])
            .set_file_name(format!(
                "backup-{}.lsbackup",
                chrono::Utc::now().format("%Y%m%d-%H%M%S")
            ));
        if let Some(dir) = backups::export_directory() {
            dialog = dialog.set_directory(dir);
        }
        if let Some(path) = dialog.save_file() {
            session.export_backup(slot, path, events);
        }
    }
    if restore {
        let mut dialog =
            rfd::FileDialog::new().add_filter(s::BACKUP_ARCHIVE, &["lsbackup", "json"]);
        if let Some(dir) = backups::export_directory() {
            dialog = dialog.set_directory(dir);
        }
        let Some(path) = dialog.pick_file() else {
            return;
        };
        let result = archive::load(&path).and_then(|snapshot| {
            session.check_backup(slot, &snapshot)?;
            Ok(snapshot)
        });
        let snapshot = match result {
            Ok(snapshot) => snapshot,
            Err(_) => {
                rfd::MessageDialog::new()
                    .set_title(s::BACKUP_LOAD)
                    .set_description(s::BACKUP_REJECTED)
                    .set_level(rfd::MessageLevel::Error)
                    .show();
                return;
            }
        };
        let scope = match info.restore {
            Some(RestoreScope::SystemConfiguration) => s::BACKUP_RESTORE_SYSTEM,
            Some(RestoreScope::Lighting) => s::BACKUP_RESTORE_LIGHTING,
            None => return,
        };
        let confirmed = rfd::MessageDialog::new()
            .set_title(identity::PRODUCT_NAME)
            .set_description(format!("{name}\n\n{scope}\n\n{}", s::BACKUP_CONFIRM))
            .set_level(rfd::MessageLevel::Warning)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        if confirmed == rfd::MessageDialogResult::Yes {
            session.recover_backup(slot, snapshot, events);
        }
    }
}

fn coverage(coverage: BackupCoverage) -> &'static str {
    match coverage {
        BackupCoverage::SystemAndPalette => s::BACKUP_SYSTEM_COVERAGE,
        BackupCoverage::Lighting => s::BACKUP_LIGHTING_COVERAGE,
    }
}
