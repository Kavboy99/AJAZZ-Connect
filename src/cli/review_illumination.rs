//! Matched material, luminous-source and finite-wake scenes, without hardware access.
use super::review::Scene;
use crate::app::Theme;
use crate::ui::{App, Route};
use eframe::egui::{Event, Key, Modifiers};

fn overview(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::native_devices_for(0));
    app.open(Route::Device(0));
}
fn flash(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::native_devices_for(3));
    app.open(Route::Device(0));
}
fn ripples(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::native_devices_for(6));
    app.open(Route::Device(0));
}

pub(super) fn scenes() -> Vec<Scene> {
    [
        (
            "studio-depth-overview-dark",
            Theme::Dark,
            840.0,
            overview as fn(&mut App),
        ),
        ("studio-depth-overview-light", Theme::Light, 840.0, overview),
        ("studio-depth-solid-dark", Theme::Dark, 840.0, overview),
        ("studio-depth-solid-light", Theme::Light, 840.0, overview),
        ("studio-depth-flash-dark", Theme::Dark, 840.0, flash),
        ("studio-depth-flash-light", Theme::Light, 840.0, flash),
        ("studio-depth-ripples-short", Theme::Dark, 520.0, ripples),
        (
            "studio-depth-ripples-short-light",
            Theme::Light,
            520.0,
            ripples,
        ),
    ]
    .into_iter()
    .map(|(name, theme, height, setup)| Scene {
        name,
        theme,
        height,
        setup,
        pointer: None,
    })
    .collect()
}

pub(super) fn is_overview(name: &str) -> bool {
    name.starts_with("studio-depth-overview")
}

pub(super) fn input(name: &str, frame: usize) -> Option<Vec<Event>> {
    if is_overview(name) {
        return Some(Vec::new());
    }
    if name.starts_with("studio-depth-")
        && (name.contains("flash") || name.contains("ripples"))
        && [58, 60].contains(&frame)
    {
        return Some(vec![Event::Key {
            key: Key::A,
            physical_key: Some(Key::A),
            pressed: frame == 58,
            repeat: false,
            modifiers: Modifiers::NONE,
        }]);
    }
    None
}
