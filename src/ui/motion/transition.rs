//! Directional page transitions (`docs/DESIGN.md`, section 9.4). The new page arrives on
//! `spring.page`, the old one leaves faster on `spring.fast`. A new navigation in the middle of a
//! transition starts from what is on screen, so nothing jumps.

use super::{Channel, Spring, step, tokens};
use crate::ui::theme::motion_tokens;
use eframe::egui::{Context, Vec2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// Peer pages: the new page fades in and rises.
    Entrance,
    /// Going deeper: the new page grows from slightly smaller.
    DrillIn,
    /// Going back: mirrors `DrillIn`.
    DrillOut,
}

/// How to draw one page this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub opacity: f32,
    pub offset: Vec2,
    pub scale: f32,
}

pub struct Transition<P> {
    current: P,
    previous: Option<P>,
    style: Style,
    incoming: Spring,
    outgoing: Spring,
}

impl<P: Copy + PartialEq> Transition<P> {
    pub fn new(page: P) -> Self {
        Self {
            current: page,
            previous: None,
            style: Style::Entrance,
            incoming: Spring::at(1.0),
            outgoing: Spring::at(0.0),
        }
    }

    pub fn current(&self) -> P {
        self.current
    }

    pub fn navigate(&mut self, to: P, style: Style) {
        if to == self.current {
            return;
        }
        // The leaving page starts from its visible opacity.
        self.outgoing = Spring::at(self.incoming.value().clamp(0.0, 1.0));
        self.outgoing.set_target(0.0);
        self.previous = Some(self.current);
        self.current = to;
        self.style = style;
        self.incoming = Spring::at(0.0);
        self.incoming.set_target(1.0);
    }

    /// The same page under a new name (a device that moved to another slot): no motion.
    pub fn rename(&mut self, to: P) {
        self.current = to;
    }

    pub fn step(&mut self, ctx: &Context) {
        step(ctx, &mut self.incoming, tokens::PAGE, Channel::Fade);
        step(ctx, &mut self.outgoing, tokens::FAST, Channel::Fade);
        if self.outgoing.value() <= 0.0 {
            self.previous = None;
        }
    }

    pub fn incoming(&self, reduced: bool) -> Frame {
        let p = self.incoming.value();
        let mut frame = Frame {
            opacity: p.clamp(0.0, 1.0),
            offset: Vec2::ZERO,
            scale: 1.0,
        };
        if !reduced {
            let d = motion_tokens::DRILL_SCALE;
            match self.style {
                Style::Entrance => frame.offset = vec2(0.0, (1.0 - p) * motion_tokens::PAGE_RISE),
                Style::DrillIn => frame.scale = d + (1.0 - d) * p,
                Style::DrillOut => frame.scale = 1.0 + (1.0 - d) * (1.0 - p),
            }
        }
        frame
    }

    /// The leaving page fades in place; it does not move.
    pub fn outgoing(&self) -> Option<(P, Frame)> {
        self.previous.map(|page| {
            (
                page,
                Frame {
                    opacity: self.outgoing.value().clamp(0.0, 1.0),
                    offset: Vec2::ZERO,
                    scale: 1.0,
                },
            )
        })
    }
}
