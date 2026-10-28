//! Real pointer/keyboard input through the offline workspace, including reversal and idle.
use super::review::Scene;
use crate::app::Theme;
use crate::ui::{App, Route, strings as s};
use anyhow::{Result, ensure};
use eframe::egui::epaint::ClippedShape;
use eframe::egui::{Event, Key, Modifiers, PointerButton, Rect, Shape, pos2};

fn setup(app: &mut App) {
    app.session_mut().adopt(super::review_state::devices());
    app.open(Route::Device(0));
}

fn setup_native(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::native_devices());
    app.open(Route::Device(0));
}

fn setup_rain(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::native_devices_for(4));
    app.open(Route::Device(0));
}
fn setup_ripples(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::native_devices_for(6));
    app.open(Route::Device(0));
}
fn setup_snake(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::native_devices_for(9));
    app.open(Route::Device(0));
}
fn setup_windmill(app: &mut App) {
    app.session_mut()
        .adopt(super::review_state::native_devices_for(14));
    app.open(Route::Device(0));
}

pub(super) fn scenes() -> Vec<Scene> {
    [
        ("studio-lighting", Theme::Dark, 840.0),
        ("studio-native-presets", Theme::Dark, 840.0),
        ("studio-native-rain", Theme::Dark, 840.0),
        ("studio-native-ripples", Theme::Dark, 840.0),
        ("studio-native-snake", Theme::Dark, 840.0),
        ("studio-native-windmill", Theme::Dark, 840.0),
        ("studio-native-settings", Theme::Dark, 840.0),
        ("studio-native-settings-short", Theme::Dark, 520.0),
        ("studio-native-scroll", Theme::Dark, 840.0),
        ("studio-wave", Theme::Dark, 840.0),
        ("studio-reactive", Theme::Dark, 840.0),
        ("studio-settings", Theme::Dark, 840.0),
        ("studio-settings-light", Theme::Light, 840.0),
        ("studio-encoder", Theme::Dark, 840.0),
        ("studio-encoder-settings", Theme::Dark, 840.0),
        ("studio-reverse", Theme::Dark, 840.0),
        ("studio-short", Theme::Dark, 520.0),
        ("studio-palette", Theme::Dark, 840.0),
        ("studio-close", Theme::Dark, 840.0),
        ("studio-back", Theme::Dark, 840.0),
        ("studio-lighting-pointer", Theme::Dark, 840.0),
        ("studio-encoder-pointer", Theme::Dark, 840.0),
    ]
    .into_iter()
    .map(|(name, theme, height)| Scene {
        name,
        theme,
        height,
        pointer: None,
        setup: if name == "studio-native-rain" {
            setup_rain
        } else if name == "studio-native-ripples" {
            setup_ripples
        } else if name == "studio-native-snake" {
            setup_snake
        } else if name == "studio-native-windmill" {
            setup_windmill
        } else if name.starts_with("studio-native-") {
            setup_native
        } else {
            setup
        },
    })
    .collect()
}

pub(super) fn labels(shape: &Shape, text: &str, found: &mut Vec<Rect>) {
    match shape {
        Shape::Text(t) if t.galley.job.text == text => {
            found.push(Rect::from_min_size(t.pos, t.galley.size()))
        }
        Shape::Vec(shapes) => {
            for shape in shapes {
                labels(shape, text, found);
            }
        }
        _ => {}
    }
}

pub(super) fn click(
    shapes: &[ClippedShape],
    text: &str,
    nth: usize,
    pressed: bool,
) -> Result<Vec<Event>> {
    let mut found = Vec::new();
    for shape in shapes {
        labels(&shape.shape, text, &mut found);
    }
    let rect = found
        .get(nth)
        .ok_or_else(|| anyhow::anyhow!("workspace control missing: {text} #{nth}"))?;
    let pos = rect.center();
    Ok(vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        },
    ])
}

pub(super) fn input(
    name: &str,
    frame: usize,
    shapes: &[ClippedShape],
    screen: Rect,
) -> Result<Vec<Event>> {
    if let Some(events) = super::review_lighting::input(name, frame, shapes)? {
        return Ok(events);
    }
    if name == "studio-native-scroll" && frame == 80 {
        return Ok(vec![
            Event::PointerMoved(pos2(screen.center().x, screen.bottom() - 100.0)),
            Event::MouseWheel {
                unit: eframe::egui::MouseWheelUnit::Point,
                delta: eframe::egui::vec2(0.0, -5000.0),
                modifiers: Modifiers::NONE,
            },
        ]);
    }
    let encoder = name.contains("encoder");
    if frame == 24 || frame == 25 {
        return click(
            shapes,
            if encoder {
                s::STUDIO_ENCODER
            } else {
                s::STUDIO_LIGHTING
            },
            0,
            frame == 24,
        );
    }
    if (name == "studio-wave" || name == "studio-reactive" || name == "studio-palette")
        && (frame == 52 || frame == 53)
    {
        return click(
            shapes,
            if name == "studio-wave" || name == "studio-palette" {
                s::STUDIO_WAVE
            } else {
                s::STUDIO_REACTIVE
            },
            0,
            frame == 52,
        );
    }
    if (name.contains("settings")
        || name == "studio-short"
        || name == "studio-reverse"
        || name == "studio-palette")
        && (frame == 70 || frame == 71)
    {
        return click(
            shapes,
            s::STUDIO_SETTINGS,
            usize::from(name == "studio-palette"),
            frame == 70,
        );
    }
    if name == "studio-back" && (frame == 70 || frame == 71) {
        return click(shapes, s::BACK, 0, frame == 70);
    }
    if name == "studio-close" && (frame == 70 || frame == 71) {
        let pos = pos2(screen.right() - 52.0, screen.bottom() - 216.0);
        return Ok(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: frame == 70,
                modifiers: Modifiers::NONE,
            },
        ]);
    }
    if name == "studio-reverse" && (frame == 76 || frame == 80) {
        return Ok(vec![key(Key::Escape, true)]);
    }
    if name == "studio-reverse" && (frame == 77 || frame == 81) {
        return Ok(vec![key(Key::Escape, false)]);
    }
    if name == "studio-reactive" && (frame == 118 || frame == 128) {
        return Ok(vec![
            key(Key::A, frame == 118),
            key(Key::Space, frame == 118),
        ]);
    }
    if frame == 100 {
        if name.ends_with("-pointer") {
            return Ok(vec![Event::PointerMoved(pos2(
                screen.width() * 0.25,
                screen.height() * 0.25,
            ))]);
        }
        return Ok(vec![Event::PointerMoved(pos2(3.0, 3.0))]);
    }
    Ok(Vec::new())
}

fn key(key: Key, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

pub(super) fn verify(name: &str, shapes: &[ClippedShape], screen: Rect) -> Result<()> {
    super::review_lighting::verify(name, shapes, screen)?;
    if name == "studio-native-scroll" || name == "studio-native-settings" {
        let target = if name.ends_with("scroll") {
            s::lighting_effect(crate::core::LightingEffect::Off)
        } else {
            s::STUDIO_BRIGHTNESS
        };
        let mut found = Vec::new();
        for shape in shapes {
            labels(&shape.shape, target, &mut found);
        }
        ensure!(
            found.iter().any(|r| screen.contains_rect(*r)),
            "{name}: native control missing: {target}"
        );
    }
    if name == "studio-wave" || name == "studio-reactive" {
        let text = if name == "studio-wave" {
            s::STUDIO_WAVE
        } else {
            s::STUDIO_REACTIVE
        };
        let mut found = Vec::new();
        for shape in shapes {
            labels(&shape.shape, text, &mut found);
        }
        ensure!(
            found.len() == 2,
            "{name}: effect was not selected through real pointer input"
        );
    }
    let mut labels_found = Vec::new();
    let label = if name == "studio-reverse" || name == "studio-close" {
        s::STUDIO_LIGHTING
    } else if name == "studio-back" {
        s::DEVICES_TITLE
    } else {
        s::STUDIO_SETTINGS
    };
    for shape in shapes {
        labels(&shape.shape, label, &mut labels_found);
    }
    ensure!(
        !labels_found.is_empty(),
        "{name}: expected control not visible"
    );
    ensure!(
        if name.starts_with("studio-native-")
            || name == "studio-connection-settings"
            || name == "studio-catalog-settings"
        {
            // A scrolling catalog intentionally lays out clipped offscreen cards.
            labels_found.iter().any(|r| screen.contains_rect(*r))
        } else {
            labels_found.iter().all(|r| screen.contains_rect(*r))
        },
        "{name}: controls outside viewport"
    );
    Ok(())
}
