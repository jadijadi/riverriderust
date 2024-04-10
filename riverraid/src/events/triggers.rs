use std::rc::Rc;

use crate::{timer::TimerKey, world::World};

use super::LeaveAlone;

#[derive(Clone)]
pub struct EventTrigger {
    trigger: Rc<dyn Fn(&World) -> bool>,
}

impl Default for EventTrigger {
    fn default() -> Self {
        LeaveAlone.into_event_trigger()
    }
}

impl EventTrigger {
    pub fn new(trigger: impl Fn(&World) -> bool + 'static) -> Self {
        Self {
            trigger: Rc::new(trigger),
        }
    }

    pub fn is_triggered(&self, world: &World) -> bool {
        (self.trigger)(world)
    }
}

pub trait IntoEventTrigger {
    fn into_event_trigger(self) -> EventTrigger;
}

impl IntoEventTrigger for EventTrigger {
    fn into_event_trigger(self) -> EventTrigger {
        self
    }
}

impl IntoEventTrigger for LeaveAlone {
    fn into_event_trigger(self) -> EventTrigger {
        EventTrigger::new(|_| true)
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum WorldEventTrigger {
    Always,
    GameStarted,
    Traveled(u16),
    TimerElapsed(TimerKey),
    DrawingExists(String),
    Signal(String),
}

impl WorldEventTrigger {
    #[allow(dead_code)]
    pub fn timer_elapsed(timer_key: impl Into<TimerKey>) -> Self {
        Self::TimerElapsed(timer_key.into())
    }

    pub fn is_triggered(&self, world: &World) -> bool {
        match self {
            WorldEventTrigger::Always => true,
            WorldEventTrigger::Traveled(distance) => &world.player.traveled >= distance,
            WorldEventTrigger::TimerElapsed(key) => world.timer_elapsed(key).unwrap_or(false),
            WorldEventTrigger::GameStarted => world.elapsed_loops <= 0,
            WorldEventTrigger::DrawingExists(key) => world.custom_drawings.contains_key(key),
            WorldEventTrigger::Signal(signal_key) => world.signaled(signal_key),
        }
    }
}

impl IntoEventTrigger for WorldEventTrigger {
    fn into_event_trigger(self) -> EventTrigger {
        EventTrigger::new(move |world| self.is_triggered(world))
    }
}
