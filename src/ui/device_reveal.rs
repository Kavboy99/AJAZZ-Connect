//! Shared geometry for device navigation. The shell owns one surface and one model throughout.
use super::motion::{self, Channel, Spring, tokens};
use super::pages::Route;
use eframe::egui::{Context, Id, Rect};

pub fn tile_id(route: Route) -> Id {
    match route {
        Route::Device(slot) => Id::new(("device-tile", slot)),
        Route::Preview(index) => Id::new(("catalog-tile", index)),
        _ => Id::NULL,
    }
}

pub fn remember(ctx: &Context, route: Route, rect: Rect) {
    ctx.data_mut(|d| d.insert_temp(tile_id(route).with("origin"), rect));
}

#[derive(Clone, Copy)]
struct Geometry {
    id: Id,
    origin: Rect,
    progress: f32,
    model_progress: f32,
    moving: bool,
}

pub fn hidden(ctx: &Context, id: Id) -> bool {
    ctx.data(|d| d.get_temp::<Geometry>(Id::new("device-reveal")))
        .is_some_and(|g| g.id == id)
}

pub fn stage(ctx: &Context, target: Rect) -> Rect {
    ctx.data(|d| d.get_temp::<Geometry>(Id::new("device-reveal")))
        .map_or(target, |g| {
            interpolate(
                super::widgets::tile_stage(g.origin),
                target,
                g.model_progress,
            )
        })
}

pub fn opacity(ctx: &Context) -> f32 {
    ctx.data(|d| d.get_temp::<Geometry>(Id::new("device-reveal")))
        .map_or(1.0, |g| g.progress)
}

pub fn moving(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp::<Geometry>(Id::new("device-reveal")))
        .is_some_and(|g| g.moving)
}

pub fn content_opacity(ctx: &Context) -> f32 {
    ((opacity(ctx) - 0.55) / 0.45).clamp(0.0, 1.0)
}

pub fn list_opacity(progress: f32) -> f32 {
    1.0 - crate::ui::theme::motion_tokens::DEVICE_LIST_DIM * progress
}

pub fn surface_opacity(progress: f32) -> f32 {
    let t = (progress / crate::ui::theme::motion_tokens::DEVICE_TILE_HANDOFF).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn clear(ctx: &Context) {
    ctx.data_mut(|d| d.remove::<Geometry>(Id::new("device-reveal")));
}

fn interpolate(from: Rect, to: Rect, progress: f32) -> Rect {
    Rect::from_min_max(
        from.min.lerp(to.min, progress),
        from.max.lerp(to.max, progress),
    )
}

pub struct Reveal {
    route: Route,
    origin: Rect,
    progress: Spring,
    model_progress: Spring,
    closing: bool,
}

impl Reveal {
    pub fn new(ctx: &Context, route: Route) -> Option<Self> {
        let origin = ctx.data(|d| d.get_temp::<Rect>(tile_id(route).with("origin")))?;
        let mut progress = Spring::at(0.0);
        progress.set_target(1.0);
        Some(Self {
            route,
            origin,
            progress,
            model_progress: progress,
            closing: false,
        })
    }

    pub fn route(&self) -> Route {
        self.route
    }

    pub fn navigate(&mut self, route: Route) -> bool {
        if route == Route::Devices || route == self.route {
            self.closing = route == Route::Devices;
            self.progress
                .set_target(if self.closing { 0.0 } else { 1.0 });
            self.model_progress
                .set_target(if self.closing { 0.0 } else { 1.0 });
            return true;
        }
        false
    }

    pub fn step(&mut self, ctx: &Context, target: Rect) -> Rect {
        // The list is laid out again during return, so resized windows land on the current tile.
        if self.closing
            && let Some(origin) =
                ctx.data(|d| d.get_temp::<Rect>(tile_id(self.route).with("origin")))
        {
            self.origin = origin;
        }
        motion::step(
            ctx,
            &mut self.progress,
            tokens::PAGE,
            if motion::reduced(ctx) {
                Channel::Fade
            } else {
                Channel::Scale
            },
        );
        motion::step(
            ctx,
            &mut self.model_progress,
            if self.closing {
                tokens::PAGE
            } else {
                tokens::DEVICE_REVEAL
            },
            Channel::Scale,
        );
        let p = self.progress.value().clamp(0.0, 1.0);
        // Reduced motion retains only the short opacity change, without spatial travel.
        let reduced = motion::reduced(ctx);
        let geometry = if reduced { 1.0 } else { p };
        let model_geometry = if reduced {
            1.0
        } else {
            self.model_progress.value()
        };
        ctx.data_mut(|d| {
            d.insert_temp(
                Id::new("device-reveal"),
                Geometry {
                    id: tile_id(self.route),
                    origin: self.origin,
                    progress: geometry,
                    model_progress: model_geometry,
                    moving: self.moving(),
                },
            )
        });
        interpolate(self.origin, target, geometry)
    }

    pub fn progress(&self) -> f32 {
        self.progress.value().clamp(0.0, 1.0)
    }
    pub fn finished(&self) -> bool {
        self.closing && !self.moving()
    }
    pub fn moving(&self) -> bool {
        !self.progress.at_rest(0.0005) || !self.model_progress.at_rest(0.0005)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Pos2, vec2};

    #[test]
    fn reversing_keeps_position_and_velocity_and_reaches_the_source() {
        let ctx = Context::default();
        let source = Rect::from_min_size(Pos2::ZERO, vec2(224.0, 224.0));
        remember(&ctx, Route::Device(0), source);
        let mut reveal = Reveal::new(&ctx, Route::Device(0)).unwrap();
        reveal.progress.step(tokens::PAGE, 0.08);
        let before = reveal.progress;
        assert!(reveal.navigate(Route::Devices));
        let mut expected = before;
        expected.set_target(0.0);
        assert_eq!(reveal.progress, expected);
        for _ in 0..120 {
            reveal.progress.step(tokens::PAGE, 1.0 / 60.0);
        }
        assert!(reveal.finished());
        assert!(!reveal.navigate(Route::Settings));
    }

    #[test]
    fn geometry_hands_off_exactly_and_missing_sources_do_not_morph() {
        let ctx = Context::default();
        assert!(Reveal::new(&ctx, Route::Device(1)).is_none());
        let source = Rect::from_min_size(Pos2::ZERO, vec2(224.0, 224.0));
        let target = Rect::from_min_size(Pos2::ZERO, vec2(1000.0, 800.0));
        assert_eq!(interpolate(source, target, 0.0), source);
        assert_eq!(interpolate(source, target, 1.0), target);
        remember(&ctx, Route::Device(1), source);
        let mut reveal = Reveal::new(&ctx, Route::Device(1)).unwrap();
        motion::begin_frame(&ctx, false);
        assert_eq!(reveal.step(&ctx, target), target);
        assert!(hidden(&ctx, tile_id(Route::Device(1))));
        assert!(!hidden(&ctx, tile_id(Route::Device(2))));
        clear(&ctx);
        assert!(!hidden(&ctx, tile_id(Route::Device(1))));
    }

    #[test]
    fn tile_handoff_reaches_normal_brightness_and_exposes_labels_before_completion() {
        assert_eq!(list_opacity(0.0), 1.0);
        assert_eq!(surface_opacity(0.0), 0.0);
        assert!(list_opacity(0.0005) > 0.999);
        assert!(surface_opacity(0.0005) < 0.0001);
        assert!(surface_opacity(0.1) > 0.0 && surface_opacity(0.1) < 1.0);
        assert_eq!(surface_opacity(1.0), 1.0);
    }

    #[test]
    fn only_the_model_overshoots_and_reversing_preserves_both_springs() {
        let ctx = Context::default();
        let origin = Rect::from_min_size(Pos2::ZERO, vec2(224.0, 224.0));
        remember(&ctx, Route::Device(0), origin);
        let mut reveal = Reveal::new(&ctx, Route::Device(0)).unwrap();
        let mut peak = 1.0_f32;
        for _ in 0..100 {
            reveal.progress.step(tokens::PAGE, 1.0 / 120.0);
            reveal
                .model_progress
                .step(tokens::DEVICE_REVEAL, 1.0 / 120.0);
            peak = peak.max(reveal.model_progress.value());
        }
        assert!((1.003..1.009).contains(&peak));
        let mut expected = reveal.model_progress;
        expected.set_target(0.0);
        reveal.navigate(Route::Devices);
        assert_eq!(reveal.model_progress, expected);
        for _ in 0..140 {
            reveal.progress.step(tokens::PAGE, 1.0 / 60.0);
            reveal.model_progress.step(tokens::PAGE, 1.0 / 60.0);
        }
        assert!(reveal.finished());
    }
}
