//! System fonts only: Segoe UI for text and Cascadia Mono for identifiers. Nothing is bundled; a
//! missing file falls back to the built-in egui font.

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use std::path::PathBuf;
use std::sync::Arc;

const SEMIBOLD: &str = "semibold";
const LIGHT: &str = "light";

pub fn semibold() -> FontFamily {
    FontFamily::Name(SEMIBOLD.into())
}

pub fn light() -> FontFamily {
    FontFamily::Name(LIGHT.into())
}

fn system_font(file: &str) -> Option<Arc<FontData>> {
    let dir = PathBuf::from(std::env::var_os("WINDIR")?).join("Fonts");
    std::fs::read(dir.join(file))
        .ok()
        .map(|data| Arc::new(FontData::from_owned(data)))
}

pub fn install(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    let mut semibold_fonts = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let mut light_fonts = semibold_fonts.clone();
    for (name, file, family) in [
        ("segoe-ui", "segoeui.ttf", FontFamily::Proportional),
        ("segoe-ui-semibold", "seguisb.ttf", semibold()),
        ("segoe-ui-light", "segoeuil.ttf", light()),
        ("cascadia-mono", "CascadiaMono.ttf", FontFamily::Monospace),
    ] {
        let Some(data) = system_font(file) else {
            continue;
        };
        fonts.font_data.insert(name.into(), data);
        if family == semibold() {
            semibold_fonts.insert(0, name.into());
        } else if family == light() {
            light_fonts.insert(0, name.into());
        } else {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, name.into());
        }
    }
    fonts.families.insert(semibold(), semibold_fonts);
    fonts.families.insert(light(), light_fonts);
    ctx.set_fonts(fonts);
}
