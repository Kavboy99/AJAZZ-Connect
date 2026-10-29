use super::mesh;
pub(super) struct Pipelines {
    pub main: wgpu::RenderPipeline,
    pub shadow: wgpu::RenderPipeline,
    pub downsample: wgpu::RenderPipeline,
    pub horizontal: wgpu::RenderPipeline,
    pub vertical: wgpu::RenderPipeline,
    pub composite: wgpu::RenderPipeline,
    pub globals: wgpu::BindGroupLayout,
    pub shadow_globals: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
    pub blur: wgpu::BindGroupLayout,
    pub post: wgpu::BindGroupLayout,
}
fn entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty,
        count: None,
    }
}
fn uniform() -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}
fn texture(sample_type: wgpu::TextureSampleType, multisampled: bool) -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        sample_type,
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled,
    }
}
impl Pipelines {
    pub fn new(device: &wgpu::Device) -> Self {
        use wgpu::ShaderStages as S;
        let float = texture(wgpu::TextureSampleType::Float { filterable: true }, false);
        let sampler = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);
        let global_entries = [
            entry(0, S::VERTEX_FRAGMENT, uniform()),
            entry(
                1,
                S::VERTEX,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            ),
            entry(
                2,
                S::FRAGMENT,
                texture(wgpu::TextureSampleType::Depth, false),
            ),
            entry(
                3,
                S::FRAGMENT,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
            ),
        ];
        let globals = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene globals"),
            entries: &global_entries,
        });
        let shadow_globals = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow globals"),
            entries: &global_entries[..2],
        });
        let material = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material layout"),
            entries: &[
                entry(0, S::FRAGMENT, uniform()),
                entry(1, S::FRAGMENT, float),
                entry(2, S::FRAGMENT, float),
                entry(3, S::FRAGMENT, sampler),
            ],
        });
        let blur = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom layout"),
            entries: &[
                entry(0, S::FRAGMENT, float),
                entry(1, S::FRAGMENT, sampler),
                entry(5, S::FRAGMENT, uniform()),
            ],
        });
        let post = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("composite layout"),
            entries: &[
                entry(0, S::FRAGMENT, float),
                entry(1, S::FRAGMENT, sampler),
                entry(2, S::FRAGMENT, float),
                entry(
                    3,
                    S::FRAGMENT,
                    texture(wgpu::TextureSampleType::Depth, true),
                ),
                entry(4, S::FRAGMENT, float),
                entry(5, S::FRAGMENT, uniform()),
            ],
        });
        let scene = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("physically based device lighting"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scene.wgsl").into()),
        });
        let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom and ambient occlusion"),
            source: wgpu::ShaderSource::Wgsl(include_str!("post.wgsl").into()),
        });
        let main_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("device layout"),
            bind_group_layouts: &[&globals, &material],
            push_constant_ranges: &[],
        });
        let shadow_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow layout"),
            bind_group_layouts: &[&shadow_globals],
            push_constant_ranges: &[],
        });
        let color_target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let main = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("device PBR / 4x MSAA"),
            layout: Some(&main_layout),
            vertex: wgpu::VertexState {
                module: &scene,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[mesh::vertex_layout()],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &scene,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[
                    color_target(wgpu::TextureFormat::Rgba16Float),
                    color_target(wgpu::TextureFormat::Rgba16Float),
                    color_target(wgpu::TextureFormat::Rgba8Unorm),
                ],
            }),
            multiview: None,
            cache: None,
        });
        let shadow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("soft shadow depth"),
            layout: Some(&shadow_layout),
            vertex: wgpu::VertexState {
                module: &scene,
                entry_point: Some("shadow_vertex"),
                compilation_options: Default::default(),
                buffers: &[mesh::vertex_layout()],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: Default::default(),
            fragment: None,
            multiview: None,
            cache: None,
        });
        let full_screen = |name: &'static str, entry, bindings: &wgpu::BindGroupLayout, format| {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(name),
                bind_group_layouts: &[bindings],
                push_constant_ranges: &[],
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &post_shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &post_shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            })
        };
        let downsample = full_screen(
            "LED bloom prefilter",
            "downsample",
            &blur,
            wgpu::TextureFormat::Rgba16Float,
        );
        let horizontal = full_screen(
            "horizontal LED bloom",
            "horizontal",
            &blur,
            wgpu::TextureFormat::Rgba16Float,
        );
        let vertical = full_screen(
            "vertical LED bloom",
            "vertical",
            &blur,
            wgpu::TextureFormat::Rgba16Float,
        );
        let composite = full_screen(
            "tone map and ambient occlusion",
            "composite",
            &post,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        Self {
            main,
            shadow,
            downsample,
            horizontal,
            vertical,
            composite,
            globals,
            shadow_globals,
            material,
            blur,
            post,
        }
    }
}
