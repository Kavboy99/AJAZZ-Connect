//! `--review DIRECTORY`: renders interface states offscreen to PNG files for design review. Uses
//! synthetic data only; no device, socket or window is opened.

use super::raster::{Canvas, update_textures};
use super::review_scenes::SCENES;
use crate::app::{Settings, Theme};
use crate::ui::{App, preload_models};
use anyhow::Result;
use eframe::egui::{
    Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput, Rect, pos2, vec2,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Enough 60 Hz frames for every spring to settle.
const SETTLE_FRAMES: usize = 140;

/// Every scene is rendered at each width: the minimum window, the default and a wide monitor
/// (`docs/DESIGN.md`, section 5.1).
const WIDTHS: [(&str, f32); 3] = [("narrow", 720.0), ("default", 1040.0), ("wide", 1600.0)];

pub(super) struct Scene {
    pub(super) name: &'static str,
    pub(super) theme: Theme,
    pub(super) height: f32,
    /// Where the pointer rests, as a share of the window size.
    pub(super) pointer: Option<(f32, f32)>,
    pub(super) setup: fn(&mut App),
}

fn render(
    scene: &Scene,
    (label, width): (&str, f32),
    out: &Path,
    gpu: &crate::ui::ReviewGpu,
) -> Result<()> {
    let ctx = Context::default();
    gpu.install(&ctx);
    let mut app = App::new(
        &ctx,
        false,
        Settings {
            theme: scene.theme,
            developer_mode: scene.name.contains("catalog") || scene.name == "settings",
            show_all_devices: scene.name.contains("catalog"),
            ..Settings::default()
        },
    );
    (scene.setup)(&mut app);
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(width, scene.height));
    let mut textures = BTreeMap::new();
    let mut last: Option<eframe::egui::FullOutput> = None;
    let mut motion_times = Vec::new();
    let mut idle_gpu = None;
    let pointer = scene
        .pointer
        .map(|(x, y)| Event::PointerMoved(pos2(x * width, y * scene.height)));
    let studio = scene.name.starts_with("studio-");
    for frame in 0..if studio { 240 } else { SETTLE_FRAMES } {
        let mut events: Vec<_> = pointer.iter().cloned().collect();
        if studio {
            events.extend(super::review_studio::input(
                scene.name,
                frame,
                last.as_ref().map_or(&[], |o| &o.shapes),
                rect,
            )?);
        }
        if scene.name == "device-opening" || scene.name == "device-reversing" {
            let open = frame == 24 || frame == 25;
            if scene.name == "device-reversing" && frame == 32 {
                events.push(Event::Key {
                    key: eframe::egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::default(),
                });
            }
            let close = scene.name == "device-opening" && (frame == 64 || frame == 65);
            if open || close {
                let source = ctx
                    .data(|d| d.get_temp::<Rect>(Id::new(("device-tile", 0usize)).with("origin")));
                let pos = if open {
                    source.map_or(Pos2::ZERO, |r| r.center())
                } else {
                    pos2(48.0 + 56.0, 40.0)
                };
                events.push(Event::PointerMoved(pos));
                events.push(Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: frame == 24 || frame == 64,
                    modifiers: Modifiers::default(),
                });
            }
        }
        let input = RawInput {
            screen_rect: Some(rect),
            events,
            time: Some(frame as f64 / 60.0),
            predicted_dt: 1.0 / 60.0,
            ..Default::default()
        };
        let started = std::time::Instant::now();
        let output = ctx.run(input, |ctx| app.frame(ctx));
        if studio && frame == 200 {
            idle_gpu = Some(crate::ui::gpu_stats(&ctx));
        }
        if (26..60).contains(&frame) {
            motion_times.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        update_textures(&mut textures, &output.textures_delta);
        if (scene.name.starts_with("device-open") || scene.name == "device-reversing")
            && [23, 27, 31, 37, 44, 50, 60, 69, 75, 82, 100].contains(&frame)
        {
            let primitives = ctx.tessellate(output.shapes.clone(), output.pixels_per_point);
            let mut canvas = Canvas::new(width as usize, scene.height as usize);
            canvas.draw(&primitives, &textures);
            canvas.save(&out.join(format!("{}-{label}-{frame}.png", scene.name)))?;
        }
        if studio && [42, 65, 78, 123].contains(&frame) {
            let primitives = ctx.tessellate(output.shapes.clone(), output.pixels_per_point);
            let mut canvas = Canvas::new(width as usize, scene.height as usize);
            canvas.draw(&primitives, &textures);
            canvas.save(&out.join(format!("{}-{label}-{frame}.png", scene.name)))?;
        }
        last = Some(output);
    }
    if scene.name == "device-opening" || scene.name == "device-reversing" {
        anyhow::ensure!(
            !crate::ui::device_reveal::hidden(&ctx, Id::new(("device-tile", 0usize))),
            "device reveal did not return to the list"
        );
        let metrics = serde_json::json!({
            "frame_mean_ms": motion_times.iter().sum::<f64>() / motion_times.len() as f64,
            "frame_max_ms": motion_times.iter().copied().fold(0.0, f64::max),
            "idle_repaint_delay_ms": last.as_ref().and_then(|o| o.viewport_output.get(&eframe::egui::ViewportId::ROOT)).map(|o| o.repaint_delay.as_millis().min(u64::MAX as u128) as u64),
        });
        std::fs::write(
            out.join(format!("{}-{label}-timing.json", scene.name)),
            serde_json::to_vec_pretty(&metrics)?,
        )?;
    }
    let output = last.ok_or_else(|| anyhow::anyhow!("no frame"))?;
    if studio {
        let stats = crate::ui::gpu_stats(&ctx);
        anyhow::ensure!(
            idle_gpu
                .as_ref()
                .is_some_and(|before| before["renders"] == stats["renders"]),
            "GPU work continued while workspace was idle"
        );
        anyhow::ensure!(
            stats["working_views"] == 0,
            "temporary GPU attachments survived idle"
        );
        if scene.name.ends_with("-pointer") {
            let yaw = stats["camera_angles"][0].as_f64().unwrap_or(0.0).abs();
            anyhow::ensure!(
                yaw > 0.005 && yaw < 0.08,
                "zoomed model did not retain restrained pointer tracking: {yaw}"
            );
        }
        std::fs::write(
            out.join(format!("{}-{label}-gpu.json", scene.name)),
            serde_json::to_vec_pretty(&stats)?,
        )?;
        super::review_studio::verify(scene.name, &output.shapes, rect)?;
        let delay = output
            .viewport_output
            .get(&eframe::egui::ViewportId::ROOT)
            .map(|o| o.repaint_delay);
        anyhow::ensure!(
            delay == Some(std::time::Duration::MAX),
            "{}: workspace continues repainting at rest: {delay:?}",
            scene.name
        );
        std::fs::write(
            out.join(format!("{}-{label}-timing.json", scene.name)),
            serde_json::to_vec_pretty(&serde_json::json!({
                "frame_mean_ms": motion_times.iter().sum::<f64>() / motion_times.len() as f64,
                "frame_max_ms": motion_times.iter().copied().fold(0.0, f64::max),
                "idle_repaint": false
            }))?,
        )?;
    }
    let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
    let mut canvas = Canvas::new(width as usize, scene.height as usize);
    canvas.draw(&primitives, &textures);
    canvas.save(&out.join(format!("{}-{label}.png", scene.name)))
}

pub fn run(directory: &Path, filter: Option<&str>) -> Result<()> {
    let gpu = crate::ui::ReviewGpu::new()?;
    preload_models();
    std::fs::create_dir_all(directory)?;
    let studio = super::review_studio::scenes();
    let lighting = super::review_lighting::scenes();
    let scenes: Vec<_> = SCENES
        .iter()
        .chain(studio.iter())
        .chain(lighting.iter())
        .filter(|scene| filter.is_none_or(|f| scene.name.starts_with(f)))
        .collect();
    anyhow::ensure!(!scenes.is_empty(), "no review scene matches the filter");
    for scene in &scenes {
        for size in WIDTHS {
            render(scene, size, directory, &gpu)?;
        }
    }
    std::fs::write(
        directory.join("complete.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "scenes": scenes.len(),
            "widths": WIDTHS.map(|(_, width)| width),
            "workspace_input_and_idle_checks": "passed"
        }))?,
    )?;
    Ok(())
}
