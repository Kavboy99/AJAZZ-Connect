//! Pages. Each one draws from the application state and returns where to go next, if anywhere.
//! Page state (scroll position, entered text) lives here and survives leaving the page.

mod device;
mod device_hero;
mod device_view;
mod devices;
mod diagnostics;
mod layout;
mod lighting;
mod preview;
mod settings;
mod studio;

pub use layout::set_settling;

use crate::app::{Diagnostics, Events, Session, Settings};
use crate::core::DriverInfo;
use crate::ui::motion::{KineticScroll, Style};
use crate::ui::theme::size;
use eframe::egui::{Frame, Id, Margin, Ui};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Route {
    Devices,
    Device(usize),
    Preview(usize),
    Diagnostics,
    Settings,
}

pub type Navigate = Option<(Route, Style)>;

/// What a page may read and change.
pub struct Model<'a> {
    pub session: &'a mut Session,
    pub diagnostics: &'a mut Diagnostics,
    pub settings: &'a mut Settings,
    pub events: &'a Events,
    pub drivers: &'a [DriverInfo],
    pub animations: bool,
}

#[derive(Default)]
pub struct Pages {
    scroll: HashMap<Route, KineticScroll>,
    lighting: HashMap<usize, studio::State>,
    preview_lighting: HashMap<usize, preview::State>,
    settings: settings::State,
}

impl Pages {
    /// Device list changed: per-device state no longer matches the slots.
    pub fn forget_devices(&mut self) {
        self.lighting.clear();
        self.scroll
            .retain(|route, _| !matches!(route, Route::Device(_)));
    }

    pub fn show(&mut self, ui: &mut Ui, route: Route, model: &mut Model<'_>) -> Navigate {
        if matches!(route, Route::Device(_) | Route::Preview(_)) {
            return Frame::new()
                .inner_margin(Margin::same(size::PAGE_MARGIN as i8))
                .show(ui, |ui| match route {
                    Route::Device(slot) => {
                        device::show(ui, model, slot, self.lighting.entry(slot).or_default())
                    }
                    Route::Preview(index) => preview::show(
                        ui,
                        model,
                        index,
                        self.preview_lighting.entry(index).or_default(),
                    ),
                    _ => None,
                })
                .inner;
        }
        let scroll = self.scroll.entry(route).or_default();
        let lighting = &mut self.lighting;
        let settings = &mut self.settings;
        let preview_lighting = &mut self.preview_lighting;
        let bars = route == Route::Devices || !crate::ui::device_reveal::moving(ui.ctx());
        scroll.show(ui, Id::new(("page", route)), bars, |ui| {
            Frame::new()
                .inner_margin(Margin::same(size::PAGE_MARGIN as i8))
                .show(ui, |ui| match route {
                    Route::Devices => devices::show(ui, model),
                    Route::Preview(index) => {
                        preview::show(ui, model, index, preview_lighting.entry(index).or_default())
                    }
                    Route::Device(slot) => {
                        device::show(ui, model, slot, lighting.entry(slot).or_default())
                    }
                    Route::Diagnostics => diagnostics::show(ui, model),
                    Route::Settings => settings::show(ui, model, settings),
                })
                .inner
        })
    }
}
