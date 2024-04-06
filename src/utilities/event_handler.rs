use crate::world::World;

pub struct EventHandler<'g> {
    handler: Box<dyn Fn(&mut World) + 'g>,
}

impl<'g> EventHandler<'g> {
    pub fn new(handler: impl Fn(&mut World) + 'g) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }

    pub fn handle(&self, world: &mut World) {
        (self.handler)(world)
    }
}

pub trait IntoEventHandler<'g> {
    fn into_event_handler(self) -> EventHandler<'g>;

    fn into_timer_event_handler(self) -> TimerEventHandler<'g>
    where
        Self: Sized,
    {
        let handler = self.into_event_handler();
        TimerEventHandler {
            handler: Box::new(move |_, world: &mut World| handler.handle(world)),
        }
    }
}

impl<'g> IntoEventHandler<'g> for EventHandler<'g> {
    fn into_event_handler(self) -> EventHandler<'g> {
        self
    }
}

impl<'g, T: Fn(&mut World) + 'g> IntoEventHandler<'g> for T {
    fn into_event_handler(self) -> EventHandler<'g> {
        EventHandler {
            handler: Box::new(self),
        }
    }
}

pub struct LeaveAlone;

impl<'g> IntoEventHandler<'g> for LeaveAlone {
    fn into_event_handler(self) -> EventHandler<'g> {
        EventHandler {
            handler: Box::new(|_| {}),
        }
    }
}

#[derive(Clone)]
pub struct TimerKey(String);

impl From<String> for TimerKey {
    fn from(value: String) -> Self {
        TimerKey(value)
    }
}

impl TimerKey {
    pub fn new(key: String) -> Self {
        TimerKey(key)
    }
}

impl std::ops::Deref for TimerKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct TimerEventHandler<'g> {
    handler: Box<dyn Fn(TimerKey, &mut World) + 'g>,
}

impl<'g> TimerEventHandler<'g> {
    pub fn into_event_handler(self, timer_key: TimerKey) -> EventHandler<'g> {
        EventHandler {
            handler: Box::new(move |world: &mut World| self.handle(timer_key.clone(), world)),
        }
    }
}

impl<'g> TimerEventHandler<'g> {
    pub fn new(handler: impl Fn(TimerKey, &mut World) + 'g) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }

    pub fn handle(&self, timer_key: TimerKey, world: &mut World) {
        (self.handler)(timer_key, world)
    }
}

pub trait IntoTimerEventHandler<'g, Params> {
    fn into_timer_event_handler(self) -> TimerEventHandler<'g>;

    fn into_event_handler(self, timer_key: TimerKey) -> EventHandler<'g>
    where
        Self: Sized,
    {
        self.into_timer_event_handler()
            .into_event_handler(timer_key)
    }
}

impl<'g> IntoTimerEventHandler<'g, ()> for EventHandler<'g> {
    fn into_timer_event_handler(self) -> TimerEventHandler<'g> {
        TimerEventHandler {
            handler: Box::new(move |_, world: &mut World| self.handle(world)),
        }
    }
}

impl<'g> IntoTimerEventHandler<'g, ()> for TimerEventHandler<'g> {
    fn into_timer_event_handler(self) -> TimerEventHandler<'g> {
        self
    }
}

impl<'g, T: Fn(TimerKey, &mut World) + 'g> IntoTimerEventHandler<'g, (TimerKey, &mut World<'g>)>
    for T
{
    fn into_timer_event_handler(self) -> TimerEventHandler<'g> {
        TimerEventHandler {
            handler: Box::new(self),
        }
    }
}

impl<'g, T: Fn(&mut World) + 'g> IntoTimerEventHandler<'g, (&mut World<'g>,)> for T {
    fn into_timer_event_handler(self) -> TimerEventHandler<'g> {
        TimerEventHandler {
            handler: Box::new(move |_: TimerKey, world: &mut World| self(world)),
        }
    }
}

impl<'g> IntoTimerEventHandler<'g, ()> for LeaveAlone {
    fn into_timer_event_handler(self) -> TimerEventHandler<'g> {
        TimerEventHandler {
            handler: Box::new(|_, _| {}),
        }
    }
}
