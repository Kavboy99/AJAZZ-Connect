//! Applies the tokens to egui's own widgets (scroll bars, text fields, tooltips) so nothing draws
//! with egui's default palette.

use super::mode;
use super::text::Role;
use super::tokens::{color, motion, radius, size, space};
use eframe::egui::{
    Context, CornerRadius, Margin, Shadow, Stroke, Style, TextStyle, Theme, Visuals, style,
};

fn widget(fill: eframe::egui::Color32, text: eframe::egui::Color32) -> style::WidgetVisuals {
    style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(size::STROKE, color::STROKE_DEFAULT.get()),
        corner_radius: CornerRadius::same(radius::CONTROL),
        fg_stroke: Stroke::new(size::STROKE, text),
        expansion: 0.0,
    }
}

fn visuals() -> Visuals {
    let mut v = if mode::is_dark() {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.handle_shape = eframe::egui::style::HandleShape::Circle;
    v.panel_fill = color::SURFACE_BASE.get();
    v.window_fill = color::SURFACE_OVERLAY.get();
    v.window_stroke = Stroke::new(size::STROKE, color::STROKE_DEFAULT.get());
    v.window_corner_radius = CornerRadius::same(radius::OVERLAY);
    v.menu_corner_radius = CornerRadius::same(radius::OVERLAY);
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.extreme_bg_color = color::SURFACE_SUNKEN.get();
    v.faint_bg_color = color::SURFACE_CARD.get();
    v.code_bg_color = color::SURFACE_SUNKEN.get();
    v.hyperlink_color = color::ACCENT.get();
    v.selection.bg_fill = color::ACCENT_SUBTLE.get();
    v.selection.stroke = Stroke::new(size::STROKE, color::ACCENT.get());
    v.text_cursor.stroke = Stroke::new(size::STROKE, color::TEXT_PRIMARY.get());
    // A blinking caret would repaint an idle window forever.
    v.text_cursor.blink = false;
    v.widgets.noninteractive = widget(color::SURFACE_CARD.get(), color::TEXT_PRIMARY.get());
    v.widgets.noninteractive.bg_stroke = Stroke::new(size::STROKE, color::STROKE_SUBTLE.get());
    v.widgets.inactive = widget(color::CONTROL_FILL.get(), color::TEXT_PRIMARY.get());
    v.widgets.hovered = widget(color::CONTROL_FILL_HOVER.get(), color::TEXT_PRIMARY.get());
    v.widgets.active = widget(color::CONTROL_FILL_PRESSED.get(), color::TEXT_PRIMARY.get());
    v.widgets.open = widget(color::CONTROL_FILL_PRESSED.get(), color::TEXT_PRIMARY.get());
    v.widgets.inactive.bg_stroke = Stroke::new(size::STROKE, color::STROKE_CONTROL.get());
    v.widgets.hovered.bg_stroke = Stroke::new(size::STROKE, color::STROKE_CONTROL.get());
    v.widgets.active.bg_stroke = Stroke::new(size::STROKE, color::ACCENT.get());
    v
}

pub fn apply(ctx: &Context) {
    ctx.set_theme(Theme::Dark);
    let mut style = Style {
        visuals: visuals(),
        ..Style::default()
    };
    style.text_styles = [
        (TextStyle::Small, Role::Caption.font()),
        (TextStyle::Body, Role::Body.font()),
        (TextStyle::Button, Role::Body.font()),
        (TextStyle::Monospace, Role::Mono.font()),
        (TextStyle::Heading, Role::Subtitle.font()),
    ]
    .into();
    style.spacing.item_spacing = eframe::egui::vec2(space::S, space::S);
    style.spacing.button_padding = eframe::egui::vec2(space::M, space::XS + 2.0);
    style.spacing.interact_size.y = size::CONTROL_HEIGHT;
    style.spacing.window_margin = Margin::same(space::L as i8);
    style.spacing.menu_margin = Margin::same(space::S as i8);
    style.spacing.scroll = style::ScrollStyle::floating();
    style.spacing.scroll.bar_width = space::S;
    style.interaction.tooltip_delay = motion::TOOLTIP_DELAY;
    // Motion is driven by springs in `ui::motion`; egui's own timed animations are off.
    style.animation_time = 0.0;
    ctx.set_style_of(Theme::Dark, style);
}
