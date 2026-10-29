//! A viewport-sized device workspace. The inspector and stage never scroll.
mod apply;
mod framing;
mod header;
mod inspector;
mod island;
mod notices;
mod overview;
mod preferences;
mod program_cards;
mod programs;
mod stage;
mod state;
#[cfg(test)]
mod tests;
pub(crate) use state::{Mode, State};

use super::device_view::DeviceView;
use super::{Model, Route};
use crate::core::LightingTraits;
use crate::ui::device_reveal;
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{breakpoint, size, space};
use eframe::egui::{Align, Id, Layout, Rect, Ui, UiBuilder, pos2, vec2};

pub(super) struct DevicePage<'a> {
    pub route: Route,
    pub view: DeviceView<'a>,
    pub dormant: bool,
    pub traits: Option<LightingTraits>,
}

pub(super) fn show(
    ui: &mut Ui,
    model: &mut Model<'_>,
    page: DevicePage<'_>,
    state: &mut State,
) -> bool {
    let live_editor = if let Route::Device(slot) = page.route {
        model
            .session
            .entries()
            .get(slot)
            .and_then(|e| e.lighting_editor())
    } else {
        None
    };
    let preview = page
        .view
        .driver
        .and_then(crate::app::catalog::lighting_preview);
    let first_sync = state.programs.is_none();
    let preference_key = format!(
        "{}:{}",
        page.view.driver.map_or("", |d| d.id),
        page.view.name
    );
    programs::sync(state, live_editor.or_else(|| preview.map(|p| &p.editor)));
    let failed = if let Route::Device(slot) = page.route {
        model.session.entries().get(slot).is_some_and(|entry| {
            matches!(entry.status(), Some(crate::app::session::Status::Failed(_)))
        })
    } else {
        false
    };
    if let Some(programs) = &mut state.programs {
        programs.observe_device(live_editor.map(|editor| editor.current), failed);
    }
    state.apply.prepare(model, &page);
    if first_sync {
        preferences::restore(state, model.settings, &preference_key);
    }
    state.preview_requires_usb = live_editor.is_none() && preview.is_some_and(|p| p.requires_usb);
    let before_preferences = state
        .programs
        .as_ref()
        .map(|p| (p.selected, p.presets.clone()));
    let id = Id::new(("studio", page.route));
    let mut back = header::show(ui, &page, state);
    // Escape is consumed before the shell can reverse the device reveal.
    if ui.input_mut(|i| i.consume_key(eframe::egui::Modifiers::NONE, eframe::egui::Key::Escape)) {
        back = state.back();
    }
    ui.add_space(space::S);
    let bounds = ui.available_rect_before_wrap();
    let compact = bounds.height() < size::STUDIO_SHORT_HEIGHT;
    let editing = state.mode != Mode::Overview;
    let editor = motion::animate(
        ui.ctx(),
        id.with("editor"),
        f32::from(editing),
        tokens::WORKSPACE,
        Channel::Scale,
    )
    .clamp(0.0, 1.0);
    let panel = motion::animate(
        ui.ctx(),
        id.with("panel"),
        f32::from(!editing || state.inspector),
        tokens::WORKSPACE,
        Channel::Scale,
    )
    .clamp(0.0, 1.0);
    let dock_height = if compact {
        size::STUDIO_ISLAND_COMPACT
    } else {
        size::STUDIO_ISLAND
    };
    let stage_bottom = bounds.bottom() - (dock_height + space::L) * editor;
    let wide = super::layout::settled_width(ui) >= breakpoint::SECTION_COLUMNS;
    let panel_width = size::STUDIO_PANEL.min(bounds.width() * 0.43);
    let panel_space = if wide {
        (panel_width + space::XL) * panel
    } else {
        0.0
    };
    let mut stage =
        Rect::from_min_max(bounds.min, pos2(bounds.right() - panel_space, stage_bottom));
    let mut destination = Rect::from_min_max(
        bounds.min,
        pos2(
            bounds.right()
                - if wide && (!editing || state.inspector) {
                    panel_width + space::XL
                } else {
                    0.0
                },
            bounds.bottom() - if editing { dock_height + space::L } else { 0.0 },
        ),
    );
    if !wide && (!editing || state.inspector) {
        destination.max.y -= size::STUDIO_ISLAND;
    }
    if !wide && panel > 0.0 {
        stage.max.y -= size::STUDIO_ISLAND * panel;
    }
    let stacked_panel_top = stage.bottom();
    let notice_space = if editing {
        let (reserved, target) = notices::show(ui, stage, destination, model, &page, state);
        destination = target;
        stage.bottom() - reserved.bottom()
    } else {
        0.0
    };
    stage.max.y -= motion::animate(
        ui.ctx(),
        id.with("notice-space"),
        notice_space,
        tokens::WORKSPACE,
        Channel::Offset,
    );
    let target = device_reveal::stage(ui.ctx(), stage);
    stage::draw(ui, id, &page, target, destination, state);
    crate::ui::viewer::appearance::persist(ui.ctx(), model.settings);
    if panel > 0.005 {
        let panel_rect = if wide {
            Rect::from_min_max(
                pos2(
                    bounds.right() - panel_width,
                    bounds.top()
                        + if editing {
                            space::L
                        } else {
                            ((bounds.height() - size::DEVICE_HERO_HEIGHT) * 0.4).max(space::L)
                        },
                ),
                pos2(bounds.right(), stage_bottom),
            )
        } else {
            Rect::from_min_max(
                pos2(bounds.left(), stacked_panel_top),
                pos2(bounds.right(), stage_bottom),
            )
        };
        let mut info = ui.new_child(
            UiBuilder::new()
                .id_salt("studio-panel")
                .max_rect(panel_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        info.set_clip_rect(panel_rect.intersect(ui.clip_rect()));
        info.multiply_opacity(panel * device_reveal::content_opacity(ui.ctx()));
        if editing && state.showing_inspector() {
            if !state.inspector {
                info.disable();
            }
            inspector::show(&mut info, model, &page, state);
        } else {
            if editing {
                info.disable();
            }
            overview::show(&mut info, &page, state);
        }
    }
    if editor > 0.005 {
        let rect = Rect::from_min_size(
            pos2(
                bounds.left(),
                bounds.bottom() + space::L - (dock_height + space::L) * editor,
            ),
            vec2(bounds.width(), dock_height),
        );
        let mut dock = ui.new_child(UiBuilder::new().id_salt("studio-island").max_rect(rect));
        dock.set_clip_rect(bounds.intersect(ui.clip_rect()));
        dock.multiply_opacity(editor);
        if !editing {
            dock.disable();
        }
        island::show(&mut dock, state, compact);
    }
    // Reserving the viewport, rather than growing with child content, prevents page scrolling.
    ui.advance_cursor_after_rect(bounds);
    state.apply.dispatch(model, &page);
    state.preferences_dirty |= before_preferences
        != state
            .programs
            .as_ref()
            .map(|p| (p.selected, p.presets.clone()));
    if state.preferences_dirty && !ui.input(|i| i.pointer.any_down()) {
        state.preferences_failed =
            preferences::save(state, model.settings, &preference_key).is_err();
    }
    if !ui.input(|i| i.pointer.any_down()) {
        state.preferences_dirty = false;
    }
    back
}
