//! Display only (`docs/DESIGN.md`). Knows no drivers and no brands: it walks device capabilities
//! and draws the sections that exist.

mod app;
mod backup_actions;
mod backup_panel;
pub(crate) mod backup_tool;
mod icons;
mod motion;
mod pages;
mod shell;
pub(crate) mod strings;
mod theme;
mod viewer;
mod widgets;

pub use app::App;
pub use pages::Route;
pub use viewer::gpu::{ReviewGpu, stats as gpu_stats};
pub use viewer::preload as preload_models;

use crate::app::{Settings, Theme, identity};
use eframe::egui::{IconData, ViewportBuilder};
use std::sync::Arc;

/// Window and taskbar icon side; Windows scales it down for small sizes.
const WINDOW_ICON: u32 = 64;

#[cfg(windows)]
mod backdrop;

pub fn run() -> anyhow::Result<()> {
    run_window(None)
}
pub fn run_backup() -> anyhow::Result<()> {
    backup_tool::run()
}
pub fn run_preview(
    found: crate::app::events::Discovered,
    metrics: std::path::PathBuf,
    theme: Theme,
) -> anyhow::Result<()> {
    run_window(Some((found, metrics, theme)))
}
fn run_window(
    preview: Option<(crate::app::events::Discovered, std::path::PathBuf, Theme)>,
) -> anyhow::Result<()> {
    let settings = if let Some((_, _, theme)) = &preview {
        Settings {
            ephemeral: true,
            theme: *theme,
            ..Settings::default()
        }
    } else {
        Settings::load()
    };
    if !settings.ephemeral {
        crate::diagnostics::logging::install_panic_hook();
        let _ = crate::diagnostics::logging::configure(settings.logging_enabled);
        crate::diagnostics::logging::event("application", "studio.start", "ok");
    }
    let title = identity::PRODUCT_NAME;
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: viewer::gpu::configuration(),
        viewport: ViewportBuilder::default()
            .with_title(title)
            .with_inner_size([1360.0, 840.0])
            .with_min_inner_size([720.0, 520.0])
            .with_transparent(true)
            .with_icon(Arc::new(IconData {
                rgba: icons::app_icon::rgba(WINDOW_ICON),
                width: WINDOW_ICON,
                height: WINDOW_ICON,
            })),
        ..Default::default()
    };
    eframe::run_native(
        title,
        options,
        Box::new(|cc| {
            let gpu = cc
                .wgpu_render_state
                .clone()
                .ok_or_else(|| anyhow::anyhow!("GPU renderer unavailable"))?;
            viewer::gpu::install_native(&cc.egui_ctx, gpu);
            let mut app = App::new(&cc.egui_ctx, false, settings);
            if let Some((found, metrics, _)) = preview {
                app.session_mut().adopt(found);
                app.open(Route::Device(0));
                app.capture_gpu_metrics(&cc.egui_ctx, metrics);
            } else {
                app.watch_devices(&cc.egui_ctx);
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub(crate) mod device_reveal;
