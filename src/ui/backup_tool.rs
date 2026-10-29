//! A separate, minimal application. No Studio shell, device viewer, editor or telemetry scheduler.
use super::{backup_panel, icons, motion, strings as s, theme};
use crate::app::{Settings, Theme, backup_session::BackupSession, events::Discovered, identity};
use eframe::egui::{Context, IconData, ViewportBuilder};
use std::sync::Arc;

pub(crate) struct BackupApp {
    session: BackupSession,
    preview: bool,
}
impl BackupApp {
    pub(crate) fn new(ctx: &Context, found: Option<Discovered>) -> Self {
        let preview = found.is_some();
        let settings = if found.is_some() {
            Settings::default()
        } else {
            Settings::load()
        };
        theme::mode::set_blend(if settings.theme == Theme::Dark {
            0.0
        } else {
            1.0
        });
        theme::fonts::install(ctx);
        theme::style::apply(ctx);
        let wake = ctx.clone();
        let mut session = BackupSession::new(Arc::new(move || wake.request_repaint()));
        if let Some(found) = found {
            session.adopt_preview(found);
        } else {
            crate::diagnostics::logging::install_panic_hook();
            let _ = crate::diagnostics::logging::configure(settings.logging_enabled);
            crate::diagnostics::logging::event("application", "backup.start", "ok");
            session.refresh();
        }
        Self { session, preview }
    }
    pub(crate) fn frame(&mut self, ctx: &Context) {
        motion::begin_frame(ctx, crate::transport::animations_enabled());
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            eframe::egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(s::BACKUP_TITLE);
                ui.add_space(theme::space::M);
                backup_panel::show(ui, &mut self.session);
            });
        });
    }
}
impl eframe::App for BackupApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        if ctx.input(|input| input.viewport().close_requested()) && self.session.session().busy() {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::CancelClose);
        }
        self.frame(ctx);
    }
}

impl Drop for BackupApp {
    fn drop(&mut self) {
        if !self.preview {
            crate::diagnostics::logging::event("application", "backup.exit", "ok");
        }
    }
}

pub(super) fn run() -> anyhow::Result<()> {
    let title = identity::backup_title();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([640.0, 460.0])
            .with_min_inner_size([480.0, 380.0])
            .with_icon(Arc::new(IconData {
                rgba: icons::app_icon::rgba(64),
                width: 64,
                height: 64,
            })),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(|cc| Ok(Box::new(BackupApp::new(&cc.egui_ctx, None)))),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
