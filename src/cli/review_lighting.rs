//! Regression scenes follow the real catalog and a live device without writable lighting.
use super::review::Scene;
use crate::app::Theme;
use crate::core::DriverInfo;
use crate::ui::{App, Route, strings as s};
use anyhow::Result;
use eframe::egui::epaint::ClippedShape;
use eframe::egui::{Event, Rect};

fn driver() -> Option<DriverInfo> {
    crate::drivers::registry()
        .iter()
        .find_map(|d| d.lighting_preview().map(|_| d.info()))
}

fn connection(app: &mut App) {
    if let Some(driver) = driver() {
        app.session_mut()
            .adopt(super::review_state::connection_devices(driver));
        app.open(Route::Device(0));
    }
}

fn catalog(app: &mut App) {
    let Some(driver) = driver() else { return };
    let drivers: Vec<_> = crate::drivers::registry()
        .iter()
        .map(|d| d.info())
        .collect();
    if let Some(index) = crate::app::catalog::devices(&drivers)
        .iter()
        .position(|group| group.variants().iter().any(|d| d.driver.id == driver.id))
    {
        app.open(Route::Preview(index));
    }
}

pub(super) fn scenes() -> Vec<Scene> {
    if driver().is_none() {
        return Vec::new();
    }
    [
        ("studio-connection-settings", connection as fn(&mut App)),
        ("studio-catalog-settings", catalog as fn(&mut App)),
    ]
    .into_iter()
    .map(|(name, setup)| Scene {
        name,
        setup,
        theme: Theme::Dark,
        height: 840.0,
        pointer: None,
    })
    .collect()
}

pub(super) fn input(
    name: &str,
    frame: usize,
    shapes: &[ClippedShape],
) -> Result<Option<Vec<Event>>> {
    if name == "studio-catalog-settings"
        && (frame == 4 || frame == 5)
        && let Some(driver) = driver()
    {
        return super::review_studio::click(shapes, driver.name, 0, frame == 4).map(Some);
    }
    Ok(None)
}

pub(super) fn verify(name: &str, shapes: &[ClippedShape], screen: Rect) -> Result<()> {
    if !matches!(
        name,
        "studio-connection-settings" | "studio-catalog-settings"
    ) {
        return Ok(());
    }
    for text in [
        s::STUDIO_BRIGHTNESS,
        s::LIGHTING_APPLY_PROGRAM,
        if name.contains("catalog") {
            s::LIGHTING_OFFLINE_CONTROLS
        } else {
            s::LIGHTING_CONNECT_USB
        },
    ] {
        let mut found = Vec::new();
        for shape in shapes {
            super::review_studio::labels(&shape.shape, text, &mut found);
        }
        anyhow::ensure!(
            found.iter().any(|r| screen.contains_rect(*r)),
            "{name}: missing visible lighting control: {text}"
        );
    }
    Ok(())
}
