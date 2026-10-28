//! The mode island: title, visual sample and a settings action for each choice.
use super::{Mode, State};
use crate::ui::theme::{Role, color, label, radius, size, space};
use crate::ui::widgets::{Button, Interaction, focus_ring};
use crate::ui::{icons::Icon, strings as s, viewer::Effect};
use eframe::egui::{
    Align, Align2, CornerRadius, Layout, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder,
    WidgetInfo, WidgetType, pos2, vec2,
};

pub(super) fn show(ui: &mut Ui, state: &mut State, compact: bool) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(radius::OVERLAY),
        color::SURFACE_CARD.get(),
    );
    let padding = if compact { space::S } else { space::M };
    let mut rect = rect.shrink(padding);
    ui.scope_builder(
        UiBuilder::new().max_rect(Rect::from_min_size(
            rect.min,
            vec2(rect.width(), size::CONTROL_HEIGHT),
        )),
        |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(Button::icon_only(Icon::Close, s::STUDIO_CLOSE_EDITOR))
                    .clicked()
                {
                    state.close_editor();
                }
            });
        },
    );
    rect.min.y += size::CONTROL_HEIGHT + space::S;
    if state.mode == Mode::Lighting && state.programs.is_some() {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            super::programs::island(ui, state)
        });
        return;
    }
    let width = (rect.width() - 2.0 * space::M) / 3.0;
    for index in 0..3 {
        let tile = Rect::from_min_size(
            rect.min + vec2(index as f32 * (width + space::M), 0.0),
            vec2(width, rect.height()),
        );
        ui.scope_builder(
            UiBuilder::new()
                .id_salt(index)
                .max_rect(tile)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                choice(ui, state, index, compact);
            },
        );
    }
}

fn choice(ui: &mut Ui, state: &mut State, index: usize, compact: bool) {
    let lighting = state.mode != Mode::Encoder;
    let effects = [Effect::Solid, Effect::Wave, Effect::Reactive];
    let titles = if lighting {
        [s::STUDIO_SOLID, s::STUDIO_WAVE, s::STUDIO_REACTIVE]
    } else {
        [s::STUDIO_VOLUME, s::STUDIO_SCROLL, s::STUDIO_BRIGHTNESS]
    };
    let selected = if lighting {
        state.effect == effects[index]
    } else {
        state.encoder == index
    };
    let rect = ui.max_rect();
    let select_rect = rect;
    let response = ui.interact(select_rect, ui.id().with("select"), Sense::click());
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::SelectableLabel, true, selected, titles[index])
    });
    let interaction = Interaction::of(ui, response.id, &response);
    ui.painter().rect_filled(
        select_rect,
        CornerRadius::same(radius::CONTROL),
        interaction.fill(
            if selected {
                color::ACCENT_SUBTLE.get()
            } else {
                color::SURFACE_SUNKEN.get()
            },
            color::CONTROL_FILL_HOVER.get(),
            color::CONTROL_FILL_PRESSED.get(),
        ),
    );
    if selected {
        ui.painter().rect_stroke(
            select_rect,
            CornerRadius::same(radius::CONTROL),
            Stroke::new(size::STROKE, color::ACCENT.get()),
            StrokeKind::Inside,
        );
    }
    focus_ring(
        ui.painter(),
        select_rect,
        radius::CONTROL,
        interaction.focus,
    );
    let mut content = ui.new_child(UiBuilder::new().max_rect(select_rect.shrink(space::S)));
    content.horizontal(|ui| {
        if selected {
            Icon::Success.show(ui, size::ICON_SMALL, color::ACCENT.get());
        }
        label(
            ui,
            titles[index],
            Role::BodyStrong,
            color::TEXT_PRIMARY.get(),
        );
    });
    let sample = Rect::from_min_max(
        pos2(
            select_rect.left() + space::L,
            select_rect.top() + size::CONTROL_HEIGHT + space::L,
        ),
        select_rect.max - vec2(space::L, size::CONTROL_HEIGHT + space::L),
    );
    if lighting {
        keyboard_sample(ui, sample, index, state.palette.colors(1)[0]);
    } else {
        let center = sample.center();
        let r = (sample.height() * 0.4).max(space::S);
        ui.painter()
            .circle_filled(center, r, color::CONTROL_FILL.get());
        ui.painter().circle_stroke(
            center,
            r,
            Stroke::new(size::FOCUS_RING, color::ACCENT.get()),
        );
        ui.painter().line_segment(
            [center, center + vec2(r * 0.6, -r * 0.5)],
            Stroke::new(size::FOCUS_RING, color::TEXT_PRIMARY.get()),
        );
        if !compact {
            ui.painter().text(
                center + vec2(r + space::L, 0.0),
                Align2::LEFT_CENTER,
                ["− / +", "↑ / ↓", "− / +"][index],
                Role::Body.font(),
                color::TEXT_SECONDARY.get(),
            );
        }
    }
    let button_rect = Rect::from_min_max(
        pos2(
            rect.left() + space::S,
            rect.bottom() - size::CONTROL_HEIGHT - space::S,
        ),
        rect.max - vec2(space::S, space::S),
    );
    let mut controls = ui.new_child(
        UiBuilder::new()
            .max_rect(button_rect)
            .layout(Layout::right_to_left(Align::Center)),
    );
    let open = controls
        .add(Button::new(s::STUDIO_SETTINGS).icon(Icon::Settings))
        .clicked();
    let sample_click = ui
        .interact(sample, ui.id().with("sample"), Sense::click())
        .clicked();
    let title_rect = Rect::from_min_size(
        rect.min + vec2(space::S, space::S),
        vec2(rect.width() - space::L, size::CONTROL_HEIGHT),
    );
    let title_click = ui
        .interact(title_rect, ui.id().with("title"), Sense::click())
        .clicked();
    if response.clicked() || sample_click || title_click || open {
        if lighting {
            state.effect = effects[index];
        } else {
            state.encoder = index;
        }
        state.replay();
        if open {
            state.open_inspector();
        }
    }
}

pub(super) fn keyboard_sample(ui: &Ui, rect: Rect, mode: usize, chosen: eframe::egui::Color32) {
    let cell = (rect.width() / 14.0).min(rect.height() / 4.0);
    let origin = rect.center() - vec2(cell * 7.0, cell * 2.0);
    for row in 0..4 {
        for col in 0..14 {
            let c = match mode {
                1 => eframe::egui::ecolor::Hsva::new(col as f32 / 14.0, 0.8, 0.9, 1.0).into(),
                2 if (col + row) % 4 != 0 => color::STROKE_DEFAULT.get(),
                _ => chosen,
            };
            let key = Rect::from_min_size(
                origin + vec2(col as f32 * cell, row as f32 * cell),
                vec2(cell - size::STROKE * 2.0, cell - size::STROKE * 2.0),
            );
            ui.painter()
                .rect_filled(key, CornerRadius::same(radius::CONTROL), c);
        }
    }
}
