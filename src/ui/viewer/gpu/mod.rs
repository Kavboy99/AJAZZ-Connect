//! Reactive GPU views. Native rendering never reads pixels back to the CPU.
mod attachments;
mod matrices;
mod mesh;
mod pipelines;
mod readback;
mod target;

use super::{Key, Pose, glb::Scene, preview::PartEffect};
use anyhow::{Context as _, Result};
use eframe::{
    egui::{Context, Id, TextureHandle, TextureId, TextureOptions},
    egui_wgpu::RenderState,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct ReviewGpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter: String,
}
impl ReviewGpu {
    pub fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))?;
        anyhow::ensure!(
            adapter.get_info().device_type != wgpu::DeviceType::Cpu,
            "hardware GPU unavailable"
        );
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("offline GPU visual review"),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            }))?;
        Ok(Self {
            device,
            queue,
            adapter: adapter.get_info().name,
        })
    }
    pub fn install(&self, ctx: &Context) {
        install(
            ctx,
            Backend::new(
                self.device.clone(),
                self.queue.clone(),
                self.adapter.clone(),
                None,
            ),
        );
    }
}

pub(crate) fn install_native(ctx: &Context, state: RenderState) {
    install(
        ctx,
        Backend::new(
            state.device.clone(),
            state.queue.clone(),
            state.adapter.get_info().name,
            Some(state),
        ),
    );
}
pub(crate) fn configuration() -> eframe::egui_wgpu::WgpuConfiguration {
    let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::default();
    setup.instance_descriptor.backends = wgpu::Backends::DX12;
    // HWND swapchains are opaque even when the window and clear color are transparent.
    setup
        .instance_descriptor
        .backend_options
        .dx12
        .presentation_system = wgpu_types::Dx12SwapchainKind::DxgiFromVisual;
    let descriptor = setup.device_descriptor.clone();
    setup.device_descriptor = Arc::new(move |adapter| {
        let mut value = descriptor(adapter);
        value.memory_hints = wgpu::MemoryHints::MemoryUsage;
        value
    });
    eframe::egui_wgpu::WgpuConfiguration {
        wgpu_setup: setup.into(),
        desired_maximum_frame_latency: Some(1),
        ..Default::default()
    }
}
fn install(ctx: &Context, backend: Backend) {
    ctx.data_mut(|d| d.insert_temp(Id::new("device-gpu"), Arc::new(Mutex::new(backend))));
}
fn backend(ctx: &Context) -> Option<Arc<Mutex<Backend>>> {
    ctx.data(|d| d.get_temp(Id::new("device-gpu")))
}

struct Entry {
    target: target::Target,
    native: Option<TextureId>,
    cpu: Option<TextureHandle>,
}
struct Backend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: pipelines::Pipelines,
    meshes: HashMap<String, mesh::Mesh>,
    entries: HashMap<(Id, String), Entry>,
    native: Option<RenderState>,
    adapter: String,
    renders: u64,
    uploads: u64,
    readbacks: u64,
    last_pose: [f32; 2],
}
impl Backend {
    fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        adapter: String,
        native: Option<RenderState>,
    ) -> Self {
        let pipelines = pipelines::Pipelines::new(&device);
        Self {
            device,
            queue,
            pipelines,
            meshes: HashMap::new(),
            entries: HashMap::new(),
            native,
            adapter,
            renders: 0,
            uploads: 0,
            readbacks: 0,
            last_pose: [0.0; 2],
        }
    }
    fn draw(
        &mut self,
        ctx: &Context,
        view: (Id, &str),
        scene: &Scene,
        pose: &Pose,
        effects: &[PartEffect],
        key: Key,
    ) -> Result<(TextureId, f32)> {
        let (id, visual) = view;
        if !self.meshes.contains_key(visual) {
            self.meshes.insert(
                visual.to_owned(),
                mesh::Mesh::new(
                    &self.device,
                    &self.queue,
                    &self.pipelines.material,
                    visual,
                    scene,
                ),
            );
            self.uploads += 1;
        }
        let mesh = self
            .meshes
            .get(visual)
            .context("missing uploaded device mesh")?;
        let map_key = (id, visual.to_owned());
        let side = (key.side as u32)
            .min(self.device.limits().max_texture_dimension_2d)
            .max(1);
        if self
            .entries
            .get(&map_key)
            .is_some_and(|e| !e.target.fits(side))
            && let Some(old) = self.entries.remove(&map_key)
            && let (Some(state), Some(id)) = (&self.native, old.native)
        {
            state.renderer.write().free_texture(&id);
        }
        let entry = self.entries.entry(map_key).or_insert_with(|| Entry {
            target: target::Target::new(&self.device, side, mesh.parts),
            native: None,
            cpu: None,
        });
        entry.target.side = side;
        if entry.target.key.as_ref() != Some(&key) {
            entry.target.render(
                (&self.device, &self.queue),
                &self.pipelines,
                mesh,
                pose,
                effects,
            );
            entry.target.key = Some(key);
            self.renders += 1;
            self.last_pose = [pose.yaw, pose.pitch];
            if self.native.is_none() {
                let image = readback::image(&self.device, &self.queue, &entry.target.output, side)?;
                self.readbacks += 1;
                match &mut entry.cpu {
                    Some(texture) => texture.set(image, TextureOptions::LINEAR),
                    None => {
                        entry.cpu = Some(ctx.load_texture(
                            "GPU review device",
                            image,
                            TextureOptions::LINEAR,
                        ))
                    }
                }
            }
        }
        if let Some(state) = &self.native {
            Ok((
                *entry.native.get_or_insert_with(|| {
                    state.renderer.write().register_native_texture(
                        &self.device,
                        &entry.target.view,
                        wgpu::FilterMode::Linear,
                    )
                }),
                entry.target.scale(),
            ))
        } else {
            entry
                .cpu
                .as_ref()
                .map(TextureHandle::id)
                .map(|id| (id, 1.0))
                .context("missing review texture")
        }
    }
}

pub(super) fn draw(
    ctx: &Context,
    id: Id,
    visual: &str,
    scene: &Scene,
    pose: &Pose,
    effects: &[PartEffect],
    key: Key,
) -> Option<Result<(TextureId, f32)>> {
    let backend = backend(ctx)?;
    Some(
        backend
            .lock()
            .map_err(|_| anyhow::anyhow!("GPU cache lock poisoned"))
            .and_then(|mut b| b.draw(ctx, (id, visual), scene, pose, effects, key)),
    )
}
pub fn stats(ctx: &Context) -> serde_json::Value {
    backend(ctx).and_then(|b| b.lock().ok().map(|b| serde_json::json!({"adapter":b.adapter,"renders":b.renders,"mesh_uploads":b.uploads,"readbacks":b.readbacks,"native_zero_readback":b.native.is_some() && b.readbacks==0,"working_views":b.entries.values().filter(|e|e.target.working()).count(),"camera_angles":b.last_pose}))).unwrap_or(serde_json::Value::Null)
}
pub(crate) fn finish_frame(ctx: &Context, hidden: bool) {
    if (hidden || !ctx.has_requested_repaint())
        && let Some(b) = backend(ctx)
        && let Ok(mut b) = b.lock()
    {
        for entry in b.entries.values_mut() {
            entry.target.trim();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn gpu_shaders_validate() {
        for source in [include_str!("scene.wgsl"), include_str!("post.wgsl")] {
            let module = naga::front::wgsl::parse_str(source)
                .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
            naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            )
            .validate(&module)
            .unwrap_or_else(|error| panic!("{error:?}"));
        }
    }
}
