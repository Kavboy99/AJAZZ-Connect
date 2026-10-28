//! Physical motion (`docs/DESIGN.md`, section 9). Every animated value is a spring moving toward
//! a target. This module is the single place that keeps the window repainting: while any spring
//! moves it asks for the next frame, and once all are at rest nothing is drawn until an event.

mod scroll;
mod spring;
pub mod tokens;
mod transition;

pub use scroll::KineticScroll;
pub use spring::{Spring, SpringSpec};
pub use transition::{Frame, Style, Transition};

use eframe::egui::{Color32, Context, Id, Pos2, pos2};
use std::collections::HashMap;

/// A stalled frame advances motion by at most this much.
const MAX_DT: f32 = 1.0 / 30.0;

/// What a value means decides how it behaves when Windows animations are off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// Pixels of displacement. Dropped when animations are off.
    Offset,
    /// Dimensionless scale. Dropped when animations are off.
    Scale,
    /// Opacity and color in 0..1. Kept, but shortened, when animations are off.
    Fade,
}

impl Channel {
    fn epsilon(self) -> f32 {
        match self {
            Channel::Offset => 0.1,
            Channel::Scale => 0.0005,
            Channel::Fade => 0.005,
        }
    }
}

#[derive(Clone)]
struct Track {
    springs: [Spring; 4],
    frame: u64,
}

#[derive(Clone, Default)]
struct Store {
    tracks: HashMap<Id, Track>,
    frame: u64,
    dt: f32,
    reduced: bool,
    suspended: bool,
}

impl Store {
    /// The spec to step with, or `None` to jump to the target.
    fn spec(&self, spec: SpringSpec, channel: Channel) -> Option<SpringSpec> {
        match (self.suspended, self.reduced, channel) {
            (true, _, _) => None,
            (false, true, Channel::Fade) => Some(tokens::REDUCED),
            (false, true, _) => None,
            (false, false, _) => Some(spec),
        }
    }
}

fn store_id() -> Id {
    Id::new("ui::motion")
}

/// Called once at the top of every frame.
pub fn begin_frame(ctx: &Context, animations_enabled: bool) {
    let frame = ctx.cumulative_frame_nr();
    let (dt, minimized) = ctx.input(|i| {
        (
            i.stable_dt.min(MAX_DT),
            i.viewport().minimized == Some(true),
        )
    });
    ctx.data_mut(|d| {
        let store = d.get_temp_mut_or_default::<Store>(store_id());
        store.frame = frame;
        store.dt = dt;
        store.reduced = !animations_enabled;
        // A minimized window computes nothing; on return values jump to their targets.
        store.suspended = minimized;
        store.tracks.retain(|_, t| t.frame + 2 >= frame);
    });
}

fn animate_n<const N: usize>(
    ctx: &Context,
    id: Id,
    start: Option<[f32; N]>,
    target: [f32; N],
    spec: SpringSpec,
    channel: Channel,
) -> [f32; N] {
    let (values, moving, suspended) = ctx.data_mut(|d| {
        let store = d.get_temp_mut_or_default::<Store>(store_id());
        let frame = store.frame;
        let step = store.spec(spec, channel);
        let dt = store.dt;
        let suspended = store.suspended;
        let track = store.tracks.entry(id).or_insert_with(|| {
            let mut springs = [Spring::default(); 4];
            for (i, s) in springs.iter_mut().enumerate().take(N) {
                *s = Spring::at(start.map_or(target[i], |v| v[i]));
            }
            Track { springs, frame }
        });
        // Not drawn last frame: it was off screen, so it simply appears at its target.
        let stale = track.frame + 1 < frame;
        let fresh = track.frame != frame;
        let mut values = [0.0; N];
        let mut moving = false;
        for i in 0..N {
            let s = &mut track.springs[i];
            s.set_target(target[i]);
            match step {
                Some(spec) if !stale => {
                    if fresh {
                        s.step(spec, dt);
                    }
                }
                _ => s.settle(),
            }
            if s.at_rest(channel.epsilon()) {
                s.settle();
            } else {
                moving = true;
            }
            values[i] = s.value();
        }
        track.frame = frame;
        (values, moving, suspended)
    });
    if moving && !suspended {
        ctx.request_repaint();
    }
    values
}

pub fn animate(ctx: &Context, id: Id, target: f32, spec: SpringSpec, channel: Channel) -> f32 {
    animate_n(ctx, id, None, [target], spec, channel)[0]
}

/// Like `animate`, but a value seen for the first time starts at `from`.
pub fn animate_from(
    ctx: &Context,
    id: Id,
    from: f32,
    target: f32,
    spec: SpringSpec,
    channel: Channel,
) -> f32 {
    animate_n(ctx, id, Some([from]), [target], spec, channel)[0]
}

pub fn animate_pos(ctx: &Context, id: Id, target: Pos2, spec: SpringSpec) -> Pos2 {
    let [x, y] = animate_n(ctx, id, None, [target.x, target.y], spec, Channel::Offset);
    pos2(x, y)
}

/// Colors flow channel by channel instead of jumping.
pub fn animate_color(ctx: &Context, id: Id, target: Color32, spec: SpringSpec) -> Color32 {
    let t = target.to_array().map(|c| c as f32 / 255.0);
    let v = animate_n(ctx, id, None, t, spec, Channel::Fade);
    let c = v.map(|x| (x.clamp(0.0, 1.0) * 255.0).round() as u8);
    Color32::from_rgba_premultiplied(c[0], c[1], c[2], c[3])
}

/// Steps a spring the caller owns (transitions) once per frame under the same rules.
pub fn step(ctx: &Context, spring: &mut Spring, spec: SpringSpec, channel: Channel) {
    let (dt, step, suspended) = ctx.data_mut(|d| {
        let store = d.get_temp_mut_or_default::<Store>(store_id());
        (store.dt, store.spec(spec, channel), store.suspended)
    });
    match step {
        Some(spec) => spring.step(spec, dt),
        None => spring.settle(),
    }
    if spring.at_rest(channel.epsilon()) {
        spring.settle();
    } else if !suspended {
        ctx.request_repaint();
    }
}

/// For motion that is not a spring (busy indicator, kinetic scroll): keeps frames coming while it
/// runs, unless the window is minimized.
pub fn keep_alive(ctx: &Context) {
    let suspended = ctx.data_mut(|d| d.get_temp_mut_or_default::<Store>(store_id()).suspended);
    if !suspended {
        ctx.request_repaint();
    }
}

pub fn dt(ctx: &Context) -> f32 {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Store>(store_id()).dt)
}

/// Windows animations are off: drop displacement and scaling.
pub fn reduced(ctx: &Context) -> bool {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Store>(store_id()).reduced)
}
