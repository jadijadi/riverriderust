use self::{
    handlers::{EventHandler, IntoEventHandler},
    triggers::{EventTrigger, IntoEventTrigger},
};

pub mod handlers;
pub mod setup;
pub mod triggers;

#[derive(Clone, Default)]
pub struct WorldBuilder<'g> {
    pub trigger: EventTrigger,
    pub continues: bool,
    pub handler: EventHandler<'g>,
}

impl<'g> WorldBuilder<'g> {
    pub fn new(trigger: impl IntoEventTrigger) -> Self {
        Self {
            trigger: trigger.into_event_trigger(),
            handler: Default::default(),
            continues: Default::default(),
        }
    }

    pub fn is_continues(mut self) -> Self {
        self.continues = true;
        self
    }

    pub fn with_handler(mut self, handler: impl IntoEventHandler<'g>) -> Self {
        self.handler = handler.into_event_handler();
        self
    }
}

pub trait Event<'g> {
    fn continues(&self) -> bool;

    fn trigger(&self) -> impl IntoEventTrigger;

    fn handler(self) -> impl IntoEventHandler<'g>;

    fn into_world_event(self) -> WorldBuilder<'g>
    where
        Self: Sized + 'g,
    {
        let trigger = self.trigger().into_event_trigger();
        let continues = self.continues();
        let handler = self.handler().into_event_handler();
        WorldBuilder {
            trigger,
            continues,
            handler,
        }
    }
}

impl<'g> Event<'g> for WorldBuilder<'g> {
    fn trigger(&self) -> impl IntoEventTrigger {
        self.trigger.to_owned()
    }

    fn handler(self) -> impl IntoEventHandler<'g> {
        self.handler
    }

    fn continues(&self) -> bool {
        self.continues
    }
}

/// Default value for callback options.
///
/// - Dose nothing if it's an event handler
/// - Always true if it's a filter callback
pub struct LeaveAlone;
