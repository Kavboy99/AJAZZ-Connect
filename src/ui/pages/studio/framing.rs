//! Resolve the destination fit before animating: fitting a changing rectangle can reverse scale.
use crate::ui::motion::{self, Channel, tokens};
use eframe::egui::{Context, Id, Rect, Vec2};

pub(super) fn animate(
    ctx: &Context,
    id: Id,
    destination: Rect,
    aspect: f32,
    encoder: bool,
) -> Rect {
    let fitted = if encoder {
        destination.width()
    } else {
        destination.width().min(destination.height() * aspect)
    };
    let center = motion::animate_pos(
        ctx,
        id.with("frame-center"),
        destination.center(),
        tokens::WORKSPACE,
    );
    let side = motion::animate(
        ctx,
        id.with("frame-side"),
        fitted,
        tokens::WORKSPACE,
        Channel::Offset,
    );
    Rect::from_center_size(center, Vec2::splat(side))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{RawInput, pos2, vec2};

    fn sample(ctx: &Context, frame: usize, stage: Rect, encoder: bool) -> Rect {
        let mut square = Rect::NOTHING;
        let _ = ctx.run(
            RawInput {
                time: Some(frame as f64 / 60.0),
                predicted_dt: 1.0 / 60.0,
                ..Default::default()
            },
            |ctx| {
                motion::begin_frame(ctx, true);
                square = animate(ctx, Id::new("model"), stage, 1.65, encoder);
            },
        );
        square
    }

    #[test]
    fn editor_framing_never_grows_then_shrinks_or_jumps_on_inspector_entry() {
        for width in [720.0, 1040.0, 1600.0] {
            let ctx = Context::default();
            let mut frame = 0;
            let overview = Rect::from_min_size(pos2(72.0, 64.0), vec2(width - 420.0, 752.0));
            let mut previous = sample(&ctx, frame, overview, false);
            for (stage, encoder) in [
                (
                    Rect::from_min_size(overview.min, vec2(width - 96.0, 468.0)),
                    false,
                ),
                (
                    Rect::from_min_size(overview.min, vec2(width - 420.0, 430.0)),
                    false,
                ),
                (
                    Rect::from_min_size(overview.min, vec2(width - 96.0, 532.0)),
                    true,
                ),
                (
                    Rect::from_min_size(overview.min, vec2(width - 420.0, 532.0)),
                    true,
                ),
                (overview, false),
            ] {
                let start = previous;
                let side = if encoder {
                    stage.width()
                } else {
                    stage.width().min(stage.height() * 1.65)
                };
                let target = Rect::from_center_size(stage.center(), Vec2::splat(side));
                for tick in 0..120 {
                    frame += 1;
                    let current = sample(&ctx, frame, stage, encoder);
                    for (old, value, end) in [
                        (previous.width(), current.width(), target.width()),
                        (previous.center().x, current.center().x, target.center().x),
                        (previous.center().y, current.center().y, target.center().y),
                    ] {
                        assert!(
                            value >= old.min(end) - 0.1 && value <= old.max(end) + 0.1,
                            "framing reversed at width {width}, tick {tick}: {old} -> {value} -> {end}"
                        );
                    }
                    if tick == 0 {
                        assert!(
                            (current.width() - start.width()).abs()
                                <= (target.width() - start.width()).abs() * 0.1 + 0.1
                        );
                        assert!(
                            (current.center() - start.center()).length()
                                <= (target.center() - start.center()).length() * 0.1 + 0.1
                        );
                    }
                    previous = current;
                }
                assert!((previous.width() - target.width()).abs() < 0.1);
                assert!((previous.center() - target.center()).length() < 0.1);
            }
        }
    }
}
