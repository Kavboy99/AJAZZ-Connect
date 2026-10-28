//! 3D views of devices (`docs/DESIGN.md`, section 8.1). A model is drawn by the software
//! rasterizer into a texture only when its pose or size changes, at full resolution with
//! consistent antialiasing. An idle tile costs one textured rectangle and no computation.

mod catalog;
mod glb;
mod gltf;
pub(crate) mod gpu;
mod keycaps;
mod led;
mod light;
mod math;
pub(crate) mod native_effect;
mod preview;
mod raster;

pub use catalog::preload;
pub use preview::{Effect, Preview, binding};
pub use raster::Pose;

use catalog::Model;
use eframe::egui::{Color32, Context, Id, Rect, TextureHandle, TextureOptions, Ui, Vec2, pos2};

/// What the caller should draw in place of the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    Model,
    Loading,
    /// No model for this device: the caller draws its kind icon.
    Missing,
}

/// The texture of one view and what it was drawn for.
#[derive(Clone)]
struct View {
    texture: TextureHandle,
    key: Key,
}

#[derive(Clone, PartialEq)]
struct Key {
    pose: [i32; 11],
    side: usize,
    effects: Vec<[u8; 4]>,
}

impl Key {
    /// Poses closer than what a pixel can show reuse the previous image.
    fn of(pose: &Pose, side: usize, effects: &[preview::PartEffect]) -> Self {
        let q = |v: f32, step: f32| (v / step).round() as i32;
        Self {
            pose: [
                q(pose.yaw, 0.002),
                q(pose.pitch, 0.002),
                q(pose.lift, 0.004),
                q(pose.glow, 0.01),
                q(pose.glow_color[0], 0.01),
                q(pose.glow_color[1], 0.01),
                q(pose.glow_color[2], 0.01),
                q(pose.zoom, 0.002),
                q(pose.focus[0], 0.002),
                q(pose.focus[1], 0.002),
                q(pose.focus[2], 0.002),
            ],
            side,
            effects: effects.iter().map(|e| e.key()).collect(),
        }
    }
}

/// Draws model `visual` into the largest square inside `rect`, faded by `opacity`.
pub fn show(ui: &Ui, id: Id, rect: Rect, visual: &str, pose: Pose, opacity: f32) -> Shown {
    show_inner(ui, id, rect, visual, pose, opacity, None)
}

pub fn show_preview(
    ui: &Ui,
    id: Id,
    rect: Rect,
    visual: &str,
    pose: Pose,
    preview: &Preview,
) -> Shown {
    show_inner(ui, id, rect, visual, pose, 1.0, Some(preview))
}

fn show_inner(
    ui: &Ui,
    id: Id,
    rect: Rect,
    visual: &str,
    pose: Pose,
    opacity: f32,
    preview: Option<&Preview>,
) -> Shown {
    let ctx = ui.ctx();
    let scene = match catalog::model(ctx, visual) {
        Model::Ready(scene) => scene,
        Model::Loading => return Shown::Loading,
        Model::Missing => return Shown::Missing,
    };
    let square =
        Rect::from_center_size(rect.center(), Vec2::splat(rect.width().min(rect.height())));
    let side = (square.width() * ctx.pixels_per_point()).round() as usize;
    if side == 0 {
        return Shown::Model;
    }
    let effects = preview
        .and_then(|p| binding(Some(visual)).map(|b| b.effects(&scene, p)))
        .unwrap_or_default();
    let key = Key::of(&pose, side, &effects);
    if let Some(result) = gpu::draw(ctx, id, visual, &scene, &pose, &effects, key.clone()) {
        if let Ok((texture, scale)) = result {
            ui.painter().image(
                texture,
                square,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(scale, scale)),
                Color32::WHITE.gamma_multiply(opacity),
            );
            return Shown::Model;
        }
        return Shown::Missing;
    }
    let view = ctx.data(|d| d.get_temp::<View>(id));
    let texture = match view {
        Some(view) if view.key == key => view.texture,
        previous => store(
            ctx,
            id,
            previous.map(|v| v.texture),
            &scene,
            &pose,
            key,
            &effects,
        ),
    };
    // A white tint leaves the rendered colors as they are; only the opacity changes.
    let tint = Color32::WHITE.gamma_multiply(opacity);
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    ui.painter().image(texture.id(), square, uv, tint);
    Shown::Model
}

fn store(
    ctx: &Context,
    id: Id,
    texture: Option<TextureHandle>,
    scene: &glb::Scene,
    pose: &Pose,
    key: Key,
    effects: &[preview::PartEffect],
) -> TextureHandle {
    let image = raster::render(scene, pose, key.side, effects);
    let texture = match texture {
        Some(mut texture) => {
            texture.set(image, TextureOptions::LINEAR);
            texture
        }
        None => ctx.load_texture(
            format!("device-model-{id:?}"),
            image,
            TextureOptions::LINEAR,
        ),
    };
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            View {
                texture: texture.clone(),
                key,
            },
        )
    });
    texture
}
