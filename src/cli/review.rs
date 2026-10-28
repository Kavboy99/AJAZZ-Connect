//! `--review DIRECTORY`: renders interface states offscreen to PNG files for design review. Uses
//! synthetic data only; no device, socket or window is opened.

use super::raster::{Canvas, update_textures};
use super::review_state;
use crate::app::{Settings, Theme};
use crate::ui::{App, Route, preload_models};
use anyhow::Result;
use eframe::egui::{Context, Event, Pos2, RawInput, Rect, pos2, vec2};
use std::collections::BTreeMap;
use std::path::Path;

/// Enough 60 Hz frames for every spring to settle.
const SETTLE_FRAMES: usize = 90;

/// Every scene is rendered at each width: the minimum window, the default and a wide monitor
/// (`docs/DESIGN.md`, section 5.1).
const WIDTHS: [(&str, f32); 3] = [("narrow", 720.0), ("default", 1040.0), ("wide", 1600.0)];

struct Scene {
    name: &'static str,
    theme: Theme,
    height: f32,
    /// Where the pointer rests, as a share of the window size.
    pointer: Option<(f32, f32)>,
    setup: fn(&mut App),
}

const SCENES: &[Scene] = &[
    Scene {
        name: "devices-empty",
        theme: Theme::Dark,
        height: 720.0,
        pointer: None,
        setup: |_| {},
    },
    Scene {
        name: "devices",
        theme: Theme::Dark,
        height: 720.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
        },
    },
    Scene {
        name: "device",
        theme: Theme::Dark,
        height: 820.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
            app.open(Route::Device(0));
        },
    },
    Scene {
        name: "device-experimental",
        theme: Theme::Dark,
        height: 1000.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
            app.open(Route::Device(2));
        },
    },
    Scene {
        name: "device-dormant",
        theme: Theme::Dark,
        height: 820.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
            app.open(Route::Device(3));
        },
    },
    Scene {
        name: "diagnostics",
        theme: Theme::Dark,
        height: 900.0,
        pointer: None,
        setup: |app| {
            app.diagnostics_mut()
                .finish_probe(Ok(review_state::report()));
            app.open(Route::Diagnostics);
        },
    },
    Scene {
        name: "settings",
        theme: Theme::Dark,
        height: 820.0,
        pointer: None,
        setup: |app| app.open(Route::Settings),
    },
    Scene {
        name: "devices-hover",
        theme: Theme::Dark,
        height: 720.0,
        pointer: Some((0.42, 0.36)),
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
        },
    },
    Scene {
        name: "devices-light",
        theme: Theme::Light,
        height: 720.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
        },
    },
    Scene {
        name: "device-light",
        theme: Theme::Light,
        height: 820.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
            app.open(Route::Device(2));
        },
    },
    Scene {
        name: "diagnostics-light",
        theme: Theme::Light,
        height: 900.0,
        pointer: None,
        setup: |app| {
            app.diagnostics_mut()
                .finish_probe(Ok(review_state::report()));
            app.open(Route::Diagnostics);
        },
    },
];

fn render(scene: &Scene, (label, width): (&str, f32), out: &Path) -> Result<()> {
    let ctx = Context::default();
    let mut app = App::new(
        &ctx,
        false,
        Settings {
            theme: scene.theme,
            ..Settings::default()
        },
    );
    (scene.setup)(&mut app);
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(width, scene.height));
    let mut textures = BTreeMap::new();
    let mut last = None;
    let pointer = scene
        .pointer
        .map(|(x, y)| Event::PointerMoved(pos2(x * width, y * scene.height)));
    for frame in 0..SETTLE_FRAMES {
        let input = RawInput {
            screen_rect: Some(rect),
            events: pointer.iter().cloned().collect(),
            time: Some(frame as f64 / 60.0),
            predicted_dt: 1.0 / 60.0,
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| app.frame(ctx));
        update_textures(&mut textures, &output.textures_delta);
        last = Some(output);
    }
    let output = last.ok_or_else(|| anyhow::anyhow!("no frame"))?;
    let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
    let mut canvas = Canvas::new(width as usize, scene.height as usize);
    canvas.draw(&primitives, &textures);
    canvas.save(&out.join(format!("{}-{label}.png", scene.name)))
}

pub fn run(directory: &Path) -> Result<()> {
    preload_models();
    std::fs::create_dir_all(directory)?;
    for scene in SCENES {
        for size in WIDTHS {
            render(scene, size, directory)?;
        }
    }
    Ok(())
}
