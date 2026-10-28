use super::layout::{columns, field, hint, section_title};
use super::lighting;
use super::{Model, Navigate, Route};
use crate::core::DeviceDescriptor;
use crate::ui::icons::Icon;
use crate::ui::motion::Style;
use crate::ui::strings as s;
use crate::ui::theme::{Role, color, rich, space};
use crate::ui::widgets::{Button, card};
use eframe::egui::{Label, Ui};

/// A device page: its passport plus one section per capability it has.
pub fn show(
    ui: &mut Ui,
    model: &mut Model<'_>,
    slot: usize,
    state: &mut lighting::State,
) -> Navigate {
    let mut navigate = None;
    let back = (Route::Devices, Style::DrillOut);
    let Some(entry) = model.session.entries().get(slot) else {
        card(ui, |ui| {
            hint(ui, s::DEVICE_GONE);
            ui.add_space(space::M);
            if ui
                .add(Button::new(s::NAV_DEVICES).icon(Icon::ChevronLeft))
                .clicked()
            {
                navigate = Some(back);
            }
        });
        return navigate;
    };
    let descriptor = entry.descriptor().clone();
    let traits = entry.lighting();
    let dormant = entry.is_dormant();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::S;
        if ui
            .add(Button::icon_only(Icon::ChevronLeft, s::BACK))
            .clicked()
        {
            navigate = Some(back);
        }
        ui.add(
            Label::new(rich(
                &descriptor.name,
                Role::Title,
                color::TEXT_PRIMARY.get(),
            ))
            .truncate(),
        );
    });
    let mut line = vec![s::kind(descriptor.kind).to_owned()];
    line.extend(descriptor.vendor.clone());
    ui.label(rich(
        line.join(" · "),
        Role::Body,
        color::TEXT_SECONDARY.get(),
    ));
    ui.add_space(space::XL);

    match traits {
        Some(traits) => columns(
            ui,
            LIGHTING_SHARE,
            &mut (model, state),
            |ui, pair| lighting::section(ui, pair.0, slot, traits, pair.1),
            |ui, _| about(ui, &descriptor),
        ),
        None if dormant => {
            let mut open = false;
            columns(
                ui,
                LIGHTING_SHARE,
                &mut open,
                |ui, open| *open = dormant_card(ui),
                |ui, _| about(ui, &descriptor),
            );
            if open {
                navigate = Some((Route::Settings, Style::Entrance));
            }
        }
        None => about(ui, &descriptor),
    }
    navigate
}

/// A recognized device whose experimental driver is off. Returns whether settings were asked for.
fn dormant_card(ui: &mut Ui) -> bool {
    card(ui, |ui| {
        section_title(ui, Icon::Caution, s::DEVICE_DORMANT);
        hint(ui, s::DEVICE_DORMANT_HINT);
        ui.add_space(space::M);
        ui.add(Button::new(s::DEVICE_OPEN_SETTINGS).icon(Icon::Settings))
            .clicked()
    })
}

/// The lighting section takes the larger column on wide windows.
const LIGHTING_SHARE: f32 = 0.62;

fn about(ui: &mut Ui, descriptor: &DeviceDescriptor) {
    card(ui, |ui| {
        section_title(ui, Icon::for_kind(descriptor.kind), s::ABOUT_DEVICE);
        field(ui, s::LABEL_KIND, s::kind(descriptor.kind), false);
        if let Some(vendor) = &descriptor.vendor {
            field(ui, s::LABEL_VENDOR, vendor, false);
        }
        let driver = format!(
            "{} · {}",
            descriptor.driver.name,
            s::maturity(descriptor.driver.maturity)
        );
        field(ui, s::LABEL_DRIVER, &driver, false);
        for detail in &descriptor.details {
            let (name, value) = s::detail(detail);
            field(ui, name, &value, true);
        }
    });
}
