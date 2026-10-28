//! Native presets use the same framed selection surface and motion as the original mode cards.
use super::State;
use crate::ui::theme::{Role, color, label, radius, size, space};
use crate::ui::widgets::{Button, Interaction, focus_ring};
use crate::ui::{icons::Icon, strings as s};
use eframe::egui::{
    Align, CornerRadius, Layout, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2,
};

pub(super) fn show(ui: &mut Ui, state: &mut State) {
    let Some(programs) = &state.programs else {
        return;
    };
    let mut selected = programs.selected;
    let mut open = false;
    let width = ((ui.available_width() - space::M * 2.0) / 3.0).max(size::STUDIO_PANEL * 0.65);
    let height = ui.available_height() - space::S;
    let chosen = state.palette.colors(1)[0];
    let phase = state.demonstration_phase();
    ui.style_mut().always_scroll_the_only_direction = true;
    eframe::egui::ScrollArea::horizontal()
        .id_salt("native-presets")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::M;
                for (index, preset) in programs.presets.iter().enumerate() {
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(width, height), Sense::click());
                    let active = index == programs.selected;
                    let interaction = Interaction::of(ui, response.id, &response);
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(radius::CONTROL),
                        interaction.fill(
                            if active {
                                color::ACCENT_SUBTLE.get()
                            } else {
                                color::SURFACE_SUNKEN.get()
                            },
                            color::CONTROL_FILL_HOVER.get(),
                            color::CONTROL_FILL_PRESSED.get(),
                        ),
                    );
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::same(radius::CONTROL),
                        Stroke::new(
                            size::STROKE,
                            if active {
                                color::ACCENT.get()
                            } else {
                                color::STROKE_DEFAULT.get()
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    focus_ring(ui.painter(), rect, radius::CONTROL, interaction.focus);
                    let mut card = ui.new_child(
                        UiBuilder::new()
                            .id_salt(index)
                            .max_rect(rect.shrink(space::S)),
                    );
                    card.horizontal(|ui| {
                        if active {
                            Icon::Success.show(ui, size::ICON_SMALL, color::ACCENT.get());
                        }
                        label(
                            ui,
                            s::lighting_effect(preset.effect),
                            Role::BodyStrong,
                            color::TEXT_PRIMARY.get(),
                        );
                    });
                    let sample = Rect::from_min_max(
                        rect.min + vec2(space::L, size::CONTROL_HEIGHT + space::S),
                        rect.max - vec2(space::L, size::CONTROL_HEIGHT + space::M),
                    );
                    let cell = (sample.width() / 14.0).min(sample.height() / 4.0).max(1.0);
                    let origin = sample.center() - vec2(cell * 7.0, cell * 2.0);
                    for row in 0..4 {
                        for col in 0..14 {
                            let lit = crate::ui::viewer::native_effect::sample(
                                preset.effect,
                                col as f32 / 13.0,
                                row as f32 / 3.0,
                                if active {
                                    phase * (0.5 + f32::from(preset.saved.speed) * 0.5)
                                } else {
                                    0.45
                                },
                                0.0,
                                chosen,
                            );
                            let ink = color::STROKE_DEFAULT.get().lerp_to_gamma(lit, 0.85);
                            card.painter().rect_filled(
                                Rect::from_min_size(
                                    origin + vec2(col as f32 * cell, row as f32 * cell),
                                    vec2(cell - size::STROKE * 2.0, cell - size::STROKE * 2.0),
                                ),
                                CornerRadius::same(radius::CONTROL),
                                ink,
                            );
                        }
                    }
                    let mut controls = card.new_child(
                        UiBuilder::new()
                            .max_rect(Rect::from_min_max(
                                pos2(
                                    rect.left() + space::S,
                                    rect.bottom() - size::CONTROL_HEIGHT - space::S,
                                ),
                                rect.max - vec2(space::S, space::S),
                            ))
                            .layout(Layout::right_to_left(Align::Center)),
                    );
                    if controls
                        .add(Button::new(s::STUDIO_SETTINGS).icon(Icon::Settings))
                        .clicked()
                    {
                        selected = index;
                        open = true;
                    }
                    if response.clicked() {
                        selected = index;
                    }
                }
            });
        });
    if let Some(programs) = &mut state.programs
        && selected != programs.selected
    {
        programs.selected = selected;
        super::programs::update_preview(state);
    }
    if open {
        state.open_inspector();
    }
}
