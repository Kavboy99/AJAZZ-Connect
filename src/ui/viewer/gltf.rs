//! The glTF 2.0 document schema, as far as `glb.rs` reads it.

use super::math::{Mat4, Vec3};
use serde::Deserialize;

pub(super) const TRIANGLES: u32 = 4;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Document {
    #[serde(default)]
    pub scene: usize,
    pub scenes: Vec<SceneNodes>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Mesh>,
    #[serde(default)]
    pub materials: Vec<GltfMaterial>,
    pub accessors: Vec<Accessor>,
    pub buffer_views: Vec<BufferView>,
    #[serde(default)]
    pub textures: Vec<Texture>,
    #[serde(default)]
    pub images: Vec<GltfImage>,
}

#[derive(Deserialize)]
pub(super) struct SceneNodes {
    pub nodes: Vec<usize>,
}

#[derive(Deserialize)]
pub(super) struct Node {
    pub name: Option<String>,
    pub mesh: Option<usize>,
    #[serde(default)]
    pub children: Vec<usize>,
    pub matrix: Option<Mat4>,
    pub translation: Option<Vec3>,
    pub rotation: Option<[f32; 4]>,
    pub scale: Option<Vec3>,
}

#[derive(Deserialize)]
pub(super) struct Mesh {
    pub primitives: Vec<Primitive>,
}

#[derive(Deserialize)]
pub(super) struct Primitive {
    pub attributes: Attributes,
    pub indices: Option<usize>,
    pub material: Option<usize>,
    #[serde(default = "triangles")]
    pub mode: u32,
}

pub(super) fn triangles() -> u32 {
    TRIANGLES
}

#[derive(Deserialize)]
pub(super) struct Attributes {
    #[serde(rename = "POSITION")]
    pub position: usize,
    #[serde(rename = "NORMAL")]
    pub normal: Option<usize>,
    #[serde(rename = "TEXCOORD_0")]
    pub uv: Option<usize>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct GltfMaterial {
    #[serde(default)]
    pub pbr_metallic_roughness: Pbr,
    #[serde(default)]
    pub alpha_mode: Option<String>,
    #[serde(default)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Pbr {
    #[serde(default = "white")]
    pub base_color_factor: [f32; 4],
    pub base_color_texture: Option<TextureRef>,
    #[serde(default = "one")]
    pub metallic_factor: f32,
    #[serde(default = "one")]
    pub roughness_factor: f32,
}

impl Default for Pbr {
    fn default() -> Self {
        Self {
            base_color_factor: white(),
            base_color_texture: None,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
        }
    }
}

pub(super) fn white() -> [f32; 4] {
    [1.0; 4]
}

pub(super) fn one() -> f32 {
    1.0
}

#[derive(Deserialize)]
pub(super) struct TextureRef {
    pub index: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Accessor {
    pub buffer_view: usize,
    #[serde(default)]
    pub byte_offset: usize,
    pub component_type: u32,
    pub count: usize,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BufferView {
    #[serde(default)]
    pub byte_offset: usize,
    pub byte_length: usize,
    pub byte_stride: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct Texture {
    pub source: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GltfImage {
    pub buffer_view: usize,
}
