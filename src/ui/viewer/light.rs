//! The lighting of every device model: a warm key light from the upper left, a sky/ground
//! hemisphere fill (as in the reference scene of the model) and a rim light in the glow color
//! that grows on hover. Computed per vertex in linear color; the camera looks along -Z.

use super::glb::Material;
use super::math::{Vec3, dot, normalize};

const KEY_INTENSITY: f32 = 1.15;
const KEY_COLOR: Vec3 = [1.0, 0.976, 0.93];
const SKY: Vec3 = [1.0, 1.0, 1.0];
const GROUND: Vec3 = [0.47, 0.48, 0.42];
const HEMISPHERE: f32 = 0.62;
/// Textures with baked shading keep most of it; light only adds a little relief.
const BAKED_SHADING: f32 = 0.82;
const VIEW: Vec3 = [0.0, 0.0, 1.0];

/// What a vertex contributes: `color = texel * multiply + add`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Lit {
    pub emission: Vec3,
    pub glow: Vec3,
    pub multiply: Vec3,
    pub add: Vec3,
}

/// Per-material constants, computed once per batch instead of once per vertex.
pub struct Surface {
    color: Vec3,
    unlit: bool,
    shininess: f32,
    /// Highlight strength and tint.
    specular: Vec3,
    diffuse_share: f32,
}

pub struct Rig {
    key: Vec3,
    half: Vec3,
    rim: Vec3,
}

impl Rig {
    /// `rim` is the glow color already scaled by its strength.
    pub fn new(rim: Vec3) -> Self {
        let key = normalize([-0.45, 0.85, 0.55]);
        let half = normalize([key[0] + VIEW[0], key[1] + VIEW[1], key[2] + VIEW[2]]);
        Self { key, half, rim }
    }

    pub fn surface(&self, m: &Material) -> Surface {
        let strength = if m.unlit { 0.12 } else { 1.0 } * (1.0 - m.roughness * 0.7) * 4.0;
        Surface {
            color: [m.color[0], m.color[1], m.color[2]],
            unlit: m.unlit,
            // Blinn-Phong with a shininess that follows roughness.
            shininess: (2.0 / (m.roughness * m.roughness).max(0.02) - 1.0).clamp(4.0, 256.0),
            specular: std::array::from_fn(|c| (0.04 + (m.color[c] - 0.04) * m.metallic) * strength),
            diffuse_share: 1.0 - m.metallic * 0.6,
        }
    }

    pub fn shade(&self, normal: Vec3, s: &Surface) -> Lit {
        let diffuse = dot(normal, self.key).max(0.0) * KEY_INTENSITY;
        let up = 0.5 + 0.5 * normal[1];
        let highlight = dot(normal, self.half).max(0.0).powf(s.shininess);
        let rim = (1.0 - dot(normal, VIEW).max(0.0)).powi(3);
        let multiply = std::array::from_fn(|c| {
            let ambient = (GROUND[c] + (SKY[c] - GROUND[c]) * up) * HEMISPHERE;
            let light = if s.unlit {
                BAKED_SHADING + (1.0 - BAKED_SHADING) * (diffuse + ambient)
            } else {
                (ambient + diffuse * KEY_COLOR[c]) * s.diffuse_share
            };
            s.color[c] * light
        });
        let add = std::array::from_fn(|c| highlight * s.specular[c] + self.rim[c] * rim);
        Lit {
            multiply,
            add,
            emission: [0.0; 3],
            glow: [0.0; 3],
        }
    }
}

/// sRGB transfer, both ways.
pub fn to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
