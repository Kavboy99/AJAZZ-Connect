//! An isolated, event-driven backup workflow shared by Settings and the small standalone utility.
use super::{Event, Events, Session, events::Waker};

pub struct BackupSession {
    session: Session,
    events: Events,
    selected: usize,
}

impl BackupSession {
    pub fn new(wake: Waker) -> Self {
        Self {
            session: Session::default(),
            events: Events::new(wake),
            selected: 0,
        }
    }
    pub fn refresh(&mut self) {
        // Driver discovery is read-only. Only backup actions are exposed by this workflow.
        self.session.discover_backups(&self.events);
    }
    pub fn drain(&mut self) {
        while let Some(event) = self.events.try_next() {
            match event {
                Event::Discovered(found) => {
                    let moved = self.session.adopt(found);
                    self.selected = moved.get(self.selected).copied().flatten().unwrap_or(0);
                }
                Event::Unchanged => self.session.unchanged(),
                Event::Lighting {
                    slot,
                    device,
                    result,
                } => self.session.finish(slot, device, result),
                Event::Battery(_) | Event::Probed(_) => {}
            }
        }
    }
    pub fn session(&self) -> &Session {
        &self.session
    }
    pub fn selected(&self) -> Option<usize> {
        (self.selected < self.session.entries().len()).then_some(self.selected)
    }
    pub fn select(&mut self, index: usize) {
        if index < self.session.entries().len() {
            self.selected = index;
        }
    }
    pub fn parts(&mut self) -> (&mut Session, &Events) {
        (&mut self.session, &self.events)
    }
    pub fn adopt_preview(&mut self, found: super::events::Discovered) {
        self.session.adopt(found);
        self.selected = 0;
    }
}
