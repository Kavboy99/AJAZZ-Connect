//! Offscreen review of the standalone application's own window, without Studio or device IO.
use crate::ui::{backup_tool::BackupApp, strings as s};
use anyhow::{Result, ensure};
use eframe::egui::{Context, Pos2, RawInput, Rect, vec2};
use std::{collections::BTreeMap, path::Path};

pub(super) fn run(directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory)?;
    for (width, label) in [(480.0, "small"), (640.0, "default"), (800.0, "wide")] {
        let ctx = Context::default();
        let mut app = BackupApp::new(&ctx, Some(super::review_state::backup_devices()));
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(width, 460.0));
        let mut textures = BTreeMap::new();
        let mut last = None;
        for frame in 0..180 {
            let output = ctx.run(
                RawInput {
                    screen_rect: Some(screen),
                    time: Some(frame as f64 / 60.0),
                    ..Default::default()
                },
                |ctx| app.frame(ctx),
            );
            super::raster::update_textures(&mut textures, &output.textures_delta);
            last = Some(output);
        }
        let output = last.ok_or_else(|| anyhow::anyhow!("missing backup frame"))?;
        for text in [s::BACKUP_TITLE, s::BACKUP_SAVE, s::BACKUP_LOAD] {
            let mut labels = vec![];
            for shape in &output.shapes {
                super::review_studio::labels(&shape.shape, text, &mut labels);
            }
            ensure!(
                labels.iter().any(|r| screen.contains_rect(*r)),
                "standalone control clipped: {text}"
            );
        }
        ensure!(
            output
                .viewport_output
                .get(&eframe::egui::ViewportId::ROOT)
                .is_some_and(|v| v.repaint_delay == std::time::Duration::MAX),
            "standalone continues repainting at rest"
        );
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
        let mut canvas = super::raster::Canvas::new(width as usize, 460);
        canvas.draw(&primitives, &textures);
        canvas.save(&directory.join(format!("backup-tool-{label}.png")))?;
    }
    std::fs::write(directory.join("complete.json"), br#"{"application":"standalone backup","widths":[480,640,800],"idle":"passed","hardware_io":false}"#)?;
    Ok(())
}
