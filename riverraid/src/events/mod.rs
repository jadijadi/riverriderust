use typed_builder::TypedBuilder;

use self::{
    handlers::{EventHandler, IntoEventHandler},
    triggers::{EventTrigger, IntoEventTrigger},
};

pub mod handlers;
pub mod setup;
pub mod triggers;

#[derive(TypedBuilder)]
pub struct WorldBuilder<'g> {
    pub trigger: EventTrigger,
    #[builder(default)]
    pub is_continues: bool,
    pub handler: EventHandler<'g>,
}

impl<'g> WorldBuilder<'g> {
    pub fn new(
        trigger: impl IntoEventTrigger,
        is_continues: bool,
        handler: impl IntoEventHandler<'g>,
    ) -> Self {
        Self {
            trigger: trigger.into_event_trigger(),
            handler: handler.into_event_handler(),
            is_continues,
        }
    }
}

pub trait Event<'g> {
    fn is_continues(&self) -> bool;

    fn trigger(&self) -> impl IntoEventTrigger;

    fn handler(self) -> impl IntoEventHandler<'g>;

    fn into_world_event(self) -> WorldBuilder<'g>
    where
        Self: Sized + 'g,
    {
        let trigger = self.trigger().into_event_trigger();
        let is_continues = self.is_continues();
        let handler = self.handler().into_event_handler();
        WorldBuilder::new(trigger, is_continues, handler)
    }
}

impl<'g> Event<'g> for WorldBuilder<'g> {
    fn trigger(&self) -> impl IntoEventTrigger {
        self.trigger.to_owned()
    }

    fn handler(self) -> impl IntoEventHandler<'g> {
        self.handler
    }

    fn is_continues(&self) -> bool {
        self.is_continues
    }
}
