//! Device identity and capability entry points in the resting workspace.
use super::{DevicePage, Mode, Route, State};
use crate::ui::theme::{size, space};
use crate::ui::widgets::ActionTile;
use crate::ui::{icons::Icon, strings as s, viewer};
use eframe::egui::{Ui, UiBuilder};

pub(super) fn show(ui: &mut Ui, page: &DevicePage<'_>, state: &mut State) {
    super::super::device_hero::identity(
        ui,
        &page.view,
        matches!(page.route, Route::Preview(_)),
        page.dormant,
    );
    ui.add_space(space::L);
    let binding = viewer::binding(page.view.visual);
    let actions = [
        (
            page.traits.is_some() || binding.is_some_and(|b| b.has_lighting()),
            Mode::Lighting,
            Icon::Lighting,
            s::STUDIO_LIGHTING,
        ),
        (
            binding.is_some_and(|b| b.has_encoder()),
            Mode::Encoder,
            Icon::Settings,
            s::STUDIO_ENCODER,
        ),
    ];
    let width = ui.available_width();
    let columns = ((width + space::M) / (size::DEVICE_ACTION_MIN + space::M))
        .floor()
        .max(1.0) as usize;
    let side =
        ((width - space::M * (columns - 1) as f32) / columns as f32).min(size::DEVICE_ACTION_MAX);
    let visible: Vec<_> = actions.into_iter().filter(|a| a.0).collect();
    eframe::egui::ScrollArea::vertical()
        .id_salt("device-actions")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = eframe::egui::vec2(space::M, space::M);
            for row in visible.chunks(columns) {
                ui.horizontal(|ui| {
                    for &(_, mode, icon, label) in row {
                        if ui
                            .scope_builder(
                                UiBuilder::new()
                                    .id(ui.make_persistent_id(("device-action", label))),
                                |ui| ui.add(ActionTile::new(icon, label, side)),
                            )
                            .inner
                            .clicked()
                        {
                            state.enter(mode);
                        }
                    }
                });
            }
        });
}
