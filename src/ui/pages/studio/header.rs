//! Quiet device status below the workspace navigation.
use super::{DevicePage, Mode, State};
use crate::core::Maturity;
use crate::ui::theme::{Role, color, label};
use crate::ui::widgets::Button;
use crate::ui::{icons::Icon, strings as s};
use eframe::egui::{Align, Layout, Ui};

pub(super) fn show(ui: &mut Ui, page: &DevicePage<'_>, state: &mut State) -> bool {
    let mut back = false;
    ui.horizontal(|ui| {
        if ui
            .add(Button::new(s::BACK).icon(Icon::ChevronLeft))
            .clicked()
        {
            back = state.back();
        }
        if state.mode != Mode::Overview {
            label(
                ui,
                if state.mode == Mode::Lighting {
                    s::LIGHTING
                } else {
                    s::STUDIO_ENCODER
                },
                Role::BodyStrong,
                color::TEXT_PRIMARY.get(),
            );
            label(
                ui,
                state.choice_label(),
                Role::Caption,
                color::TEXT_SECONDARY.get(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if state.inspector
                    && ui
                        .add(Button::icon_only(
                            Icon::ChevronRight,
                            s::STUDIO_CLOSE_SETTINGS,
                        ))
                        .clicked()
                {
                    state.inspector = false;
                }
            });
        }
    });
    if page
        .view
        .driver
        .is_some_and(|d| d.maturity == Maturity::Experimental)
    {
        label(
            ui,
            s::EXPERIMENTAL,
            Role::CaptionLight,
            color::STATUS_EXPERIMENTAL.get(),
        );
    }
    back
}
