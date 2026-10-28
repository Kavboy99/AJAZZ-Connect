use super::device_view::DeviceView;
use super::studio::{self, DevicePage, State};
use super::{Model, Navigate, Route};
use crate::ui::motion::Style;
use crate::ui::strings as s;
use crate::ui::widgets::Button;
use eframe::egui::Ui;

pub fn show(ui: &mut Ui, model: &mut Model<'_>, slot: usize, state: &mut State) -> Navigate {
    let Some(entry) = model.session.entries().get(slot) else {
        super::layout::hint(ui, s::DEVICE_GONE);
        return Some((Route::Devices, Style::DrillOut));
    };
    let descriptor = entry.descriptor().clone();
    let traits = entry.lighting();
    let dormant = entry.is_dormant();
    if dormant
        && ui
            .add(Button::new(s::DEVICE_OPEN_SETTINGS))
            .on_hover_text(s::DEVICE_DORMANT_HINT)
            .clicked()
    {
        return Some((Route::Settings, Style::Entrance));
    }
    if model.settings.developer_mode && model.settings.show_all_devices {
        let catalog = crate::app::catalog::devices(model.drivers);
        if let Some(index) = catalog.iter().position(|group| group.matches(&descriptor))
            && ui.add(Button::new(s::DEVICE_PREVIEW_VARIANTS)).clicked()
        {
            return Some((Route::Preview(index), Style::DrillIn));
        }
    }
    let page = DevicePage {
        route: Route::Device(slot),
        view: DeviceView::from(&descriptor),
        dormant,
        traits,
    };
    studio::show(ui, model, page, state).then_some((Route::Devices, Style::DrillOut))
}
