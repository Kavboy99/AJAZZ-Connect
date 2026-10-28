//! 3D views of devices (`docs/DESIGN.md`, section 8.1). A model is drawn by the software
//! rasterizer into a texture only when its pose or size changes: a cheap draft while it moves,
//! one fine frame when it stops. An idle tile costs one textured rectangle and no computation.

mod catalog;
mod glb;
mod gltf;
mod light;
mod math;
mod raster;

pub use catalog::preload;
pub use raster::Pose;

use catalog::Model;
use eframe::egui::{Color32, Context, Id, Rect, TextureHandle, TextureOptions, Ui, Vec2, pos2};
use raster::Quality;

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
    quality: Quality,
}

#[derive(Clone, Copy, PartialEq)]
struct Key {
    pose: [i32; 7],
    side: usize,
}

impl Key {
    /// Poses closer than what a pixel can show reuse the previous image.
    fn of(pose: &Pose, side: usize) -> Self {
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
            ],
            side,
        }
    }
}

/// Draws model `visual` into the largest square inside `rect`, faded by `opacity`.
pub fn show(ui: &Ui, id: Id, rect: Rect, visual: &str, pose: Pose, opacity: f32) -> Shown {
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
    let key = Key::of(&pose, side);
    let view = ctx.data(|d| d.get_temp::<View>(id));
    let texture = match view {
        Some(view) if view.key == key && view.quality == Quality::Fine => view.texture,
        // A first view and a pose that has just stopped get the fine frame.
        Some(view) if view.key == key => store(
            ctx,
            id,
            Some(view.texture),
            &scene,
            &pose,
            key,
            Quality::Fine,
        ),
        None => store(ctx, id, None, &scene, &pose, key, Quality::Fine),
        Some(view) => {
            // Motion springs stop asking for frames once they rest; one more frame upgrades the draft.
            crate::ui::motion::keep_alive(ctx);
            store(
                ctx,
                id,
                Some(view.texture),
                &scene,
                &pose,
                key,
                Quality::Draft,
            )
        }
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
    quality: Quality,
) -> TextureHandle {
    let image = raster::render(scene, pose, key.side, quality);
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
                quality,
            },
        )
    });
    texture
}
