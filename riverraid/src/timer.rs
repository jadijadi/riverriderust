use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::events::{
    handlers::{IntoEventHandler, IntoTimerEventHandler, TimerEventHandler},
    setup::{EventContainer, EventSetup, IntoEventSetup, TimerContainer},
    triggers::{IntoEventTrigger, WorldEventTrigger},
    WorldBuilder,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimerKey(String);

impl From<String> for TimerKey {
    fn from(value: String) -> Self {
        TimerKey(value)
    }
}

impl std::ops::Deref for TimerKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct TimerData {
    pub(crate) key: TimerKey,
    pub(crate) duration: Duration,
    pub(crate) repeat: bool,
}

impl TimerData {
    pub fn new(duration: Duration, repeat: bool) -> Self {
        Self {
            key: TimerKey(Uuid::new_v4().to_string()),
            duration,
            repeat,
        }
    }
}

impl TimerData {
    pub fn key(&self) -> &TimerKey {
        &self.key
    }

    #[allow(dead_code)]
    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    pub fn is_repeat(&self) -> bool {
        self.repeat
    }
}

#[derive(Debug)]
pub struct Timer {
    pub data: TimerData,
    instant: Instant,
}

impl Timer {
    pub fn new(duration: Duration, repeat: bool) -> Self {
        Self {
            data: TimerData::new(duration, repeat),
            instant: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> bool {
        self.instant.elapsed() > self.data.duration
    }

    pub fn reset(&mut self) {
        log::info!("Timer {:?} reset.", self.data.key);
        self.instant = Instant::now();
    }
}

pub struct TimerEventSetup<'g> {
    timer: Timer,
    handler: TimerEventHandler<'g>,
}

impl<'g> TimerEventSetup<'g> {
    pub fn new<Params>(timer: Timer, handler: impl IntoTimerEventHandler<'g, Params>) -> Self {
        Self {
            timer,
            handler: handler.into_timer_event_handler(),
        }
    }

    fn setup<C: EventContainer<'g> + TimerContainer<'g>>(self, container: &mut C) {
        let TimerEventSetup { timer, handler } = self;

        container.add_event(WorldBuilder {
            trigger: WorldEventTrigger::TimerElapsed(timer.data.key.clone()).into_event_trigger(),
            continues: timer.data.repeat,
            handler: handler.into_event_handler(timer.data.key.clone()),
        });
        container.add_raw_timer(timer);
    }
}

impl<'g, C: EventContainer<'g> + TimerContainer<'g>> EventSetup<'g, C> for TimerEventSetup<'g> {
    fn setup_event(self, container: &mut C) {
        self.setup(container)
    }
}

impl<'g, C: EventContainer<'g> + TimerContainer<'g>> IntoEventSetup<'g, C> for Timer {
    fn into_event_setup(self, handler: impl IntoEventHandler<'g>) -> impl EventSetup<'g, C> {
        TimerEventSetup::new(self, handler.into_event_handler())
    }
}
