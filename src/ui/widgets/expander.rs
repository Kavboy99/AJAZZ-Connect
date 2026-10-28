//! A collapsible group. Height and opacity grow toward the target on `spring.standard`, so the
//! content below moves instead of jumping.

use super::interaction::{Interaction, focus_ring};
use crate::ui::icons::Icon;
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{Role, color, radius, size, space};
use eframe::egui::{
    Color32, CornerRadius, Id, Rect, Sense, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};
use std::f32::consts::FRAC_PI_2;

pub fn expander(ui: &mut Ui, id: Id, title: &str, default_open: bool, add: impl FnOnce(&mut Ui)) {
    let mut open = ui.data_mut(|d| *d.get_temp_mut_or(id, default_open));
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, size::NAV_ROW), Sense::click());
    if response.clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    response.widget_info(|| WidgetInfo::selected(WidgetType::CollapsingHeader, true, open, title));
    let state = Interaction::of(ui, response.id, &response);
    let p = motion::animate(
        ui.ctx(),
        id.with("open"),
        if open { 1.0 } else { 0.0 },
        tokens::STANDARD,
        Channel::Fade,
    );
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same(radius::CONTROL),
        state.fill(
            Color32::TRANSPARENT,
            color::CONTROL_FILL_HOVER.get(),
            color::CONTROL_FILL_PRESSED.get(),
        ),
    );
    let chevron = Rect::from_center_size(
        pos2(
            rect.left() + space::S + size::ICON_SMALL / 2.0,
            rect.center().y,
        ),
        Vec2::splat(size::ICON_SMALL),
    );
    Icon::ChevronRight.paint_rotated(painter, chevron, color::TEXT_SECONDARY.get(), p * FRAC_PI_2);
    let galley = ui.painter().layout_no_wrap(
        title.to_owned(),
        Role::BodyStrong.font(),
        color::TEXT_PRIMARY.get(),
    );
    ui.painter().galley(
        pos2(
            chevron.right() + space::S,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        color::TEXT_PRIMARY.get(),
    );
    focus_ring(ui.painter(), rect, radius::CONTROL, state.focus);

    if p <= 0.0 {
        return;
    }
    let full_id = id.with("height");
    let full: f32 = ui.data(|d| d.get_temp(full_id)).unwrap_or(0.0);
    let visible = (full * p).round();
    let top = ui.cursor().min;
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(Rect::from_min_size(top, vec2(width, f32::INFINITY)))
            .layout(*ui.layout()),
    );
    child.set_clip_rect(
        child
            .clip_rect()
            .intersect(Rect::from_min_size(top, vec2(width, visible))),
    );
    child.multiply_opacity(p);
    child.add_space(space::S);
    add(&mut child);
    let measured = child.min_rect().height();
    if (measured - full).abs() > 0.5 {
        ui.data_mut(|d| d.insert_temp(full_id, measured));
        // Layout measurement, not motion: redo this pass with the real height.
        ui.ctx().request_discard("expander height");
    }
    ui.allocate_exact_size(vec2(width, visible), Sense::hover());
}
