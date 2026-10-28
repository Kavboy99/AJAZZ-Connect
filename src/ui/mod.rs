//! Display only (`docs/DESIGN.md`). Knows no drivers and no brands: it walks device capabilities
//! and draws the sections that exist.

mod app;
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

/// Mica behind the window, shown through the navigation pane. DWM does the blur; when Windows
/// cannot provide it (Windows 10, transparency off), the pane falls back to an opaque token.
#[cfg(windows)]
fn apply_backdrop(cc: &eframe::CreationContext<'_>, dark: bool) -> bool {
    window_vibrancy::apply_mica(cc, Some(dark)).is_ok()
}

#[cfg(not(windows))]
fn apply_backdrop(_: &eframe::CreationContext<'_>, _: bool) -> bool {
    false
}

/// Re-tints Mica (and the title bar) after the user switches themes.
#[cfg(windows)]
fn tint_backdrop(frame: &eframe::Frame, dark: bool) {
    // Without Mica the pane already shows the opaque token; nothing else depends on this call.
    let _ = window_vibrancy::apply_mica(frame, Some(dark));
}

#[cfg(not(windows))]
fn tint_backdrop(_: &eframe::Frame, _: bool) {}

pub fn run() -> anyhow::Result<()> {
    run_window(None)
}
pub fn run_preview(
    found: crate::app::events::Discovered,
    metrics: std::path::PathBuf,
) -> anyhow::Result<()> {
    run_window(Some((found, metrics)))
}
fn run_window(
    preview: Option<(crate::app::events::Discovered, std::path::PathBuf)>,
) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: viewer::gpu::configuration(),
        viewport: ViewportBuilder::default()
            .with_title(identity::PRODUCT_NAME)
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
        identity::PRODUCT_NAME,
        options,
        Box::new(|cc| {
            let gpu = cc
                .wgpu_render_state
                .clone()
                .ok_or_else(|| anyhow::anyhow!("GPU renderer unavailable"))?;
            viewer::gpu::install_native(&cc.egui_ctx, gpu);
            let settings = if preview.is_some() {
                Settings::default()
            } else {
                Settings::load()
            };
            let backdrop = apply_backdrop(cc, settings.theme == Theme::Dark);
            let mut app = App::new(&cc.egui_ctx, backdrop, settings);
            if let Some((found, metrics)) = preview {
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
