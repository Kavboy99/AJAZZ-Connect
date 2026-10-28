//! Offline design previews never enter the live device session.
use super::device_view::DeviceView;
use super::{Model, Navigate, Route, layout, studio};
use crate::app::catalog::CatalogDevice;
use crate::ui::theme::{size, space};
use crate::ui::widgets::{Button, TileContent, Tone, device_tile};
use crate::ui::{icons::Icon, motion::Style, strings as s};
use eframe::egui::{Id, Ui};
use std::collections::HashMap;

#[derive(Default)]
pub struct State {
    selected: usize,
    lighting: HashMap<usize, studio::State>,
}

fn tile(ui: &mut Ui, index: usize, side: f32, group: &CatalogDevice) -> bool {
    let d = DeviceView::catalog(group, 0);
    let caption = if group.variants().len() > 1 {
        s::DEVICE_MULTIPLE_DRIVERS
    } else {
        d.driver.map_or(s::DEVICE_NO_DRIVER, |driver| driver.name)
    };
    let response = device_tile(
        ui,
        Id::new(("catalog-tile", index)),
        side,
        &TileContent {
            name: group.name(),
            caption,
            icon: Icon::for_kind(d.kind),
            visual: d.visual,
            busy: false,
            badges: &[(
                Tone::Info,
                if d.driver.is_none() {
                    s::DEVICE_PLACEHOLDER
                } else {
                    s::DEVICE_CATALOG
                },
            )],
        },
    );
    crate::ui::device_reveal::remember(ui.ctx(), Route::Preview(index), response.rect);
    response.clicked()
}

pub fn show(ui: &mut Ui, model: &mut Model<'_>, index: usize, state: &mut State) -> Navigate {
    if !model.settings.developer_mode || !model.settings.show_all_devices {
        return Some((Route::Devices, Style::DrillOut));
    }
    let catalog = crate::app::catalog::devices(model.drivers);
    let Some(group) = catalog.get(index) else {
        return Some((Route::Devices, Style::DrillOut));
    };
    if group.variants().len() > 1 {
        layout::hint(ui, s::DEVICE_PREVIEW_DRIVER);
        ui.horizontal_wrapped(|ui| {
            for (variant, d) in group.variants().iter().enumerate() {
                let button = Button::new(d.driver.name);
                let button = if state.selected == variant {
                    button.primary()
                } else {
                    button
                };
                if ui.add(button).clicked() {
                    state.selected = variant;
                }
            }
        });
        ui.add_space(space::M);
    }
    let d = DeviceView::catalog(group, state.selected);
    let page = studio::DevicePage {
        route: Route::Preview(index),
        view: d,
        dormant: false,
        traits: None,
    };
    studio::show(
        ui,
        model,
        page,
        state.lighting.entry(state.selected).or_default(),
    )
    .then_some((Route::Devices, Style::DrillOut))
}

pub fn grid(ui: &mut Ui, model: &Model<'_>) -> Navigate {
    let catalog = crate::app::catalog::devices(model.drivers);
    let visible: Vec<_> = catalog.iter().enumerate().collect();
    let gap = size::CARD_GAP;
    let columns = ((layout::settled_width(ui) + gap) / (size::DEVICE_TILE_MIN + gap))
        .floor()
        .max(1.0) as usize;
    let side = ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32).floor();
    let mut navigate = None;
    for chunk in visible.chunks(columns) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for &(index, group) in chunk {
                if tile(ui, index, side, group) {
                    navigate = Some((Route::Preview(index), Style::DrillIn));
                }
            }
        });
        ui.add_space(gap);
    }
    navigate
}
