//! A viewport-sized device workspace. Only its optional inspector may scroll.
mod inspector;
mod island;
mod program_cards;
mod programs;
mod stage;
mod state;
pub(crate) use state::{Mode, State};

use super::device_view::DeviceView;
use super::{Model, Route};
use crate::core::LightingTraits;
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{Role, breakpoint, color, label, size, space};
use crate::ui::widgets::Button;
use crate::ui::{device_reveal, icons::Icon, strings as s, viewer};
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
    programs::sync(state, live_editor.or_else(|| preview.map(|p| &p.editor)));
    state.preview_requires_usb = live_editor.is_none() && preview.is_some_and(|p| p.requires_usb);
    let id = Id::new(("studio", page.route));
    let mut back = false;
    ui.horizontal(|ui| {
        if ui
            .add(Button::new(s::BACK).icon(Icon::ChevronLeft))
            .clicked()
        {
            back = true;
        }
        if state.mode != Mode::Overview {
            label(
                ui,
                if state.mode == Mode::Lighting {
                    s::LIGHTING
                } else {
                    s::STUDIO_ENCODER
                },
                Role::BodyStrong,
                color::TEXT_PRIMARY.get(),
            );
            label(
                ui,
                state.choice_label(),
                Role::Caption,
                color::TEXT_SECONDARY.get(),
            );
            if state.mode == Mode::Lighting
                && state.effect == viewer::Effect::Wave
                && ui
                    .add(Button::new(s::STUDIO_REPLAY).icon(Icon::Lighting))
                    .clicked()
            {
                state.replay();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if state.inspector
                    && ui
                        .add(Button::icon_only(
                            Icon::ChevronRight,
                            s::STUDIO_CLOSE_SETTINGS,
                        ))
                        .clicked()
                {
                    state.inspector = false;
                }
            });
        }
    });
    // Escape is consumed before the shell can reverse the device reveal.
    if ui.input_mut(|i| i.consume_key(eframe::egui::Modifiers::NONE, eframe::egui::Key::Escape)) {
        back = state.back();
    }
    ui.add_space(space::S);
    if state.mode == Mode::Lighting && state.preview_requires_usb {
        label(
            ui,
            if matches!(page.route, Route::Preview(_)) {
                s::LIGHTING_OFFLINE_CONTROLS
            } else if page.dormant {
                s::DEVICE_DORMANT_HINT
            } else {
                s::LIGHTING_CONNECT_USB
            },
            Role::Caption,
            color::TEXT_SECONDARY.get(),
        );
        ui.add_space(space::S);
    }
    let bounds = ui.available_rect_before_wrap();
    let compact = bounds.height() < size::STUDIO_SHORT_HEIGHT;
    let editing = state.mode != Mode::Overview;
    let editor = motion::animate(
        ui.ctx(),
        id.with("editor"),
        f32::from(editing),
        tokens::PAGE,
        Channel::Scale,
    )
    .clamp(0.0, 1.0);
    let panel = motion::animate(
        ui.ctx(),
        id.with("panel"),
        f32::from(!editing || state.inspector),
        tokens::PAGE,
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
    if !wide && panel > 0.0 {
        stage.max.y -= size::STUDIO_ISLAND * panel;
    }
    let target = device_reveal::stage(ui.ctx(), stage);
    stage::draw(ui, id, &page, target, state);
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
                pos2(bounds.left(), stage.bottom()),
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
            overview(&mut info, &page, state, compact);
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
    back
}

fn overview(ui: &mut Ui, page: &DevicePage<'_>, state: &mut State, compact: bool) {
    super::device_hero::identity(
        ui,
        &page.view,
        matches!(page.route, Route::Preview(_)),
        page.dormant,
    );
    ui.add_space(space::L);
    let binding = viewer::binding(page.view.visual);
    if (page.traits.is_some() || binding.is_some_and(|b| b.has_lighting()))
        && ui
            .add(
                Button::new(s::STUDIO_LIGHTING)
                    .icon(Icon::Lighting)
                    .primary(),
            )
            .clicked()
    {
        state.enter(Mode::Lighting);
    }
    if binding.is_some_and(|b| b.has_encoder())
        && ui
            .add(Button::new(s::STUDIO_ENCODER).icon(Icon::Settings))
            .clicked()
    {
        state.enter(Mode::Encoder);
    }
    ui.add_space(space::S);
    label(
        ui,
        s::STUDIO_PREVIEW,
        Role::Caption,
        color::TEXT_TERTIARY.get(),
    );
    if !compact {
        ui.add_space(space::L);
        for detail in page.view.details {
            if matches!(
                detail,
                crate::core::Detail::Battery(_) | crate::core::Detail::Power(_)
            ) {
                continue;
            }
            let (name, value) = s::detail(detail);
            super::layout::spans(
                ui,
                &[
                    (name, Role::Caption, color::TEXT_SECONDARY.get()),
                    (&value, Role::Mono, color::TEXT_PRIMARY.get()),
                ],
            );
        }
    } else {
        ui.collapsing(s::STUDIO_MORE, |ui| {
            for detail in page.view.details {
                let (name, value) = s::detail(detail);
                label(
                    ui,
                    format!("{name} · {value}"),
                    Role::Caption,
                    color::TEXT_SECONDARY.get(),
                );
            }
        });
    }
}
