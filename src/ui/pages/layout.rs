//! Building blocks every page shares: the page header, section titles and label/value rows.

use crate::ui::icons::Icon;
use crate::ui::theme::{Role, breakpoint, color, label, rich, size, space};
use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{Align, Color32, Label, Layout, Ui};

/// Title on the left, actions on the right, an optional lead line below.
pub fn header(ui: &mut Ui, title: &str, lead: Option<&str>, actions: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add(Label::new(rich(title, Role::Title, color::TEXT_PRIMARY.get())).truncate());
        ui.with_layout(Layout::right_to_left(Align::Center), actions);
    });
    if let Some(lead) = lead {
        ui.add_space(space::XS);
        ui.scope(|ui| {
            ui.set_max_width(size::READABLE_WIDTH.min(ui.available_width()));
            label(ui, lead, Role::Body, color::TEXT_SECONDARY.get());
        });
    }
    ui.add_space(space::XL);
}

pub fn section_title(ui: &mut Ui, icon: Icon, title: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::S;
        icon.show(ui, size::ICON_LARGE, color::TEXT_PRIMARY.get());
        ui.label(rich(title, Role::Subtitle, color::TEXT_PRIMARY.get()));
    });
    ui.add_space(space::S);
}

/// Secondary text that wraps at a readable width.
pub fn hint(ui: &mut Ui, text: &str) {
    ui.scope(|ui| {
        ui.set_max_width(size::READABLE_WIDTH.min(ui.available_width()));
        label(ui, text, Role::Body, color::TEXT_SECONDARY.get());
    });
}

/// Text runs of different roles on one baseline.
pub fn spans(ui: &mut Ui, runs: &[(&str, Role, Color32)]) {
    let mut job = LayoutJob::default();
    for (i, (text, role, color)) in runs.iter().enumerate() {
        let format = TextFormat {
            font_id: role.font(),
            color: *color,
            valign: Align::BOTTOM,
            ..Default::default()
        };
        if i > 0 {
            job.append(" · ", 0.0, format.clone());
        }
        job.append(text, 0.0, format);
    }
    ui.add(Label::new(job).wrap());
}

pub fn gap(ui: &mut Ui) {
    ui.add_space(size::CARD_GAP);
}

/// Two parts of a page side by side when the content is wide, stacked otherwise. `share` is the
/// first column's part of the width. `state` is lent to whichever part is drawing.
pub fn columns<T>(
    ui: &mut Ui,
    share: f32,
    state: &mut T,
    first: impl FnOnce(&mut Ui, &mut T),
    second: impl FnOnce(&mut Ui, &mut T),
) {
    let width = ui.available_width();
    if settled_width(ui) < breakpoint::PAGE_COLUMNS {
        first(ui, state);
        gap(ui);
        second(ui, state);
        return;
    }
    let left = ((width - size::CARD_GAP) * share).floor();
    let right = width - size::CARD_GAP - left;
    let column = |ui: &mut Ui, width: f32, add: &mut dyn FnMut(&mut Ui)| {
        ui.allocate_ui_with_layout(
            eframe::egui::vec2(width, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(width);
                add(ui);
            },
        );
    };
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = size::CARD_GAP;
        let mut first = Some(first);
        let mut second = Some(second);
        column(ui, left, &mut |ui| {
            if let Some(f) = first.take() {
                f(ui, state);
            }
        });
        column(ui, right, &mut |ui| {
            if let Some(f) = second.take() {
                f(ui, state);
            }
        });
    });
}

fn settling_id() -> eframe::egui::Id {
    eframe::egui::Id::new("layout-settling")
}

/// The shell reports how much wider the content will be once the navigation pane stops moving.
pub fn set_settling(ctx: &eframe::egui::Context, delta: f32) {
    ctx.data_mut(|d| d.insert_temp(settling_id(), delta));
}

/// The width this part will have when the pane settles. Breakpoints are decided on it, so a moving
/// pane never flips a layout halfway through its motion.
pub fn settled_width(ui: &Ui) -> f32 {
    ui.available_width()
        + ui.ctx()
            .data(|d| d.get_temp::<f32>(settling_id()).unwrap_or(0.0))
}
