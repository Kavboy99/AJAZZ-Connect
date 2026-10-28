use super::interaction::{Interaction, focus_ring};
use super::spinner;
use crate::ui::icons::Icon;
use crate::ui::theme::{Role, color, radius, size, space};
use eframe::egui::{
    Color32, CornerRadius, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, Widget,
    WidgetInfo, WidgetText, WidgetType, emath::TSTransform, vec2,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// One per screen or card: the main action.
    Primary,
    Standard,
    /// No surface at rest: navigation and icon buttons.
    Subtle,
}

pub struct Button<'a> {
    kind: Kind,
    text: &'a str,
    icon: Option<Icon>,
    enabled: bool,
    passive: bool,
    busy: bool,
    tooltip: Option<&'a str>,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            kind: Kind::Standard,
            text,
            icon: None,
            enabled: true,
            passive: false,
            busy: false,
            tooltip: None,
        }
    }

    /// A square icon-only button; `name` is its tooltip and accessible name.
    pub fn icon_only(icon: Icon, name: &'a str) -> Self {
        Self::new("").icon(icon).kind(Kind::Subtle).tooltip(name)
    }

    pub fn kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    pub fn primary(self) -> Self {
        self.kind(Kind::Primary)
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Keeps the visual state while a transitioning source page cannot accept input.
    pub fn passive(mut self, passive: bool) -> Self {
        self.passive = passive;
        self
    }

    /// Shows the busy indicator in place of the icon and ignores clicks.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    fn colors(&self, state: &Interaction, enabled: bool) -> (Color32, Color32, Option<Stroke>) {
        if !enabled && !self.busy {
            let fill = match self.kind {
                Kind::Subtle => Color32::TRANSPARENT,
                _ => color::CONTROL_FILL_DISABLED.get(),
            };
            return (fill, color::TEXT_DISABLED.get(), None);
        }
        let border = Some(Stroke::new(size::STROKE, color::STROKE_DEFAULT.get()));
        match self.kind {
            Kind::Primary => (
                state.fill(
                    color::ACCENT.get(),
                    color::ACCENT_HOVER.get(),
                    color::ACCENT_PRESSED.get(),
                ),
                color::TEXT_ON_ACCENT.get(),
                None,
            ),
            Kind::Standard => (
                state.fill(
                    color::CONTROL_FILL.get(),
                    color::CONTROL_FILL_HOVER.get(),
                    color::CONTROL_FILL_PRESSED.get(),
                ),
                color::TEXT_PRIMARY.get(),
                border,
            ),
            Kind::Subtle => (
                state.fill(
                    Color32::TRANSPARENT,
                    color::CONTROL_FILL_HOVER.get(),
                    color::CONTROL_FILL_PRESSED.get(),
                ),
                color::TEXT_PRIMARY.get(),
                None,
            ),
        }
    }
}

impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let visually_enabled = self.enabled && !self.busy && (ui.is_enabled() || self.passive);
        let enabled = self.enabled && !self.busy && ui.is_enabled() && !self.passive;
        let galley = (!self.text.is_empty()).then(|| {
            WidgetText::from(self.text).into_galley(
                ui,
                Some(eframe::egui::TextWrapMode::Extend),
                f32::INFINITY,
                Role::Body.font(),
            )
        });
        let icon_space = if self.icon.is_some() || self.busy {
            size::ICON_SMALL
        } else {
            0.0
        };
        let width = match &galley {
            Some(g) => {
                let gap = if icon_space > 0.0 { space::S } else { 0.0 };
                space::M * 2.0 + icon_space + gap + g.size().x
            }
            None => size::CONTROL_HEIGHT,
        };
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(width.round(), size::CONTROL_HEIGHT), sense);
        let label = self.tooltip.unwrap_or(self.text);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
        if let Some(tip) = self.tooltip {
            response = response.on_hover_text(tip);
        }
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let state = Interaction::of(ui, response.id, &response);
        let (fill, ink, stroke) = self.colors(&state, visually_enabled);
        let mut shapes = vec![Shape::rect_filled(
            rect,
            CornerRadius::same(radius::CONTROL),
            fill,
        )];
        if let Some(stroke) = stroke {
            shapes.push(Shape::rect_stroke(
                rect,
                CornerRadius::same(radius::CONTROL),
                stroke,
                StrokeKind::Inside,
            ));
        }
        let content_width = icon_space
            + galley.as_ref().map_or(0.0, |g| {
                g.size().x + if icon_space > 0.0 { space::S } else { 0.0 }
            });
        let mut x = rect.center().x - content_width / 2.0;
        let icon_rect = Rect::from_center_size(
            eframe::egui::pos2(x + icon_space / 2.0, rect.center().y),
            Vec2::splat(size::ICON_SMALL),
        );
        let painter = ui.painter();
        if let Some(g) = galley {
            x += if icon_space > 0.0 {
                icon_space + space::S
            } else {
                0.0
            };
            shapes.push(Shape::galley(
                eframe::egui::pos2(x, rect.center().y - g.size().y / 2.0),
                g,
                ink,
            ));
        }
        // Press feedback: the whole control scales around its center.
        let transform =
            TSTransform::new(rect.center().to_vec2() * (1.0 - state.scale), state.scale);
        for shape in &mut shapes {
            shape.transform(transform);
        }
        painter.extend(shapes);
        if self.busy {
            spinner::paint(ui, transform.mul_rect(icon_rect), ink);
        } else if let Some(icon) = self.icon {
            icon.paint(painter, transform.mul_rect(icon_rect), ink);
        }
        focus_ring(painter, rect, radius::CONTROL, state.focus);
        response
    }
}
