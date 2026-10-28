//! Page composition. Device drill navigation expands the actual page surface from its tile.
use super::Shell;
use crate::ui::device_reveal::{self, Reveal};
use crate::ui::motion::{self, Frame};
use crate::ui::pages::{Model, Pages, Route};
use crate::ui::theme::{color, radius};
use crate::ui::widgets::mix;
use eframe::egui::{CornerRadius, Rect, Ui, UiBuilder, Vec2, emath::TSTransform};

impl Shell {
    pub(super) fn content(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        model: &mut Model<'_>,
        pages: &mut Pages,
    ) {
        if self.reveal.is_some() {
            self.device_content(ui, rect, model, pages);
            return;
        }
        device_reveal::clear(ui.ctx());
        self.transition.step(ui.ctx());
        let reduced = motion::reduced(ui.ctx());
        let mut navigate = None;
        if let Some((route, frame)) = self.transition.outgoing() {
            draw_page(ui, rect, frame, false, |ui| {
                pages.show(ui, route, model);
            });
        }
        let current = self.transition.current();
        draw_page(ui, rect, self.transition.incoming(reduced), true, |ui| {
            navigate = pages.show(ui, current, model);
        });
        if let Some((route, style)) = navigate {
            if current == Route::Devices
                && matches!(route, Route::Device(_) | Route::Preview(_))
                && let Some(reveal) = Reveal::new(ui.ctx(), route)
            {
                self.reveal = Some(reveal);
                self.transition.rename(route);
                motion::keep_alive(ui.ctx());
            } else {
                self.navigate(route, style);
            }
        }
    }

    fn device_content(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        model: &mut Model<'_>,
        pages: &mut Pages,
    ) {
        let Some(reveal) = &mut self.reveal else {
            return;
        };
        let surface = reveal.step(ui.ctx(), rect);
        let route = reveal.route();
        let p = reveal.progress();
        let moving = reveal.moving();
        let finished = reveal.finished();
        let still = Frame {
            opacity: 1.0,
            offset: Vec2::ZERO,
            scale: 1.0,
        };
        let mut navigate = None;
        if finished {
            device_reveal::clear(ui.ctx());
            self.reveal = None;
            draw_page(ui, rect, still, true, |ui| {
                navigate = pages.show(ui, Route::Devices, model);
            });
            if let Some((route, style)) = navigate {
                self.navigate(route, style);
            }
            return;
        }
        if moving {
            let list = Frame {
                opacity: device_reveal::list_opacity(p),
                ..still
            };
            draw_page(ui, rect, list, false, |ui| {
                pages.show(ui, Route::Devices, model);
            });
        }
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .id_salt(("page-layer", true)),
        );
        child.set_clip_rect(surface.intersect(rect));
        if motion::reduced(ui.ctx()) {
            child.multiply_opacity(p);
        }
        child.painter().rect_filled(
            surface,
            CornerRadius::same(radius::OVERLAY),
            mix(color::SURFACE_CARD.get(), color::SURFACE_BASE.get(), p)
                .gamma_multiply(device_reveal::surface_opacity(p)),
        );
        navigate = pages.show(&mut child, route, model);
        if let Some((route, style)) = navigate {
            self.navigate(route, style);
            motion::keep_alive(ui.ctx());
        }
    }
}

fn draw_page(ui: &mut Ui, rect: Rect, frame: Frame, interactive: bool, add: impl FnOnce(&mut Ui)) {
    if frame.opacity <= 0.0 {
        return;
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .id_salt(("page-layer", interactive)),
    );
    child.set_clip_rect(rect);
    if !interactive {
        // Blocking input must not introduce egui's abrupt disabled-state dimming.
        child.style_mut().visuals.disabled_alpha = 1.0;
        child.disable();
    }
    child.multiply_opacity(frame.opacity);
    let center = rect.center().to_vec2();
    let transform = TSTransform::new(center * (1.0 - frame.scale) + frame.offset, frame.scale);
    child.with_visual_transform(transform, add);
}
