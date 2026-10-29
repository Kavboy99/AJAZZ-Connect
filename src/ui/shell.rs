//! The window: navigation pane on the left (the only place the acrylic backdrop shows through), an
//! opaque content layer on the right, and directional transitions between pages.

use super::device_reveal::Reveal;
use super::icons::Icon;
use super::motion::{self, Channel, Style, Transition, tokens};
mod content;
use super::pages::{Model, Pages, Route, set_settling};
use super::strings as s;
use super::theme::{Role, breakpoint, color, radius, size, space};
use super::widgets::{Button, Interaction, focus_ring, mix};
use crate::app::{Settings, Theme};
use eframe::egui::{
    self, CentralPanel, Color32, Context, CornerRadius, Id, Painter, Rect, Sense, SidePanel,
    Stroke, StrokeKind, Ui, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};
use std::f32::consts::FRAC_PI_2;

const INDICATOR_HEIGHT: f32 = 16.0;

pub struct Shell {
    expanded: bool,
    device_expanded: bool,
    device_route: Option<Route>,
    transition: Transition<Route>,
    reveal: Option<Reveal>,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            expanded: true,
            device_expanded: false,
            device_route: None,
            reveal: None,
            transition: Transition::new(Route::Devices),
        }
    }
}

/// The navigation item a route belongs to.
fn section(route: Route) -> Route {
    match route {
        Route::Device(_) | Route::Preview(_) => Route::Devices,
        other => other,
    }
}

impl Shell {
    pub fn route(&self) -> Route {
        self.transition.current()
    }
    pub fn navigate(&mut self, route: Route, style: Style) {
        if let Some(reveal) = &mut self.reveal
            && reveal.navigate(route)
        {
            self.transition.rename(route);
            return;
        }
        self.reveal = None;
        self.transition.navigate(route, style);
    }

    /// After a new search: an open device page follows its device to its new slot, or goes back to
    /// the list when the device is gone. `moved` maps old slots to new ones.
    pub fn follow(&mut self, moved: &[Option<usize>]) {
        if let Route::Device(slot) = self.transition.current() {
            self.reveal = None;
            match moved.get(slot).copied().flatten() {
                Some(new) => self.transition.rename(Route::Device(new)),
                None => self.transition.navigate(Route::Devices, Style::DrillOut),
            }
        }
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        backdrop: bool,
        model: &mut Model<'_>,
        pages: &mut Pages,
    ) {
        let wide = ctx.content_rect().width() >= breakpoint::NAV_EXPANDED;
        let in_device = matches!(self.route(), Route::Device(_) | Route::Preview(_));
        let device_route = in_device.then_some(self.route());
        if self.device_route != device_route {
            self.device_expanded = false;
            self.device_route = device_route;
        }
        let expanded = if in_device {
            self.device_expanded
        } else {
            self.expanded
        };
        let target = if expanded && wide {
            size::NAV_EXPANDED
        } else {
            size::NAV_COLLAPSED
        };
        let width = motion::animate(
            ctx,
            Id::new("nav-width"),
            target,
            tokens::STANDARD,
            Channel::Offset,
        );
        let fill = if backdrop {
            Color32::TRANSPARENT
        } else {
            color::SURFACE_BASE.get()
        };
        SidePanel::left("navigation")
            .exact_width(width.round())
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(fill))
            .show(ctx, |ui| {
                let reveal = ((width - size::NAV_COLLAPSED)
                    / (size::NAV_EXPANDED - size::NAV_COLLAPSED))
                    .clamp(0.0, 1.0);
                self.navigation(ui, reveal, model.settings, super::theme::mode::blend())
            });
        // Pages choose their columns for the width they will have when the pane settles, so a
        // collapsing pane never flips the layout halfway through.
        set_settling(ctx, width - target);

        CentralPanel::default()
            .frame(egui::Frame::new())
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let layer = CornerRadius {
                    nw: radius::OVERLAY,
                    ..CornerRadius::ZERO
                };
                ui.painter()
                    .rect_filled(rect, layer, color::SURFACE_BASE.get());
                ui.painter().rect_stroke(
                    rect,
                    layer,
                    Stroke::new(size::STROKE, color::STROKE_SUBTLE.get()),
                    StrokeKind::Inside,
                );
                self.content(ui, rect, model, pages);
            });
    }

    fn navigation(&mut self, ui: &mut Ui, reveal: f32, settings: &mut Settings, theme_blend: f32) {
        ui.add_space(space::S);
        ui.horizontal(|ui| {
            ui.add_space(space::S);
            if ui
                .add(Button::icon_only(Icon::Menu, s::NAV_TOGGLE))
                .clicked()
            {
                if self.device_route.is_some() {
                    self.device_expanded = !self.device_expanded;
                } else {
                    self.expanded = !self.expanded;
                }
            }
            if reveal > 0.0 {
                // Painted, not laid out: the name fades and slides under the pane edge.
                let at = ui.cursor().min + vec2(space::XS, 0.0);
                let icon = Rect::from_min_size(
                    pos2(at.x, at.y + (size::CONTROL_HEIGHT - size::ICON_SMALL) / 2.0),
                    Vec2::splat(size::ICON_SMALL),
                );
                Icon::App.paint(
                    ui.painter(),
                    icon,
                    color::TEXT_PRIMARY.get().gamma_multiply(reveal),
                );
                let ink = color::TEXT_SECONDARY.get().gamma_multiply(reveal);
                let name = ui.painter().layout_no_wrap(
                    crate::app::identity::PRODUCT_NAME.to_owned(),
                    Role::Caption.font(),
                    ink,
                );
                let y = at.y + (size::CONTROL_HEIGHT - name.size().y) / 2.0;
                ui.painter()
                    .galley(pos2(icon.right() + space::S, y), name, ink);
            }
        });
        ui.add_space(space::S);
        let current = section(self.transition.current());
        let mut selected = None;
        let mut clicked = None;
        for (route, icon, text) in [
            (Route::Devices, Icon::Devices, s::NAV_DEVICES),
            (Route::Diagnostics, Icon::Diagnostics, s::NAV_DIAGNOSTICS),
        ] {
            let (rect, hit) = nav_item(ui, text, route == current, reveal, |p, r| {
                icon.paint(p, r, color::TEXT_PRIMARY.get())
            });
            if route == current {
                selected = Some(rect);
            }
            if hit {
                clicked = Some(route);
            }
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(space::S);
            let route = Route::Settings;
            let (rect, hit) = nav_item(ui, s::NAV_SETTINGS, route == current, reveal, |p, r| {
                Icon::Settings.paint(p, r, color::TEXT_PRIMARY.get())
            });
            if route == current {
                selected = Some(rect);
            }
            if hit {
                clicked = Some(route);
            }
            if theme_item(ui, settings.theme, reveal, theme_blend) {
                settings.theme = settings.theme.toggled();
                // The switch applies now; a failed save only loses it on the next launch.
                let _ = settings.save();
            }
        });
        // The indicator travels from the old item to the new one.
        if let Some(rect) = selected {
            let y = motion::animate(
                ui.ctx(),
                Id::new("nav-indicator"),
                rect.center().y,
                tokens::STANDARD,
                Channel::Offset,
            );
            let bar = Rect::from_center_size(
                pos2(rect.left() + size::NAV_INDICATOR / 2.0, y),
                vec2(size::NAV_INDICATOR, INDICATOR_HEIGHT),
            );
            ui.painter()
                .rect_filled(bar, CornerRadius::ZERO, color::ACCENT.get());
        }
        if let Some(route) = clicked
            && (route != current || self.reveal.is_some())
        {
            self.navigate(route, Style::Entrance);
        }
    }
}

/// A 40 px navigation row. Returns its rectangle and whether it was clicked.
fn nav_item(
    ui: &mut Ui,
    text: &str,
    selected: bool,
    reveal: f32,
    icon: impl FnOnce(&Painter, Rect),
) -> (Rect, bool) {
    let width = ui.available_width() - space::S;
    let (outer, mut response) =
        ui.allocate_exact_size(vec2(ui.available_width(), size::NAV_ROW), Sense::click());
    let rect = Rect::from_center_size(outer.center(), vec2(width, size::NAV_ROW - space::XS));
    response
        .widget_info(|| WidgetInfo::selected(WidgetType::SelectableLabel, true, selected, text));
    if reveal < 0.5 {
        response = response.on_hover_text(text);
    }
    let state = Interaction::of(ui, response.id, &response);
    let base = motion::animate_color(
        ui.ctx(),
        response.id.with("selected"),
        if selected {
            color::ACCENT_SUBTLE.get()
        } else {
            Color32::TRANSPARENT
        },
        tokens::STANDARD,
    );
    let fill = mix(
        base,
        if selected {
            color::ACCENT_SUBTLE.get()
        } else {
            color::CONTROL_FILL_HOVER.get()
        },
        state.hover,
    );
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same(radius::CONTROL),
        state.fill(fill, fill, color::CONTROL_FILL_PRESSED.get()),
    );
    let icon_rect = Rect::from_center_size(
        pos2(outer.left() + size::NAV_COLLAPSED / 2.0, rect.center().y),
        Vec2::splat(size::ICON),
    );
    icon(painter, icon_rect);
    if reveal > 0.0 {
        let ink = color::TEXT_PRIMARY.get().gamma_multiply(reveal);
        let galley = painter.layout_no_wrap(text.to_owned(), Role::Body.font(), ink);
        painter.galley(
            pos2(
                outer.left() + size::NAV_COLLAPSED,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            ink,
        );
    }
    focus_ring(painter, rect, radius::CONTROL, state.focus);
    (rect, response.clicked())
}

/// The theme switch: sun and moon cross-fade and turn with the theme blend itself.
fn theme_item(ui: &mut Ui, theme: Theme, reveal: f32, t: f32) -> bool {
    let text = match theme {
        Theme::Dark => s::THEME_TO_LIGHT,
        Theme::Light => s::THEME_TO_DARK,
    };
    let (_, clicked) = nav_item(ui, text, false, reveal, |painter, rect| {
        let ink = color::TEXT_PRIMARY.get();
        Icon::Sun.paint_rotated(painter, rect, ink.gamma_multiply(1.0 - t), t * FRAC_PI_2);
        Icon::Moon.paint_rotated(painter, rect, ink.gamma_multiply(t), (t - 1.0) * FRAC_PI_2);
    });
    clicked
}
