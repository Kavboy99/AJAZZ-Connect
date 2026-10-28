//! Compact identity on the trailing side of the device stage.
use super::device_view::DeviceView;
use super::layout::spans;
use crate::ui::theme::{Role, color, label, size, space};
use crate::ui::{icons::Icon, strings as s};
use eframe::egui::Ui;

pub(super) fn identity(ui: &mut Ui, d: &DeviceView<'_>, preview: bool, dormant: bool) {
    spans(
        ui,
        &[
            (s::kind(d.kind), Role::Caption, color::TEXT_SECONDARY.get()),
            (
                d.vendor.unwrap_or_default(),
                Role::Caption,
                color::TEXT_SECONDARY.get(),
            ),
        ],
    );
    ui.add_space(space::S);
    label(ui, d.name, Role::Title, color::TEXT_PRIMARY.get());
    ui.add_space(space::M);
    status(
        ui,
        Icon::Component,
        &format!(
            "{} · {}",
            s::LABEL_DRIVER,
            d.driver.map_or(s::DEVICE_NO_DRIVER, |driver| driver.name)
        ),
        color::TEXT_SECONDARY.get(),
    );
    if let Some(driver) = d.driver {
        label(
            ui,
            s::maturity(driver.maturity),
            Role::Caption,
            color::TEXT_TERTIARY.get(),
        );
    }
    status(
        ui,
        if dormant {
            Icon::Caution
        } else if preview {
            Icon::Info
        } else {
            Icon::Success
        },
        if dormant {
            s::DEVICE_DORMANT
        } else if preview {
            if d.driver.is_none() {
                s::DEVICE_PLACEHOLDER
            } else {
                s::DEVICE_CATALOG
            }
        } else {
            s::DEVICE_CONNECTED
        },
        if dormant {
            color::STATUS_CAUTION.get()
        } else {
            color::TEXT_SECONDARY.get()
        },
    );
    ui.add_space(space::M);
    crate::ui::widgets::battery_status(ui, d.details);
}

fn status(ui: &mut Ui, icon: Icon, text: &str, ink: eframe::egui::Color32) {
    ui.horizontal_top(|ui| {
        icon.show(ui, size::ICON_SMALL, ink);
        label(ui, text, Role::Caption, ink);
    });
}
