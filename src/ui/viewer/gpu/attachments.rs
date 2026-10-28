use super::{matrices::Globals, pipelines::Pipelines};
mod draw;
use bytemuck::Zeroable;
use wgpu::util::DeviceExt;

pub(super) struct Attachments {
    pub side: u32,
    uniform: wgpu::Buffer,
    effects: wgpu::Buffer,
    globals: wgpu::BindGroup,
    shadow_globals: wgpu::BindGroup,
    color: [wgpu::TextureView; 3],
    multisampled: [wgpu::TextureView; 3],
    depth: wgpu::TextureView,
    shadow: wgpu::TextureView,
    bloom: [wgpu::TextureView; 2],
    blur_groups: [wgpu::BindGroup; 2],
    post: wgpu::BindGroup,
}
fn view(
    device: &wgpu::Device,
    label: &'static str,
    side: u32,
    format: wgpu::TextureFormat,
    samples: u32,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}
impl Attachments {
    pub fn new(device: &wgpu::Device, p: &Pipelines, side: u32, parts: usize) -> Self {
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("view globals"),
            contents: bytemuck::bytes_of(&Globals::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let effects = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("per-part LED and key state"),
            size: (parts * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow = view(
            device,
            "2048 soft shadow",
            2048,
            wgpu::TextureFormat::Depth32Float,
            1,
        );
        let compare = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow comparison"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let entries = [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: effects.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&shadow),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&compare),
            },
        ];
        let globals = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("view bindings"),
            layout: &p.globals,
            entries: &entries,
        });
        let shadow_globals = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow bindings"),
            layout: &p.shadow_globals,
            entries: &entries[..2],
        });
        let formats = [
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Rgba8Unorm,
        ];
        let color =
            formats.map(|format| view(device, "resolved scene attachment", side, format, 1));
        let multisampled =
            formats.map(|format| view(device, "4x scene attachment", side, format, 4));
        let depth = view(
            device,
            "4x scene depth",
            side,
            wgpu::TextureFormat::Depth32Float,
            4,
        );
        let bloom = std::array::from_fn(|_| {
            view(
                device,
                "quarter resolution bloom",
                side.div_ceil(4),
                wgpu::TextureFormat::Rgba16Float,
                1,
            )
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let blur_group = |source| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bloom bindings"),
                layout: &p.blur,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: uniform.as_entire_binding(),
                    },
                ],
            })
        };
        let blur_groups = [blur_group(&color[1]), blur_group(&bloom[0])];
        let post = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tone map bindings"),
            layout: &p.post,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&bloom[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&depth),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&color[2]),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: uniform.as_entire_binding(),
                },
            ],
        });
        Self {
            side,
            uniform,
            effects,
            globals,
            shadow_globals,
            color,
            multisampled,
            depth,
            shadow,
            bloom,
            blur_groups,
            post,
        }
    }
}
