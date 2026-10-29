//! Reusable square entry point for every device capability.
use super::{Interaction, focus_ring};
use crate::ui::icons::Icon;
use crate::ui::theme::{Role, color, radius, size, space};
use eframe::egui::{
    Align, Align2, CornerRadius, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Widget,
    WidgetInfo, WidgetType, emath::TSTransform, vec2,
};

pub struct ActionTile<'a> {
    icon: Icon,
    label: &'a str,
    side: f32,
}

impl<'a> ActionTile<'a> {
    pub fn new(icon: Icon, label: &'a str, side: f32) -> Self {
        Self { icon, label, side }
    }
}

impl Widget for ActionTile<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(self.side, self.side), Sense::click());
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), self.label));
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let state = Interaction::of(ui, response.id, &response);
        let ink = color::TEXT_PRIMARY.get();
        let mut text = eframe::egui::text::LayoutJob::simple(
            self.label.to_owned(),
            Role::Body.font(),
            ink,
            (self.side - space::L * 2.0).max(space::L),
        );
        text.halign = Align::Center;
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(text));
        let content_height = size::DEVICE_ACTION_ICON + space::M + galley.size().y;
        let top = rect.center().y - content_height / 2.0;
        let transform =
            TSTransform::new(rect.center().to_vec2() * (1.0 - state.scale), state.scale);
        let mut shapes = [
            Shape::rect_filled(
                rect,
                CornerRadius::same(radius::OVERLAY),
                state.fill(
                    color::CONTROL_FILL.get(),
                    color::CONTROL_FILL_HOVER.get(),
                    color::CONTROL_FILL_PRESSED.get(),
                ),
            ),
            Shape::rect_stroke(
                rect,
                CornerRadius::same(radius::OVERLAY),
                Stroke::new(size::STROKE, color::STROKE_DEFAULT.get()),
                StrokeKind::Inside,
            ),
            Shape::galley(
                Align2([Align::Center, Align::Min])
                    .anchor_size(
                        eframe::egui::pos2(
                            rect.center().x,
                            top + size::DEVICE_ACTION_ICON + space::M,
                        ),
                        galley.size(),
                    )
                    .min
                    - galley.rect.min.to_vec2(),
                galley,
                ink,
            ),
        ];
        for shape in &mut shapes {
            shape.transform(transform);
        }
        ui.painter().extend(shapes);
        let icon = Rect::from_center_size(
            eframe::egui::pos2(rect.center().x, top + size::DEVICE_ACTION_ICON / 2.0),
            vec2(size::DEVICE_ACTION_ICON, size::DEVICE_ACTION_ICON),
        );
        self.icon
            .paint(ui.painter(), transform.mul_rect(icon), color::ACCENT.get());
        focus_ring(ui.painter(), rect, radius::OVERLAY, state.focus);
        response
    }
}
