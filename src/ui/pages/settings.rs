use super::layout::{columns, gap, header, hint, section_title, spans};
use super::{Model, Navigate};
use crate::app::identity;
use crate::core::Maturity;
use crate::storage::backups;
use crate::ui::icons::Icon;
use crate::ui::strings as s;
use crate::ui::theme::{Role, color, label, space};
use crate::ui::widgets::{Button, Tone, badge, card, switch};
use eframe::egui::Ui;

#[derive(Default)]
pub struct State {
    save_failed: bool,
}

pub fn show(ui: &mut Ui, model: &mut Model<'_>, state: &mut State) -> Navigate {
    header(ui, s::SETTINGS_TITLE, None, |_| {});

    card(ui, |ui| {
        section_title(ui, Icon::Settings, s::SETTINGS_DRIVERS);
        let mut on = model.settings.experimental_drivers;
        if switch(ui, &mut on, s::SETTINGS_EXPERIMENTAL).changed() {
            model.settings.experimental_drivers = on;
            state.save_failed = model.settings.save().is_err();
            // The device list follows the setting at once.
            model.session.discover(model.events, on);
        }
        hint(ui, s::SETTINGS_EXPERIMENTAL_HINT);
        if state.save_failed {
            badge(ui, Tone::Critical, s::SETTINGS_SAVE_FAILED);
        }
        ui.add_space(space::M);
        for driver in model.drivers {
            let tone = match driver.maturity {
                Maturity::Stable => color::TEXT_SECONDARY.get(),
                Maturity::Experimental => color::STATUS_CAUTION.get(),
            };
            spans(
                ui,
                &[
                    (driver.name, Role::BodyStrong, color::TEXT_PRIMARY.get()),
                    (s::maturity(driver.maturity), Role::Caption, tone),
                ],
            );
            if let Some(requirement) = driver.requirement {
                label(
                    ui,
                    s::requirement(requirement),
                    Role::Caption,
                    color::TEXT_SECONDARY.get(),
                );
            }
            ui.add_space(space::XS);
        }
    });
    gap(ui);
    columns(ui, 0.5, model, |ui, _| data(ui), about);
    None
}

fn data(ui: &mut Ui) {
    card(ui, |ui| {
        section_title(ui, Icon::Folder, s::SETTINGS_DATA);
        label(
            ui,
            s::SETTINGS_BACKUPS,
            Role::Body,
            color::TEXT_SECONDARY.get(),
        );
        match backups::directory() {
            Some(dir) => {
                label(
                    ui,
                    dir.display().to_string(),
                    Role::Mono,
                    color::TEXT_PRIMARY.get(),
                );
                ui.add_space(space::M);
                if ui
                    .add(Button::new(s::SETTINGS_OPEN_FOLDER).icon(Icon::Folder))
                    .clicked()
                    && std::fs::create_dir_all(&dir).is_ok()
                {
                    let _ = std::process::Command::new("explorer").arg(&dir).spawn();
                }
            }
            None => badge(ui, Tone::Caution, s::SETTINGS_NO_DATA_DIR),
        }
    });
}

fn about(ui: &mut Ui, model: &mut Model<'_>) {
    card(ui, |ui| {
        section_title(ui, Icon::Info, s::SETTINGS_ABOUT);
        label(
            ui,
            identity::PRODUCT_NAME,
            Role::BodyStrong,
            color::TEXT_PRIMARY.get(),
        );
        label(
            ui,
            s::version(identity::version()),
            Role::Caption,
            color::TEXT_SECONDARY.get(),
        );
        ui.add_space(space::S);
        hint(ui, s::SETTINGS_ABOUT_TEXT);
        label(
            ui,
            s::SETTINGS_ICONS,
            Role::Caption,
            color::TEXT_TERTIARY.get(),
        );
        if !model.animations {
            ui.add_space(space::S);
            badge(ui, Tone::Info, s::SETTINGS_ANIMATIONS_OFF);
        }
    });
}
