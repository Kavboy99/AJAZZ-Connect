//! Cosmetic preferences never participate in device identification or writes.
use crate::app::settings::Settings;
use crate::ui::{
    motion::{self, Channel, tokens},
    strings as s,
    theme::color,
};
use eframe::egui::{self, Context, Id, Rect, Sense, Stroke, Ui, vec2};

pub fn initialize(ctx: &Context, settings: &Settings) {
    ctx.data_mut(|d| d.insert_temp(Id::new("model-finish"), settings.orange_keycaps));
}
fn orange(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp(Id::new("model-finish")).unwrap_or(false))
}
pub fn selected<'a>(ctx: &Context, visual: &'a str) -> &'a str {
    if visual == "ak820-purple" && orange(ctx) {
        "ak820-max"
    } else {
        visual
    }
}

pub fn fallback(visual: Option<&str>, keyboard: bool) -> Option<&str> {
    visual.or(if keyboard { Some("ak820-max") } else { None })
}

pub fn swatches(ui: &mut Ui, stage: Rect) {
    let current = orange(ui.ctx());
    for (i, (value, rgb, label)) in [
        (true, [231, 146, 66], s::FINISH_ORANGE),
        (false, [174, 149, 202], s::FINISH_PURPLE),
    ]
    .into_iter()
    .enumerate()
    {
        let center = egui::pos2(
            stage.center().x + (i as f32 - 0.5) * 28.0,
            stage.bottom() - 22.0,
        );
        let id = Id::new(("finish-swatch", i));
        let response = ui.interact(
            Rect::from_center_size(center, vec2(26.0, 30.0)),
            id,
            Sense::click(),
        );
        let selected = motion::animate(
            ui.ctx(),
            id,
            f32::from(current == value),
            tokens::STANDARD,
            Channel::Scale,
        );
        ui.painter()
            .circle_filled(center, 7.0, egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        ui.painter().circle_stroke(
            center,
            10.0,
            Stroke::new(1.5_f32, color::TEXT_PRIMARY.get().gamma_multiply(selected)),
        );
        if response.clicked() {
            ui.ctx()
                .data_mut(|d| d.insert_temp(Id::new("model-finish"), value));
            ui.ctx().request_repaint();
        }
        response.on_hover_text(label);
    }
}

pub fn persist(ctx: &Context, settings: &mut Settings) {
    let value = orange(ctx);
    if settings.orange_keycaps != value {
        settings.orange_keycaps = value;
        let _ = settings.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_finish_only_changes_the_selectable_model() -> Result<(), serde_json::Error> {
        let ctx = Context::default();
        let legacy: Settings = serde_json::from_str("{}")?;
        initialize(&ctx, &legacy);
        assert_eq!(selected(&ctx, "ak820-purple"), "ak820-purple");
        let settings = Settings {
            orange_keycaps: true,
            ..legacy
        };
        let restored = serde_json::from_slice(&serde_json::to_vec(&settings)?)?;
        initialize(&ctx, &restored);
        assert_eq!(selected(&ctx, "ak820-purple"), "ak820-max");
        assert_eq!(selected(&ctx, "ak820-white"), "ak820-white");
        assert_eq!(fallback(None, true), Some("ak820-max"));
        assert_eq!(fallback(None, false), None);
        assert_eq!(fallback(Some("custom-model"), true), Some("custom-model"));
        Ok(())
    }
}
