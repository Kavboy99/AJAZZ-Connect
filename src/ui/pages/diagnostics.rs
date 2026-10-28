use super::layout::{gap, header, hint, section_title};
use super::{Model, Navigate};
use crate::app::identity;
use crate::diagnostics::Interface;
use crate::ui::icons::Icon;
use crate::ui::strings as s;
use crate::ui::theme::{Role, color, label, space};
use crate::ui::widgets::{Button, Tone, badge, card, expander};
use eframe::egui::{Id, Ui};

pub fn show(ui: &mut Ui, model: &mut Model<'_>) -> Navigate {
    header(ui, s::DIAG_TITLE, Some(s::DIAG_LEAD), |_| {});
    connection(ui, model);
    gap(ui);
    report(ui, model);
    None
}

fn connection(ui: &mut Ui, model: &mut Model<'_>) {
    card(ui, |ui| {
        section_title(ui, Icon::Devices, s::DIAG_CONNECTION);
        hint(ui, s::DIAG_CONNECTION_HINT);
        ui.add_space(space::M);
        let probing = model.diagnostics.probing();
        let check = Button::new(s::DIAG_CHECK)
            .primary()
            .icon(Icon::Search)
            .busy(probing);
        if ui.add(check).clicked() {
            model.diagnostics.probe(model.events);
        }
        match model.diagnostics.report() {
            Some(Ok(report)) if report.interfaces.is_empty() => {
                ui.add_space(space::M);
                badge(ui, Tone::Info, s::DIAG_NOTHING);
            }
            Some(Ok(report)) => {
                ui.add_space(space::M);
                let title = format!(
                    "{} · {}",
                    s::DIAG_INTERFACES,
                    s::interfaces(report.interfaces.len())
                );
                expander(ui, Id::new("diag-interfaces"), &title, true, |ui| {
                    for interface in &report.interfaces {
                        show_interface(ui, interface);
                        ui.add_space(space::M);
                    }
                });
            }
            Some(Err(error)) => {
                ui.add_space(space::M);
                badge(ui, Tone::Critical, s::error(error));
            }
            None => {}
        }
    });
}

fn show_interface(ui: &mut Ui, i: &Interface) {
    let ids = format!(
        "{:04X}:{:04X} · {:04X}:{:04X} · {}",
        i.vid, i.pid, i.usage_page, i.usage, i.product
    );
    label(ui, ids, Role::Mono, color::TEXT_PRIMARY.get());
    let family = i.family();
    let name = family.map_or(i.protocol_family.as_str(), |f| f.name.as_str());
    if !name.is_empty() {
        label(ui, name, Role::BodyStrong, color::TEXT_PRIMARY.get());
    }
    if let Some(profile) = &i.profile {
        label(ui, profile, Role::Caption, color::TEXT_SECONDARY.get());
    }
    if let Some(family) = family {
        label(
            ui,
            &family.hint.ru,
            Role::Caption,
            color::TEXT_SECONDARY.get(),
        );
    }
    if let Some(identification) = &i.identification {
        badge(
            ui,
            Tone::Success,
            &s::identified(identification.firmware_version.as_deref()),
        );
    }
    if let Some(error) = &i.read_error {
        label(ui, error, Role::Caption, color::TEXT_TERTIARY.get());
    }
}

fn report(ui: &mut Ui, model: &mut Model<'_>) {
    card(ui, |ui| {
        section_title(ui, Icon::Privacy, s::DIAG_REPORT);
        hint(ui, s::DIAG_REPORT_HINT);
        ui.add_space(space::M);
        let ready = matches!(model.diagnostics.report(), Some(Ok(_)));
        if ui
            .add(Button::new(s::DIAG_SAVE).icon(Icon::Save).enabled(ready))
            .clicked()
            && let Some(path) = rfd::FileDialog::new()
                .set_file_name(identity::report_file_name())
                .add_filter(s::DIAG_REPORT_FILTER, &["json"])
                .save_file()
        {
            model.diagnostics.save_report(path);
        }
        match model.diagnostics.saved() {
            Some(Ok(path)) => {
                ui.add_space(space::S);
                badge(ui, Tone::Success, &s::saved_to(&path.display().to_string()));
            }
            Some(Err(detail)) => {
                ui.add_space(space::S);
                badge(ui, Tone::Critical, &s::save_failed(detail));
            }
            None => {}
        }
    });
}
