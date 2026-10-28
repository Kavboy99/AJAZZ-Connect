//! Readback is limited to offline screenshots; the native window samples the GPU texture directly.
use anyhow::{Context, Result};
pub(super) fn image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    side: u32,
) -> Result<eframe::egui::ColorImage> {
    let stride = (side * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("offline screenshot readback"),
        size: u64::from(stride) * u64::from(side),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(side),
            },
        },
        wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
    );
    let submission = queue.submit([encoder.finish()]);
    let (send, receive) = std::sync::mpsc::channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        let _ = send.send(result);
    });
    device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: None,
    })?;
    receive.recv().context("GPU readback callback missing")??;
    let data = buffer.get_mapped_range(..);
    let mut bytes = Vec::with_capacity((side * side * 4) as usize);
    for row in data.chunks_exact(stride as usize).take(side as usize) {
        bytes.extend_from_slice(&row[..side as usize * 4]);
    }
    let image = eframe::egui::ColorImage::from_rgba_premultiplied([side as usize; 2], &bytes);
    drop(data);
    buffer.unmap();
    Ok(image)
}
