use crate::app::battery::{BatteryMode, BatteryView};
use crate::ui::pages::Route;
use eframe::egui::Context;

#[derive(Default)]
pub(super) struct BatterySchedule {
    pub(super) window_hidden: bool,
    previous: Option<(BatteryMode, BatteryView)>,
}
impl super::App {
    /// A tray integration can report hiding separately from native window minimization.
    pub fn set_window_hidden(&mut self, ctx: &Context, hidden: bool) {
        self.battery.window_hidden = hidden;
        ctx.request_repaint();
    }
    pub(super) fn update_batteries(&mut self, ctx: &Context, hidden: bool) {
        if self.watch.is_none() {
            return;
        }
        let view = if hidden {
            BatteryView::Background
        } else {
            match self.shell.route() {
                Route::Device(slot) => BatteryView::Device(slot),
                _ => BatteryView::Window,
            }
        };
        let mode = self.settings.battery_mode;
        let force = self.battery.previous.is_some_and(|(old_mode, old_view)| {
            old_mode != mode
                || (old_view == BatteryView::Background && view != BatteryView::Background)
        });
        self.battery.previous = Some((mode, view));
        if let Some(after) = self.session.poll_batteries(&self.events, mode, view, force) {
            ctx.request_repaint_after(after);
        }
    }
}
