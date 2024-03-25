use std::{cell::RefCell, io::Stdout, thread, time::Duration};

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
    pub fn new(world: World<'g>) -> Self {
        Self {
            world: RefCell::new(world),
            events: Vec::new(),
        }
    }

    pub fn add_event_handler(&mut self, event: WorldEvent<'g>) {
        self.events.push(event);
    }

    pub fn add_timer(
        &mut self,
        key: impl Into<String>,
        timer: WorldTimer,
        on_elapsed: impl Fn(&mut World) + 'g,
    ) {
        let is_repeat = timer.repeat;
        let key: String = key.into();
        self.world
            .borrow_mut()
            .timers
            .get_mut()
            .insert(key.clone(), timer);
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::TimerElapsed(key),
            is_repeat,
            on_elapsed,
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
                    // Draw drawings on canvas first
                    let world = &mut self.world.borrow_mut();

                    self.world.borrow_mut().draw_on_canvas();
                }
                WorldStatus::Paused => self.world.borrow_mut().pause_screen(),
            }

            // Draw canvas map into stdout.
            let world = &mut self.world.borrow_mut();
            world.canvas.draw_map(stdout)?;

            thread::sleep(Duration::from_millis(slowness));
            world.elapsed_loops += 1;
        }

        Ok(())
    }
}
