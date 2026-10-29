//! Exercise the actual inspector and cards while telemetry owns an in-memory device.
use super::*;
use crate::app::battery::{BatteryMode, BatteryView};
use crate::app::{Diagnostics, Event, Events, Session, Settings};
use crate::core::{
    Battery, Device, DeviceDescriptor, DeviceKind, DriverError, DriverInfo, Lighting,
    LightingEditor, LightingEffect, LightingPreset, LightingSettings, LightingTraits, Maturity,
    Outcome, Rgb, Rollback, Snapshot,
};
use crate::ui::strings as s;
use eframe::egui::{
    CentralPanel, Context, Event as Input, FullOutput, Modifiers, PointerButton, RawInput, Shape,
};
use std::sync::{Arc, mpsc};
use std::time::Duration;

struct Keyboard {
    descriptor: DeviceDescriptor,
    editor: LightingEditor,
}

impl Device for Keyboard {
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }
    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        Some(self)
    }
    fn battery(&mut self) -> Option<&mut dyn Battery> {
        Some(self)
    }
}
impl Battery for Keyboard {
    fn refresh_battery(&mut self) -> Result<(), DriverError> {
        Ok(())
    }
}
impl Lighting for Keyboard {
    fn traits(&self) -> LightingTraits {
        LightingTraits {
            writable: true,
            persistent: true,
            rollback: Rollback::Unavailable,
        }
    }
    fn editor(&self) -> Option<LightingEditor> {
        Some(self.editor.clone())
    }
    fn apply_settings(&mut self, settings: LightingSettings) -> Result<Outcome, DriverError> {
        self.editor.current = settings.preset;
        self.editor.presets[usize::from(settings.preset)].saved = settings;
        Ok(Outcome::Confirmed)
    }
    fn apply_solid(&mut self, _: Rgb) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
    fn restore(&mut self) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
    fn restore_snapshot(&mut self, _: &Snapshot) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
}

fn session() -> Session {
    let mut session = Session::default();
    session.adopt(crate::app::events::Discovered {
        devices: vec![Box::new(Keyboard {
            descriptor: DeviceDescriptor {
                name: "Test".into(),
                vendor: None,
                visual: None,
                details: vec![],
                kind: DeviceKind::Keyboard,
                driver: DriverInfo {
                    id: "test",
                    name: "Test",
                    maturity: Maturity::Stable,
                    requirement: None,
                },
            },
            editor: LightingEditor {
                current: 0,
                presets: [
                    LightingEffect::Solid,
                    LightingEffect::Breathing,
                    LightingEffect::Rainbow,
                ]
                .into_iter()
                .enumerate()
                .map(|(i, effect)| LightingPreset {
                    effect,
                    saved: LightingSettings {
                        preset: i as u8,
                        brightness: 3,
                        speed: 2,
                        color: None,
                    },
                    device_color: None,
                    custom_color: false,
                    brightness_max: Some(4),
                    speed_max: Some(4),
                })
                .collect(),
            },
        })],
        dormant: vec![],
        notes: vec![],
        admitted: vec![],
    });
    session
}

fn frame(
    ctx: &Context,
    session: &mut Session,
    events: &Events,
    state: &mut State,
    input: Vec<Input>,
) -> FullOutput {
    ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Default::default(), vec2(1040.0, 840.0))),
            events: input,
            focused: false,
            ..Default::default()
        },
        |ctx| {
            motion::begin_frame(ctx, true);
            CentralPanel::default().show(ctx, |ui| {
                let mut settings = Settings {
                    ephemeral: true,
                    ..Default::default()
                };
                let mut diagnostics = Diagnostics::default();
                let mut model = Model {
                    session,
                    events,
                    settings: &mut settings,
                    diagnostics: &mut diagnostics,
                    drivers: &[],
                    animations: true,
                };
                crate::ui::pages::device::show(ui, &mut model, 0, state);
            });
        },
    )
}

fn labels(output: &FullOutput, text: &str) -> Vec<Rect> {
    fn collect(shape: &Shape, text: &str, rects: &mut Vec<Rect>) {
        match shape {
            Shape::Text(s) if s.galley.job.text == text => {
                rects.push(Rect::from_min_size(s.pos, s.galley.size()))
            }
            Shape::Vec(shapes) => shapes.iter().for_each(|s| collect(s, text, rects)),
            _ => {}
        }
    }
    let mut rects = vec![];
    for s in &output.shapes {
        collect(&s.shape, text, &mut rects);
    }
    rects
}

#[test]
fn battery_refreshes_preserve_card_actions_and_apply_accepts_one_click_during_telemetry() {
    let ctx = Context::default();
    crate::ui::theme::fonts::install(&ctx);
    crate::ui::theme::style::apply(&ctx);
    let mut session = session();
    let (tx, rx) = mpsc::channel();
    let events = Events::new(Arc::new(move || {
        let _ = tx.send(());
    }));
    let mut state = State::default();
    state.enter(Mode::Lighting);
    let initial = frame(&ctx, &mut session, &events, &mut state, vec![]);
    let preview = labels(&initial, s::lighting_effect(LightingEffect::Breathing))[0].center();
    for pressed in [true, false] {
        frame(
            &ctx,
            &mut session,
            &events,
            &mut state,
            vec![
                Input::PointerMoved(preview),
                Input::PointerButton {
                    pos: preview,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    assert_eq!(state.programs.as_ref().unwrap().selected, 1);
    assert_eq!(session.entries()[0].lighting_editor().unwrap().current, 0);
    assert!(session.entries()[0].status().is_none());
    state.open_inspector();
    for _ in 0..160 {
        frame(&ctx, &mut session, &events, &mut state, vec![]);
    }
    let settled = frame(&ctx, &mut session, &events, &mut state, vec![]);
    // Previewing another card must keep Select; only the confirmed device mode hides it.
    assert_eq!(labels(&settled, s::LIGHTING_SELECT).len(), 2);
    for _ in 0..3 {
        session.poll_batteries(
            &events,
            BatteryMode::Automatic,
            BatteryView::Device(0),
            true,
        );
        assert!(session.entries()[0].busy());
        let pending = frame(&ctx, &mut session, &events, &mut state, vec![]);
        assert_eq!(
            pending.shapes, settled.shapes,
            "telemetry changed editor visuals"
        );
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let Event::Battery(results) = events.try_next().unwrap() else {
            panic!("expected battery event")
        };
        for result in results {
            session.finish_battery(result);
        }
        let finished = frame(&ctx, &mut session, &events, &mut state, vec![]);
        assert_eq!(
            finished.shapes, settled.shapes,
            "battery completion changed editor visuals"
        );
    }
    session.poll_batteries(
        &events,
        BatteryMode::Automatic,
        BatteryView::Device(0),
        true,
    );
    let pos = labels(&settled, s::LIGHTING_APPLY_PROGRAM)[0].center();
    for pressed in [true, false] {
        frame(
            &ctx,
            &mut session,
            &events,
            &mut state,
            vec![
                Input::PointerMoved(pos),
                Input::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    assert!(
        session.entries()[0].lighting_pending(),
        "Apply was disabled by telemetry"
    );
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let Event::Battery(results) = events.try_next().unwrap() else {
        panic!("expected battery event")
    };
    for result in results {
        session.finish_battery(result);
    }
    session.flush_lighting(&events);
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let Event::Lighting {
        slot,
        device,
        result,
    } = events.try_next().unwrap()
    else {
        panic!("expected lighting event")
    };
    session.finish(slot, device, result);
    assert_eq!(session.entries()[0].lighting_editor().unwrap().current, 1);
    assert!(events.try_next().is_none());
}
