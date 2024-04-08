use std::rc::Rc;

use crate::world::{TimerKey, World};

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
        TimerEventHandler::new(move |_, world| handler.handle(world))
    }
}

impl<'g> IntoEventHandler<'g> for EventHandler<'g> {
    fn into_event_handler(self) -> EventHandler<'g> {
        self
    }
}

impl<'g, T: Fn(&mut World) + 'g> IntoEventHandler<'g> for T {
    fn into_event_handler(self) -> EventHandler<'g> {
        EventHandler::new(self)
    }
}

pub struct LeaveAlone;

impl<'g> IntoEventHandler<'g> for LeaveAlone {
    fn into_event_handler(self) -> EventHandler<'g> {
        EventHandler::new(|_| {})
    }
}

#[derive(Clone)]
pub struct TimerEventHandler<'g> {
    handler: Rc<dyn Fn(TimerKey, &mut World) + 'g>,
}

impl<'g> TimerEventHandler<'g> {
    pub fn into_event_handler(self, timer_key: TimerKey) -> EventHandler<'g> {
        EventHandler::new(move |world| self.handle(timer_key.clone(), world))
    }
}

impl<'g> TimerEventHandler<'g> {
    pub fn new(handler: impl Fn(TimerKey, &mut World) + 'g) -> Self {
        Self {
            handler: Rc::new(handler),
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
        TimerEventHandler::new(move |_, world| self.handle(world))
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
        TimerEventHandler::new(self)
    }
}

impl<'g, T: Fn(&mut World) + 'g> IntoTimerEventHandler<'g, (&mut World<'g>,)> for T {
    fn into_timer_event_handler(self) -> TimerEventHandler<'g> {
        TimerEventHandler::new(move |_, world| self(world))
    }
}

impl<'g> IntoTimerEventHandler<'g, ()> for LeaveAlone {
    fn into_timer_event_handler(self) -> TimerEventHandler<'g> {
        TimerEventHandler::new(|_, _| {})
    }
}
