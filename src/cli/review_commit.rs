//! Pointer-driven explicit commits against an in-memory device, never real HID.
use super::review::Scene;
use crate::app::{Theme, session::Status};
use crate::ui::{App, Route, strings as s};
use anyhow::{Result, ensure};
use eframe::egui::{Event, Modifiers, PointerButton, Shape, epaint::ClippedShape};

fn success(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::commit_devices(true));
    app.open(Route::Device(0));
}
fn failure(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::commit_devices(false));
    app.open(Route::Device(0));
}
pub(super) fn scenes() -> Vec<Scene> {
    [
        ("studio-commit-preview", Theme::Dark, 840.0),
        ("studio-commit-select", Theme::Dark, 840.0),
        ("studio-commit-settings-short", Theme::Dark, 520.0),
        ("studio-commit-settings-light", Theme::Light, 840.0),
        ("studio-commit-settings-brightness", Theme::Dark, 840.0),
        ("studio-commit-settings-speed", Theme::Dark, 840.0),
        ("studio-commit-failure", Theme::Dark, 840.0),
    ]
    .into_iter()
    .map(|(name, theme, height)| Scene {
        name,
        theme,
        height,
        pointer: None,
        setup: if name.ends_with("failure") {
            failure
        } else {
            success
        },
    })
    .collect()
}
pub(super) fn input(
    name: &str,
    frame: usize,
    shapes: &[ClippedShape],
) -> Result<Option<Vec<Event>>> {
    if !name.starts_with("studio-commit-") {
        return Ok(None);
    }
    let click =
        |text, index, pressed| super::review_studio::click(shapes, text, index, pressed).map(Some);
    if frame == 52 || frame == 53 {
        return click(
            s::lighting_effect(crate::core::LightingEffect::Breathing),
            0,
            frame == 52,
        );
    }
    if (name.ends_with("brightness") || name.ends_with("speed")) && (frame == 90 || frame == 91) {
        let text = if name.ends_with("brightness") {
            s::STUDIO_BRIGHTNESS
        } else {
            s::LIGHTING_SPEED
        };
        let mut found = Vec::new();
        for shape in shapes {
            super::review_studio::labels(&shape.shape, text, &mut found);
        }
        let row = found
            .last()
            .ok_or_else(|| anyhow::anyhow!("missing slider label: {text}"))?;
        let pos = shapes
            .iter()
            .find_map(|s| match &s.shape {
                Shape::Rect(rail)
                    if rail.rect.height() <= 4.0
                        && rail.rect.width() > 20.0
                        && rail.rect.left() > row.right()
                        && (rail.rect.center().y - row.center().y).abs() < 4.0 =>
                {
                    Some(rail.rect.right_center())
                }
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("missing slider rail: {text}"))?;
        return Ok(Some(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: frame == 90,
                modifiers: Modifiers::NONE,
            },
        ]));
    }
    if frame == 70 || frame == 71 {
        return if name.contains("settings") {
            click(s::STUDIO_SETTINGS, 1, frame == 70)
        } else if !name.ends_with("preview") {
            click(s::LIGHTING_SELECT, 0, frame == 70)
        } else {
            Ok(Some(Vec::new()))
        };
    }
    if name.contains("settings") && (frame == 100 || frame == 101) {
        return click(s::LIGHTING_APPLY_PROGRAM, 0, frame == 100);
    }
    Ok(None)
}
pub(super) fn verify(name: &str, app: &mut App) -> Result<()> {
    if !name.starts_with("studio-commit-") {
        return Ok(());
    }
    let entry = &app.session_mut().entries()[0];
    let expected = u8::from(!name.ends_with("preview") && !name.ends_with("failure"));
    ensure!(
        entry
            .lighting_editor()
            .is_some_and(|e| e.current == expected),
        "{name}: incorrect committed mode"
    );
    if name.ends_with("brightness") || name.ends_with("speed") {
        let preset = entry
            .lighting_editor()
            .and_then(|e| e.presets.iter().find(|p| p.saved.preset == 1))
            .ok_or_else(|| anyhow::anyhow!("missing committed Breathing preset"))?;
        let expected = if name.ends_with("brightness") {
            (4, 2)
        } else {
            (3, 4)
        };
        ensure!(
            (preset.saved.brightness, preset.saved.speed) == expected,
            "{name}: sliders committed the wrong fields: {:?}",
            preset.saved
        );
    }
    if name.ends_with("preview") {
        ensure!(entry.status().is_none(), "preview dispatched a write");
    } else if name.ends_with("failure") {
        ensure!(
            matches!(entry.status(), Some(Status::Failed(_))),
            "failure not reported"
        );
    } else {
        ensure!(
            matches!(
                entry.status(),
                Some(Status::Done(crate::core::Outcome::Confirmed))
            ),
            "commit not confirmed"
        );
    }
    Ok(())
}
