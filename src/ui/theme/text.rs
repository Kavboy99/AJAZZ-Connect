//! Type ramp (`docs/DESIGN.md`, section 4). Body text is never below 14 px.

use super::fonts;
use eframe::egui::{Color32, FontFamily, FontId, RichText, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Caption,
    Body,
    BodyStrong,
    Subtitle,
    Title,
    /// Hashes and identifiers only.
    Mono,
}

impl Role {
    pub fn size(self) -> f32 {
        match self {
            Role::Caption => 12.0,
            Role::Body | Role::BodyStrong | Role::Mono => 14.0,
            Role::Subtitle => 20.0,
            Role::Title => 28.0,
        }
    }

    pub fn line_height(self) -> f32 {
        match self {
            Role::Caption => 16.0,
            Role::Body | Role::BodyStrong | Role::Mono => 20.0,
            Role::Subtitle => 28.0,
            Role::Title => 36.0,
        }
    }

    pub fn font(self) -> FontId {
        let family = match self {
            Role::Caption | Role::Body => FontFamily::Proportional,
            Role::BodyStrong | Role::Subtitle | Role::Title => fonts::semibold(),
            Role::Mono => FontFamily::Monospace,
        };
        FontId::new(self.size(), family)
    }
}

pub fn rich(text: impl Into<String>, role: Role, color: Color32) -> RichText {
    RichText::new(text)
        .font(role.font())
        .color(color)
        .line_height(Some(role.line_height()))
}

/// A wrapping label in the given role and color.
pub fn label(ui: &mut Ui, text: impl Into<String>, role: Role, color: Color32) {
    ui.add(eframe::egui::Label::new(rich(text, role, color)).wrap());
}
