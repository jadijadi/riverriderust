use std::{cell::RefCell, io::Stdout, thread, time::Duration};

use uuid::Uuid;

use crate::{
    entities::PlayerStatus,
    events::handle_pressed_keys,
    world::{World, WorldEvent, WorldEventTrigger, WorldStatus, WorldTimer},
};

pub struct Game<'g> {
    pub world: RefCell<World<'g>>,
    events: Vec<WorldEvent<'g>>,
}

impl<'g> Game<'g> {
    pub fn new(max_c: u16, max_l: u16) -> Self {
        Self {
            world: RefCell::new(World::new(max_c, max_l)),
            events: Vec::new(),
        }
    }

    pub fn add_event_handler(&mut self, event: WorldEvent<'g>) {
        self.events.push(event);
    }

    pub fn add_timer(&mut self, timer: WorldTimer, on_elapsed: impl Fn(String, &mut World) + 'g) {
        let is_repeat = timer.repeat;
        let key: String = Uuid::new_v4().to_string();
        self.world
            .borrow_mut()
            .timers
            .get_mut()
            .insert(key.clone(), timer);
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::TimerElapsed(key.clone()),
            is_repeat,
            move |world| on_elapsed(key.clone(), world),
        ));
    }

    fn run_events(&mut self) {
        self.events.retain(|event| {
            if event.trigger.is_triggered(&self.world.borrow()) {
                (event.handler)(&mut self.world.borrow_mut());
                event.is_continues
            } else {
                true
            }
        });
    }

    pub fn game_loop(&mut self, stdout: &mut Stdout, slowness: u64) -> Result<(), std::io::Error> {
        while self.world.borrow().player.status == PlayerStatus::Alive {
            handle_pressed_keys(&mut self.world.borrow_mut())?;
            let world_status = self.world.borrow().status;
            match world_status {
                WorldStatus::Fluent => {
                    self.run_events();

                    let new_events: Vec<WorldEvent<'g>> =
                        self.world.borrow_mut().new_events.drain(0..).collect();
                    for event in new_events {
                        self.add_event_handler(event)
                    }
                    // Draw drawings on canvas first
                    self.world.borrow_mut().draw_on_canvas();
                    self.draw_status();
                }
                WorldStatus::Solid => self.world.borrow_mut().pause_screen(),
            }

            // Draw canvas map into stdout.
            let world = &mut self.world.borrow_mut();
            world.canvas.draw_map(stdout)?;

            thread::sleep(Duration::from_millis(slowness));
            world.elapsed_loops += 1;
        }

        Ok(())
    }

    pub fn events_len(&self) -> usize {
        self.events.len()
    }
}
