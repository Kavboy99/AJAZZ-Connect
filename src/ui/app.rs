use super::motion::{self, Channel, Style, tokens};
use super::pages::{Model, Pages, Route};
use super::shell::Shell;
use super::theme::{self, color, mode};
use crate::app::{Diagnostics, Due, Event, Events, Session, Settings, Theme, Watch};
use crate::core::DriverInfo;
use crate::drivers;
use crate::transport;
use eframe::egui::{Context, Id};
use std::sync::Arc;

pub struct App {
    events: Events,
    session: Session,
    diagnostics: Diagnostics,
    settings: Settings,
    drivers: Vec<DriverInfo>,
    shell: Shell,
    pages: Pages,
    /// Device arrival and removal; only in the real window, never in `--review`.
    watch: Option<Watch>,
    /// The DWM backdrop is active: the navigation pane is left transparent.
    backdrop: bool,
    /// The theme the backdrop was last tinted for.
    backdrop_theme: Theme,
}

fn blend_target(theme: Theme) -> f32 {
    match theme {
        Theme::Dark => 0.0,
        Theme::Light => 1.0,
    }
}

impl App {
    pub fn new(ctx: &Context, backdrop: bool, settings: Settings) -> Self {
        mode::set_blend(blend_target(settings.theme));
        theme::fonts::install(ctx);
        theme::style::apply(ctx);
        let repaint = ctx.clone();
        Self {
            events: Events::new(Arc::new(move || repaint.request_repaint())),
            session: Session::default(),
            diagnostics: Diagnostics::default(),
            backdrop_theme: settings.theme,
            settings,
            drivers: drivers::registry().iter().map(|d| d.info()).collect(),
            shell: Shell::default(),
            pages: Pages::default(),
            watch: None,
            backdrop,
        }
    }

    /// Finds devices now and again whenever one comes or goes. The operating system calls back;
    /// nothing polls.
    pub fn watch_devices(&mut self, ctx: &Context) {
        let wake = ctx.clone();
        self.watch = Some(Watch::start(Arc::new(move |after| {
            wake.request_repaint_after(after)
        })));
    }

    /// Starts the automatic search once the device list has settled and nothing else runs.
    fn search_when_due(&mut self, ctx: &Context) {
        let Some(watch) = &self.watch else {
            return;
        };
        match watch.due() {
            Due::Now if !self.session.busy() => {
                watch.clear();
                self.session
                    .refresh(&self.events, self.settings.experimental_drivers);
            }
            Due::After(left) => ctx.request_repaint_after(left),
            // A running operation finishes with an event, which brings the next frame.
            Due::Now | Due::Nothing => {}
        }
    }

    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    pub fn diagnostics_mut(&mut self) -> &mut Diagnostics {
        &mut self.diagnostics
    }

    pub fn open(&mut self, route: Route) {
        self.shell.navigate(route, Style::Entrance);
    }

    fn drain(&mut self) {
        while let Some(event) = self.events.try_next() {
            match event {
                Event::Discovered(found) => {
                    self.pages.forget_devices();
                    let moved = self.session.adopt(found);
                    self.shell.follow(&moved);
                }
                Event::Unchanged => self.session.unchanged(),
                Event::Lighting {
                    slot,
                    device,
                    result,
                } => self.session.finish(slot, device, result),
                Event::Probed(result) => self.diagnostics.finish_probe(*result),
            }
        }
    }

    /// One frame. Runs only on input, a finished task or a moving spring.
    pub fn frame(&mut self, ctx: &Context) {
        let animations = transport::animations_enabled();
        motion::begin_frame(ctx, animations);
        // The whole palette flows to the new theme on one spring.
        let blend = motion::animate(
            ctx,
            Id::new("theme"),
            blend_target(self.settings.theme),
            tokens::STANDARD,
            Channel::Fade,
        );
        if blend != mode::blend() {
            mode::set_blend(blend);
            theme::style::apply(ctx);
        }
        self.drain();
        self.search_when_due(ctx);
        let mut model = Model {
            session: &mut self.session,
            diagnostics: &mut self.diagnostics,
            settings: &mut self.settings,
            events: &self.events,
            drivers: &self.drivers,
            animations,
        };
        self.shell
            .show(ctx, self.backdrop, &mut model, &mut self.pages);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.frame(ctx);
        if self.backdrop && self.backdrop_theme != self.settings.theme {
            self.backdrop_theme = self.settings.theme;
            super::tint_backdrop(frame, self.settings.theme == Theme::Dark);
        }
    }

    fn clear_color(&self, _: &eframe::egui::Visuals) -> [f32; 4] {
        if self.backdrop {
            [0.0; 4]
        } else {
            color::SURFACE_BASE.get().to_normalized_gamma_f32()
        }
    }
}
