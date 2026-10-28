//! Device models built into the executable. A profile names a model by id (`visual = "..."`);
//! the file is parsed once, on a short-lived thread, the first time a tile asks for it.

use super::glb::{self, Scene};
use eframe::egui::Context;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

const MODELS: &[(&str, &[u8])] = &[
    (
        "ak820-max",
        include_bytes!("../../../assets/models/ak820-max.glb"),
    ),
    ("aj159", include_bytes!("../../../assets/models/aj159.glb")),
];

#[cfg(test)]
pub(super) fn model_bytes(id: &str) -> Option<&'static [u8]> {
    MODELS
        .iter()
        .find(|(key, _)| *key == id)
        .map(|(_, bytes)| *bytes)
}

#[derive(Clone)]
enum Load {
    Loading,
    Ready(Arc<Scene>),
    Failed,
}

pub enum Model {
    /// No such model, or its file is broken: draw the kind icon instead.
    Missing,
    Loading,
    Ready(Arc<Scene>),
}

fn loads() -> &'static Mutex<HashMap<&'static str, Load>> {
    static LOADS: OnceLock<Mutex<HashMap<&'static str, Load>>> = OnceLock::new();
    LOADS.get_or_init(Mutex::default)
}

/// Parses every built-in model now, on this thread. For `--review`, which renders a fixed number
/// of frames and cannot wait for a loader thread.
pub fn preload() {
    if let Ok(mut state) = loads().lock() {
        for (key, bytes) in MODELS {
            let load = glb::parse(bytes).map_or(Load::Failed, |mut s| {
                super::led::prepare(&mut s, key);
                super::keycaps::prepare(&mut s, key);
                Load::Ready(Arc::new(s))
            });
            state.insert(key, load);
        }
    }
}

pub fn model(ctx: &Context, id: &str) -> Model {
    let Some(&(key, bytes)) = MODELS.iter().find(|(key, _)| *key == id) else {
        return Model::Missing;
    };
    let Ok(mut state) = loads().lock() else {
        return Model::Missing;
    };
    match state.get(key).cloned() {
        Some(Load::Ready(scene)) => Model::Ready(scene),
        Some(Load::Failed) => Model::Missing,
        Some(Load::Loading) => Model::Loading,
        None => {
            state.insert(key, Load::Loading);
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                let load = glb::parse(bytes).map_or(Load::Failed, |mut s| {
                    super::led::prepare(&mut s, key);
                    super::keycaps::prepare(&mut s, key);
                    Load::Ready(Arc::new(s))
                });
                if let Ok(mut loads) = loads().lock() {
                    loads.insert(key, load);
                }
                ctx.request_repaint();
            });
            Model::Loading
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_models_parse() {
        for (id, bytes) in MODELS {
            let scene = glb::parse(bytes).unwrap_or_else(|e| panic!("{id}: {e}"));
            assert!(scene.radius > 0.0);
            assert!(scene.batches.iter().any(|b| !b.indices.is_empty()));
        }
    }
}
