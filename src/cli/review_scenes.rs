//! Synthetic visual review scenes; device I/O is never used.
use super::review::Scene;
use super::review_state;
use crate::app::Theme;
use crate::ui::Route;
pub(super) const SCENES: &[Scene] = &[
    Scene {
        name: "catalog-placeholder",
        theme: Theme::Dark,
        height: 900.0,
        pointer: None,
        setup: |app| {
            let drivers: Vec<_> = crate::drivers::registry()
                .iter()
                .map(|d| d.info())
                .collect();
            if let Some(index) = crate::app::catalog::devices(&drivers)
                .iter()
                .position(|group| group.variants().is_empty())
            {
                app.open(Route::Preview(index));
            }
        },
    },
    Scene {
        name: "device-opening",
        theme: Theme::Dark,
        height: 900.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
        },
    },
    Scene {
        name: "device-reversing",
        theme: Theme::Dark,
        height: 900.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
        },
    },
    Scene {
        name: "catalog-connected",
        theme: Theme::Dark,
        height: 1400.0,
        pointer: None,
        setup: |app| {
            app.session_mut().adopt(review_state::devices());
        },
    },
    Scene {
        name: "catalog",
        theme: Theme::Dark,
        height: 900.0,
        pointer: None,
        setup: |_| {},
    },
    Scene {
        name: "catalog-device",
        theme: Theme::Dark,
        height: 900.0,
        pointer: None,
        setup: |app| app.open(Route::Preview(0)),
    },
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
