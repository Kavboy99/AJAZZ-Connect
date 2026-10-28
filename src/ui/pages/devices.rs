use super::layout::{gap, header, hint, settled_width};
use super::{Model, Navigate, Route};
use crate::app::events::DriverNote;
use crate::app::session::Entry;
use crate::core::Maturity;
use crate::ui::icons::Icon;
use crate::ui::motion::Style;
use crate::ui::strings as s;
use crate::ui::theme::{Role, color, label, size, space};
use crate::ui::widgets::{Button, TileContent, Tone, card, device_tile, expander};
use eframe::egui::{Id, Ui};

pub fn show(ui: &mut Ui, model: &mut Model<'_>) -> Navigate {
    let mut navigate = None;
    let discovering = model.session.discovering();
    let busy = model.session.busy();
    let passive = !ui.is_enabled() && crate::ui::device_reveal::moving(ui.ctx());
    let mut find = false;
    header(ui, s::DEVICES_TITLE, Some(s::DEVICES_LEAD), |ui| {
        let text = if discovering {
            s::DEVICES_FINDING
        } else {
            s::DEVICES_FIND
        };
        find = ui
            .add(
                Button::new(text)
                    .icon(Icon::Search)
                    .tooltip(s::DEVICES_FIND_HINT)
                    .busy(discovering)
                    .passive(passive)
                    .enabled(!busy),
            )
            .clicked();
    });
    if find {
        model
            .session
            .discover(model.events, model.settings.experimental_drivers);
    }

    let entries = model.session.entries();
    let catalog = model.settings.developer_mode && model.settings.show_all_devices;
    if entries.is_empty() && !catalog {
        card(ui, |ui| {
            if model.session.searched() {
                label(
                    ui,
                    s::DEVICES_NONE,
                    Role::BodyStrong,
                    color::TEXT_PRIMARY.get(),
                );
                ui.add_space(space::XS);
                hint(ui, s::DEVICES_NONE_HINT);
                ui.add_space(space::M);
                if ui
                    .add(Button::new(s::DEVICES_OPEN_DIAGNOSTICS).icon(Icon::Diagnostics))
                    .clicked()
                {
                    navigate = Some((Route::Diagnostics, Style::Entrance));
                }
            } else {
                hint(ui, s::DEVICES_NEVER_SEARCHED);
            }
        });
    } else {
        label(
            ui,
            s::found(entries.len()),
            Role::Caption,
            color::TEXT_SECONDARY.get(),
        );
        ui.add_space(space::S);
        if let Some(slot) = grid(ui, entries) {
            navigate = Some((Route::Device(slot), Style::DrillIn));
        }
    }

    if catalog {
        gap(ui);
        hint(ui, s::SETTINGS_ALL_DEVICES);
        if let Some(target) = super::preview::grid(ui, model) {
            navigate = Some(target);
        }
    }
    let notes = model.session.notes();
    if !notes.is_empty() {
        gap(ui);
        let title = format!("{} ({})", s::DEVICES_NOTES, notes.len());
        expander(ui, Id::new("discovery-notes"), &title, false, |ui| {
            for note in notes {
                show_note(ui, note);
                ui.add_space(space::S);
            }
        });
    }
    navigate
}

/// Square tiles in as many columns as fit (`docs/DESIGN.md`, section 8.1). Returns the clicked slot.
fn grid(ui: &mut Ui, entries: &[Entry]) -> Option<usize> {
    let gap = size::CARD_GAP;
    let width = ui.available_width();
    // The column count is a breakpoint: decided on the settled width, so a moving pane never
    // flips it halfway. The tile side follows the window directly.
    let columns = ((settled_width(ui) + gap) / (size::DEVICE_TILE_MIN + gap))
        .floor()
        .max(1.0) as usize;
    let side = ((width - gap * (columns - 1) as f32) / columns as f32).floor();
    let mut clicked = None;
    for (row, chunk) in entries.chunks(columns).enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (i, entry) in chunk.iter().enumerate() {
                let slot = row * columns + i;
                if tile(ui, slot, side, entry).clicked() {
                    clicked = Some(slot);
                }
            }
        });
        ui.add_space(gap);
    }
    clicked
}

fn tile(ui: &mut Ui, slot: usize, side: f32, entry: &Entry) -> eframe::egui::Response {
    let d = entry.descriptor();
    let mut caption = vec![s::kind(d.kind).to_owned()];
    caption.extend(d.vendor.clone());
    caption.push(d.driver.name.to_owned());
    let experimental = [(Tone::Caution, s::EXPERIMENTAL)];
    let dormant = [(Tone::Caution, s::DEVICE_DORMANT)];
    let badges: &[(Tone, &str)] = if entry.is_dormant() {
        &dormant
    } else if d.driver.maturity == Maturity::Experimental {
        &experimental
    } else {
        &[]
    };
    let response = device_tile(
        ui,
        Id::new(("device-tile", slot)),
        side,
        &TileContent {
            name: &d.name,
            caption: &caption.join(" · "),
            icon: Icon::for_kind(d.kind),
            visual: d.visual.as_deref(),
            busy: entry.busy(),
            badges,
        },
    );
    crate::ui::device_reveal::remember(ui.ctx(), Route::Device(slot), response.rect);
    response
}

fn show_note(ui: &mut Ui, note: &DriverNote) {
    let mut title = note.driver.map_or("HID", |d| d.name).to_owned();
    if let Some(subject) = &note.subject {
        title = format!("{title} · {subject}");
    }
    label(ui, title, Role::BodyStrong, color::TEXT_PRIMARY.get());
    label(
        ui,
        s::error(&note.error),
        Role::Body,
        color::TEXT_SECONDARY.get(),
    );
    if let Some(requirement) = note.driver.and_then(|d| d.requirement) {
        label(
            ui,
            s::requirement(requirement),
            Role::Caption,
            color::TEXT_TERTIARY.get(),
        );
    }
}
