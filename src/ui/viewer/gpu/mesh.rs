use super::super::{
    glb::{Image, Scene},
    preview::binding,
};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    part: u32,
    legend: f32,
    sources: [u32; 4],
    weights: [f32; 4],
    flags: [f32; 2],
}
pub(super) const ATTRIBUTES: [wgpu::VertexAttribute; 8] = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x2,3=>Uint32,4=>Float32,5=>Uint32x4,6=>Float32x4,7=>Float32x2];
pub(super) fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ATTRIBUTES,
    }
}

pub(super) struct Batch {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub count: u32,
    pub material: wgpu::BindGroup,
}
pub(super) struct Mesh {
    pub batches: Vec<Batch>,
    pub parts: usize,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        visual: &str,
        scene: &Scene,
    ) -> Self {
        let binding = binding(Some(visual));
        let images: Vec<_> = scene
            .images
            .iter()
            .map(|image| texture(device, queue, image))
            .collect();
        let white = texture(
            device,
            queue,
            &Image {
                width: 1,
                height: 1,
                rgba: vec![255; 4],
                legend: vec![0],
            },
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("model anisotropic sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        let batches = scene
            .batches
            .iter()
            .map(|batch| {
                let vertices: Vec<_> = batch
                    .vertices
                    .iter()
                    .map(|v| {
                        let name = scene.parts[v.part].as_deref().unwrap_or_default();
                        Vertex {
                            position: std::array::from_fn(|i| {
                                (v.position[i] - scene.center[i]) / scene.radius
                            }),
                            normal: v.normal,
                            uv: v.uv,
                            part: v.part as u32,
                            legend: v.legend,
                            sources: v.bounce.map(|(p, _)| p as u32),
                            weights: v.bounce.map(|(_, w)| w),
                            flags: [
                                f32::from(binding.is_some_and(|b| b.key_index(name).is_some())),
                                f32::from(binding.is_some_and(|b| b.socket_index(name).is_some())),
                            ],
                        }
                    })
                    .collect();
                let m = &batch.material;
                let parameters = [
                    m.color,
                    [
                        m.metallic,
                        m.roughness,
                        f32::from(m.unlit),
                        f32::from(m.blend),
                    ],
                ];
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("material"),
                    contents: bytemuck::cast_slice(&parameters),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
                let (albedo, mask) = m
                    .texture
                    .and_then(|index| images.get(index))
                    .unwrap_or(&white);
                let material = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("model material"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(albedo),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(mask),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                });
                Batch {
                    vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("device vertices"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                    indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("device indices"),
                        contents: bytemuck::cast_slice(&batch.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                    count: batch.indices.len() as u32,
                    material,
                }
            })
            .collect();
        Self {
            batches,
            parts: scene.parts.len().max(1),
        }
    }
}

fn texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &Image,
) -> (wgpu::TextureView, wgpu::TextureView) {
    let size = wgpu::Extent3d {
        width: image.width,
        height: image.height,
        depth_or_array_layers: 1,
    };
    let levels = image.width.max(image.height).ilog2() + 1;
    let create = |format| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("model texture mip chain"),
            size,
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    };
    let albedo = create(wgpu::TextureFormat::Rgba8UnormSrgb);
    let mask = create(wgpu::TextureFormat::R8Unorm);
    let mut rgba = image::RgbaImage::from_raw(image.width, image.height, image.rgba.clone())
        .unwrap_or_else(|| image::RgbaImage::new(image.width, image.height));
    let mut legend = image::GrayImage::from_raw(
        image.width,
        image.height,
        if image.legend.is_empty() {
            vec![0; (image.width * image.height) as usize]
        } else {
            image.legend.clone()
        },
    )
    .unwrap_or_else(|| image::GrayImage::new(image.width, image.height));
    for mip in 0..levels {
        let extent = wgpu::Extent3d {
            width: rgba.width(),
            height: rgba.height(),
            depth_or_array_layers: 1,
        };
        for (texture, bytes, stride) in [
            (&albedo, rgba.as_raw().as_slice(), 4),
            (&mask, legend.as_raw().as_slice(), 1),
        ] {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: mip,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(extent.width * stride),
                    rows_per_image: Some(extent.height),
                },
                extent,
            );
        }
        if mip + 1 < levels {
            rgba = image::imageops::resize(
                &rgba,
                (rgba.width() / 2).max(1),
                (rgba.height() / 2).max(1),
                image::imageops::FilterType::Triangle,
            );
            legend = image::imageops::resize(
                &legend,
                (legend.width() / 2).max(1),
                (legend.height() / 2).max(1),
                image::imageops::FilterType::Triangle,
            );
        }
    }
    (
        albedo.create_view(&Default::default()),
        mask.create_view(&Default::default()),
    )
}
