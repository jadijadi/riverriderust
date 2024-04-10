use crate::timer::{Timer, TimerKey};

use super::{handlers::IntoEventHandler, Event};

pub trait EventContainer<'g> {
    fn add_event(&mut self, event: impl Event<'g> + 'g);
}

pub trait TimerContainer<'g> {
    fn add_raw_timer(&mut self, timer: Timer) -> TimerKey;
}

pub trait EventSetup<'g, C: EventContainer<'g>> {
    fn setup_event(self, container: &mut C);
}

impl<'g, T: Event<'g> + 'g, C: EventContainer<'g>> EventSetup<'g, C> for T {
    fn setup_event(self, container: &mut C) {
        container.add_event(self)
    }
}

pub trait IntoEventSetup<'g, C: EventContainer<'g>> {
    fn into_event_setup(self, handler: impl IntoEventHandler<'g>) -> impl EventSetup<'g, C>;
}
