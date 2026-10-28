use super::super::{matrices::Globals, mesh::Mesh, pipelines::Pipelines};
use super::Attachments;
use crate::ui::viewer::{Pose, preview::PartEffect};
impl Attachments {
    pub fn render(
        &self,
        gpu: (&wgpu::Device, &wgpu::Queue),
        p: &Pipelines,
        mesh: &Mesh,
        output: (&wgpu::TextureView, u32),
        pose: &Pose,
        effects: &[PartEffect],
    ) {
        let (device, queue) = gpu;
        let (output, side) = output;
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&Globals::new(pose, side, self.side)),
        );
        let fx: Vec<[f32; 4]> = (0..mesh.parts)
            .map(|i| {
                let e = effects.get(i).copied().unwrap_or_default();
                [e.emission[0], e.emission[1], e.emission[2], e.press]
            })
            .collect();
        queue.write_buffer(&self.effects, 0, bytemuck::cast_slice(&fx));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("changed device view"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("device soft shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&p.shadow);
            pass.set_bind_group(0, &self.shadow_globals, &[]);
            for batch in &mesh.batches {
                pass.set_vertex_buffer(0, batch.vertices.slice(..));
                pass.set_index_buffer(batch.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..batch.count, 0, 0..1);
            }
        }
        {
            let attachments: Vec<_> = (0..3)
                .map(|i| {
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.multisampled[i],
                        resolve_target: Some(&self.color[i]),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Discard,
                        },
                        depth_slice: None,
                    })
                })
                .collect();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("device material lighting"),
                color_attachments: &attachments,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&p.main);
            pass.set_viewport(0.0, 0.0, side as f32, side as f32, 0.0, 1.0);
            pass.set_bind_group(0, &self.globals, &[]);
            for batch in &mesh.batches {
                pass.set_bind_group(1, &batch.material, &[]);
                pass.set_vertex_buffer(0, batch.vertices.slice(..));
                pass.set_index_buffer(batch.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..batch.count, 0, 0..1);
            }
        }
        for (pipeline, group, output, extent) in [
            (
                &p.horizontal,
                &self.blur_groups[0],
                &self.bloom[0],
                side.div_ceil(4),
            ),
            (
                &p.vertical,
                &self.blur_groups[1],
                &self.bloom[1],
                side.div_ceil(4),
            ),
            (&p.composite, &self.post, output, side),
        ] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("device post process"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            pass.set_pipeline(pipeline);
            pass.set_viewport(0.0, 0.0, extent as f32, extent as f32, 0.0, 1.0);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}
