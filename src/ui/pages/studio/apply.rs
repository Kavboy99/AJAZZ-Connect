//! One explicit native-program commit path shared by cards and the inspector.
use super::{DevicePage, Model, Route};
use crate::core::LightingSettings;
use crate::ui::{icons::Icon, strings as s, widgets::Button};
use eframe::egui::{Ui, UiBuilder};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Access {
    Ready,
    Busy,
    #[default]
    Offline,
    ReadOnly,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    Select,
    Apply,
}

#[derive(Default)]
pub(super) struct State {
    access: Access,
    request: Option<LightingSettings>,
    in_flight: Option<(Action, u8)>,
}

impl State {
    pub fn prepare(&mut self, model: &Model<'_>, page: &DevicePage<'_>) {
        self.access = match page.route {
            Route::Device(slot) => {
                model
                    .session
                    .entries()
                    .get(slot)
                    .map_or(Access::Offline, |entry| {
                        if entry.is_dormant() || entry.lighting_editor().is_none() {
                            Access::Offline
                        } else if entry.lighting_pending()
                            || entry.status() == Some(&crate::app::session::Status::Running)
                        {
                            Access::Busy
                        } else if entry.lighting().is_some_and(|t| t.writable) {
                            Access::Ready
                        } else {
                            Access::ReadOnly
                        }
                    })
            }
            _ => Access::Offline,
        };
        if self.access != Access::Busy {
            self.in_flight = None;
        }
    }

    fn request(&mut self, action: Action, settings: LightingSettings) -> bool {
        if self.access != Access::Ready || self.in_flight.is_some() {
            return false;
        }
        self.request = Some(settings);
        self.in_flight = Some((action, settings.preset));
        true
    }

    pub fn button(&mut self, ui: &mut Ui, action: Action, settings: LightingSettings) -> bool {
        let text = match action {
            Action::Select => s::LIGHTING_SELECT,
            Action::Apply => s::LIGHTING_APPLY_PROGRAM,
        };
        let reason = match self.access {
            Access::Ready => s::LIGHTING_SELECT_HINT,
            Access::Busy => s::LIGHTING_RUNNING,
            Access::Offline => s::LIGHTING_CONNECT_USB,
            Access::ReadOnly => s::LIGHTING_WRITE_BLOCKED,
        };
        if ui
            .scope_builder(
                UiBuilder::new().id(ui.make_persistent_id((
                    "native-commit",
                    text,
                    settings.preset,
                ))),
                |ui| {
                    ui.add(
                        Button::new(text)
                            .primary()
                            .icon(Icon::Success)
                            .enabled(matches!(self.access, Access::Ready | Access::Busy))
                            .passive(self.access == Access::Busy || self.in_flight.is_some())
                            .busy(self.in_flight == Some((action, settings.preset)))
                            .tooltip(reason),
                    )
                },
            )
            .inner
            .clicked()
        {
            return self.request(action, settings);
        }
        false
    }

    pub fn dispatch(&mut self, model: &mut Model<'_>, page: &DevicePage<'_>) {
        if let Some(settings) = self.request.take()
            && let Route::Device(slot) = page.route
            && !model.session.commit_lighting(slot, settings, model.events)
        {
            self.in_flight = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_keeps_apply_and_neighbor_geometry_stable() {
        use eframe::egui::{CentralPanel, Context};
        let ctx = Context::default();
        crate::ui::theme::fonts::install(&ctx);
        let settings = LightingSettings {
            preset: 2,
            brightness: 3,
            speed: 1,
            color: None,
        };
        let mut state = State {
            access: Access::Ready,
            ..Default::default()
        };
        let mut rects = Vec::new();
        for busy in [false, true, false] {
            state.in_flight = busy.then_some((Action::Apply, settings.preset));
            state.access = if busy { Access::Busy } else { Access::Ready };
            let _ = ctx.run(Default::default(), |ctx| {
                crate::ui::motion::begin_frame(ctx, true);
                CentralPanel::default().show(ctx, |ui| {
                    rects.push(
                        ui.horizontal(|ui| {
                            state.button(ui, Action::Apply, settings);
                            ui.add(Button::new("Neighbor"));
                        })
                        .response
                        .rect,
                    );
                });
            });
        }
        assert_eq!(rects[0], rects[1]);
        assert_eq!(rects[1], rects[2]);
    }

    #[test]
    fn commits_capture_the_clicked_draft_and_block_duplicates_and_unavailable_devices() {
        let settings = LightingSettings {
            preset: 7,
            speed: 2,
            brightness: 3,
            color: None,
        };
        for access in [Access::Offline, Access::ReadOnly, Access::Busy] {
            let mut state = State {
                access,
                ..Default::default()
            };
            assert!(!state.request(Action::Apply, settings));
            assert!(state.request.is_none());
        }
        let mut state = State {
            access: Access::Ready,
            ..Default::default()
        };
        assert!(state.request(Action::Apply, settings));
        assert!(!state.request(
            Action::Select,
            LightingSettings {
                preset: 8,
                ..settings
            }
        ));
        assert_eq!(state.request, Some(settings));
    }
}
