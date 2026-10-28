//! Only the resolved image survives idle. Work attachments are released when motion stops.
use super::super::{Key, Pose, preview::PartEffect};
use super::{attachments::Attachments, mesh::Mesh, pipelines::Pipelines};
pub(super) struct Target {
    pub side: u32,
    capacity: u32,
    parts: usize,
    pub output: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub key: Option<Key>,
    work: Option<Attachments>,
}
impl Target {
    pub fn new(device: &wgpu::Device, side: u32, parts: usize) -> Self {
        let capacity = (side.div_ceil(128) * 128).min(device.limits().max_texture_dimension_2d);
        let output = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cached device image"),
            size: wgpu::Extent3d {
                width: capacity,
                height: capacity,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = output.create_view(&Default::default());
        Self {
            side,
            capacity,
            parts,
            output,
            view,
            key: None,
            work: None,
        }
    }
    pub fn fits(&self, side: u32) -> bool {
        side <= self.capacity
    }
    pub fn scale(&self) -> f32 {
        self.side as f32 / self.capacity as f32
    }
    pub fn working(&self) -> bool {
        self.work.is_some()
    }
    pub fn trim(&mut self) {
        self.work = None;
    }
    pub fn render(
        &mut self,
        gpu: (&wgpu::Device, &wgpu::Queue),
        p: &Pipelines,
        mesh: &Mesh,
        pose: &Pose,
        effects: &[PartEffect],
    ) {
        let work = self
            .work
            .get_or_insert_with(|| Attachments::new(gpu.0, p, self.capacity, self.parts));
        work.render(gpu, p, mesh, (&self.view, self.side), pose, effects);
    }
}
