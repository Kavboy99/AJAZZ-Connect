//! A reader for the subset of binary glTF 2.0 the built-in models use: triangle lists with
//! positions, normals and texture coordinates, node matrices or TRS, base-color factors and PNG
//! textures, `KHR_materials_unlit` and blended materials. Everything else is refused.

use super::gltf::{Document, GltfImage, GltfMaterial, TRIANGLES};
use super::math::{self, Mat4, Vec3};

const MAGIC: &[u8; 4] = b"glTF";
const CHUNK_JSON: u32 = 0x4e4f_534a;
const CHUNK_BIN: u32 = 0x004e_4942;
const FLOAT: u32 = 5126;
/// Textures are halved until no side exceeds this (`docs/ARCHITECTURE.md`, section 5).
const MAX_TEXTURE_SIDE: u32 = 4096;

/// One vertex as the GPU reads it: position, normal, texture coordinate.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Vertex {
    pub part: usize,
    pub bounce: [(usize, f32); 4],
    pub legend: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
}

#[derive(Clone, Debug)]
pub struct Material {
    /// Linear base color and alpha.
    pub color: [f32; 4],
    pub texture: Option<usize>,
    /// The texture already carries its own shading.
    pub unlit: bool,
    pub blend: bool,
    pub metallic: f32,
    pub roughness: f32,
}

/// All triangles of one material, in model space.
pub struct Batch {
    pub material: Material,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Straight (not premultiplied) sRGB pixels.
pub struct Image {
    pub legend: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub struct Scene {
    pub parts: Vec<Option<String>>,
    /// Opaque batches first, blended ones last.
    pub batches: Vec<Batch>,
    pub images: Vec<Image>,
    pub center: Vec3,
    pub radius: f32,
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, String> {
    bytes
        .get(at..at + 4)
        .and_then(|b| b.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "truncated file".into())
}

/// The JSON and BIN chunks of a GLB container.
fn chunks(bytes: &[u8]) -> Result<(&[u8], &[u8]), String> {
    if bytes.get(..4) != Some(MAGIC) || u32_at(bytes, 4)? != 2 {
        return Err("not a glTF 2.0 binary".into());
    }
    let mut at = 12;
    let (mut json, mut bin) = (None, None);
    while at + 8 <= bytes.len() {
        let length = u32_at(bytes, at)? as usize;
        let kind = u32_at(bytes, at + 4)?;
        let data = bytes
            .get(at + 8..at + 8 + length)
            .ok_or("truncated chunk")?;
        match kind {
            CHUNK_JSON => json = Some(data),
            CHUNK_BIN => bin = Some(data),
            _ => {}
        }
        at += 8 + length;
    }
    Ok((json.ok_or("no JSON chunk")?, bin.ok_or("no BIN chunk")?))
}

struct Reader<'a> {
    doc: &'a Document,
    bin: &'a [u8],
}

impl Reader<'_> {
    fn view(&self, index: usize) -> Result<(&[u8], Option<usize>), String> {
        let view = self.doc.buffer_views.get(index).ok_or("bad buffer view")?;
        let data = self
            .bin
            .get(view.byte_offset..view.byte_offset + view.byte_length)
            .ok_or("buffer view out of range")?;
        Ok((data, view.byte_stride))
    }

    fn floats<const N: usize>(&self, index: usize) -> Result<Vec<[f32; N]>, String> {
        let accessor = self.doc.accessors.get(index).ok_or("bad accessor")?;
        let width = match accessor.kind.as_str() {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            _ => 0,
        };
        if accessor.component_type != FLOAT || width != N {
            return Err("unsupported vertex attribute".into());
        }
        let (data, stride) = self.view(accessor.buffer_view)?;
        let stride = stride.unwrap_or(4 * N);
        (0..accessor.count)
            .map(|i| {
                let mut out = [0.0; N];
                for (c, value) in out.iter_mut().enumerate() {
                    let at = accessor.byte_offset + i * stride + 4 * c;
                    *value = f32::from_bits(u32_at(data, at)?);
                }
                Ok(out)
            })
            .collect()
    }

    fn indices(&self, index: usize) -> Result<Vec<u32>, String> {
        let accessor = self.doc.accessors.get(index).ok_or("bad accessor")?;
        let (data, _) = self.view(accessor.buffer_view)?;
        let size = match accessor.component_type {
            5121 => 1,
            5123 => 2,
            5125 => 4,
            _ => return Err("unsupported index type".into()),
        };
        (0..accessor.count)
            .map(|i| {
                let at = accessor.byte_offset + i * size;
                let b = data.get(at..at + size).ok_or("index out of range")?;
                Ok(match size {
                    1 => u32::from(b[0]),
                    2 => u32::from(u16::from_le_bytes([b[0], b[1]])),
                    _ => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                })
            })
            .collect()
    }

    fn material(&self, index: Option<usize>) -> Material {
        let fallback = GltfMaterial::default();
        let m = index
            .and_then(|i| self.doc.materials.get(i))
            .unwrap_or(&fallback);
        let pbr = &m.pbr_metallic_roughness;
        Material {
            color: pbr.base_color_factor,
            texture: pbr
                .base_color_texture
                .as_ref()
                .and_then(|t| self.doc.textures.get(t.index))
                .map(|t| t.source),
            unlit: m.extensions.contains_key("KHR_materials_unlit"),
            blend: m.alpha_mode.as_deref() == Some("BLEND"),
            metallic: pbr.metallic_factor,
            roughness: pbr.roughness_factor,
        }
    }

    fn image(&self, image: &GltfImage) -> Result<Image, String> {
        let (data, _) = self.view(image.buffer_view)?;
        let mut decoded = image::load_from_memory_with_format(data, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?
            .to_rgba8();
        // The same asset also fills the device workspace: retain native legend detail there.
        while decoded.width().max(decoded.height()) > MAX_TEXTURE_SIDE {
            let (w, h) = (decoded.width().div_ceil(2), decoded.height().div_ceil(2));
            decoded =
                image::imageops::resize(&decoded, w, h, image::imageops::FilterType::Triangle);
        }
        Ok(Image {
            legend: Vec::new(),
            width: decoded.width(),
            height: decoded.height(),
            rgba: decoded.into_raw(),
        })
    }
}

/// Collects every primitive under `node` into per-material batches.
fn walk(
    reader: &Reader<'_>,
    node: usize,
    parent: &Mat4,
    batches: &mut Vec<(Option<usize>, Batch)>,
    depth: usize,
) -> Result<(), String> {
    let n = reader.doc.nodes.get(node).ok_or("bad node")?;
    if depth > 32 {
        return Err("node hierarchy too deep".into());
    }
    let local = n.matrix.unwrap_or_else(|| {
        math::from_trs(
            n.translation.unwrap_or([0.0; 3]),
            n.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]),
            n.scale.unwrap_or([1.0; 3]),
        )
    });
    let world = math::mul(parent, &local);
    if let Some(mesh) = n.mesh {
        for p in &reader.doc.meshes.get(mesh).ok_or("bad mesh")?.primitives {
            if p.mode != TRIANGLES {
                return Err("only triangle lists are supported".into());
            }
            let positions = reader.floats::<3>(p.attributes.position)?;
            let normals = match p.attributes.normal {
                Some(i) => reader.floats::<3>(i)?,
                None => vec![[0.0, 1.0, 0.0]; positions.len()],
            };
            let uvs = match p.attributes.uv {
                Some(i) => reader.floats::<2>(i)?,
                None => vec![[0.0; 2]; positions.len()],
            };
            let indices = match p.indices {
                Some(i) => reader.indices(i)?,
                None => (0..positions.len() as u32).collect(),
            };
            if normals.len() != positions.len() || uvs.len() != positions.len() {
                return Err("attribute counts differ".into());
            }
            if indices.iter().any(|&i| i as usize >= positions.len()) {
                return Err("index out of range".into());
            }
            let slot = match batches.iter().position(|(m, _)| *m == p.material) {
                Some(slot) => slot,
                None => {
                    batches.push((
                        p.material,
                        Batch {
                            material: reader.material(p.material),
                            vertices: Vec::new(),
                            indices: Vec::new(),
                        },
                    ));
                    batches.len() - 1
                }
            };
            let batch = &mut batches[slot].1;
            let base = batch.vertices.len() as u32;
            batch
                .vertices
                .extend(
                    positions
                        .iter()
                        .zip(&normals)
                        .zip(&uvs)
                        .map(|((&p, &n), &uv)| Vertex {
                            part: node,
                            bounce: [(0, 0.0); 4],
                            legend: 0.0,
                            position: math::transform_point(&world, p),
                            normal: math::transform_direction(&world, n),
                            uv,
                        }),
                );
            batch.indices.extend(indices.iter().map(|i| base + i));
        }
    }
    for &child in &n.children {
        walk(reader, child, &world, batches, depth + 1)?;
    }
    Ok(())
}

pub fn parse(bytes: &[u8]) -> Result<Scene, String> {
    let (json, bin) = chunks(bytes)?;
    let doc: Document = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let reader = Reader { doc: &doc, bin };
    let roots = &doc.scenes.get(doc.scene).ok_or("no scene")?.nodes;
    let mut batches = Vec::new();
    for &root in roots {
        walk(&reader, root, &math::IDENTITY, &mut batches, 0)?;
    }
    let mut batches: Vec<Batch> = batches.into_iter().map(|(_, b)| b).collect();
    batches.sort_by_key(|b| b.material.blend);
    let images = doc
        .images
        .iter()
        .map(|i| reader.image(i))
        .collect::<Result<Vec<_>, _>>()?;
    if batches
        .iter()
        .any(|b| b.material.texture.is_some_and(|t| t >= images.len()))
    {
        return Err("texture out of range".into());
    }
    let (center, radius) = bounds(&batches).ok_or("empty model")?;
    Ok(Scene {
        parts: doc.nodes.iter().map(|n| n.name.clone()).collect(),
        batches,
        images,
        center,
        radius,
    })
}

fn bounds(batches: &[Batch]) -> Option<(Vec3, f32)> {
    let mut points = batches.iter().flat_map(|b| &b.vertices).map(|v| v.position);
    let first = points.next()?;
    let (mut lo, mut hi) = (first, first);
    for p in points {
        for c in 0..3 {
            lo[c] = lo[c].min(p[c]);
            hi[c] = hi[c].max(p[c]);
        }
    }
    let center = [
        (lo[0] + hi[0]) / 2.0,
        (lo[1] + hi[1]) / 2.0,
        (lo[2] + hi[2]) / 2.0,
    ];
    let half = math::sub(hi, center);
    Some((center, math::dot(half, half).sqrt()))
}
