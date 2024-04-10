use std::{
    cell::RefCell,
    io::{self, Stdout},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, Show},
    terminal::{disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand,
};

use crate::{
    entities::PlayerStatus,
    events::{
        handlers::IntoTimerEventHandler,
        setup::{EventContainer, EventSetup, TimerContainer},
        triggers::WorldEventTrigger,
        Event, WorldBuilder,
    },
    timer::Timer,
    utilities::stout_ext::StdoutExt,
    world::{map::MapUpdater, World, WorldStatus},
};

/// The [`Game`].
///
/// Contains [`World`] and a list of events that act on world.
pub struct Game<'g> {
    pub(crate) world: RefCell<World<'g>>,
    events: Vec<WorldBuilder<'g>>,
}

impl<'g> Game<'g> {
    fn run_events(&mut self) {
        self.events.retain(|event| {
            if event.trigger.is_triggered(&self.world.borrow()) {
                event.handler.handle(&mut self.world.borrow_mut());
                event.is_continues
            } else {
                true
            }
        });
    }

    fn setup_basic_events(&mut self) {
        // move the map Downward
        self.add_event(
            MapUpdater, // Exclusive type (implements IntoWorldEvent) to update map
        );
    }

    pub fn new() -> Self {
        let (max_c, max_l) = size().unwrap();
        let mut game = Self {
            world: RefCell::new(World::new(max_c, max_l)),
            events: Vec::new(),
        };

        game.setup_basic_events();
        game
    }

    pub fn setup_event(&mut self, setup: impl EventSetup<'g, Game<'g>>) {
        setup.setup_event(self);
    }

    /// Adds a timer with a job for every ticks.
    ///
    /// The job is a [`TimerEventHandler`] which can accepts both
    /// [`TimerKey`] and [`&mut World`] or just [`&mut World`] or anything that
    /// implements [`IntoTimerEventHandler`].
    ///
    /// You can use [`add_raw_timer`] to add timer without any job on ticks but that
    /// would be useless. You may want to use [`add_event`] to attach an event to the timer.
    pub fn add_timer<Params>(
        &mut self,
        timer: Timer,
        on_elapsed: impl IntoTimerEventHandler<'g, Params>,
    ) {
        let is_repeat = timer.data.is_repeat();
        let timer_key = self.add_raw_timer(timer);

        self.add_event(WorldBuilder::new(
            WorldEventTrigger::TimerElapsed(timer_key.clone()),
            is_repeat,
            on_elapsed.into_event_handler(timer_key.clone()),
        ));
    }

    /// Runs main game loop.
    pub fn game_loop(&mut self, stdout: &mut Stdout, slowness: u64) -> Result<(), std::io::Error> {
        while self.world.borrow().player.status == PlayerStatus::Alive {
            let world_status = self.world.borrow().status;
            match world_status {
                WorldStatus::Fluent => {
                    self.run_events();

                    let new_events: Vec<WorldBuilder<'g>> =
                        self.world.borrow_mut().new_events.drain(0..).collect();
                    for event in new_events {
                        self.add_event(event)
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

    pub fn prepare_terminal(sc: &mut Stdout) -> io::Result<()> {
        sc.execute(Hide)?;
        enable_raw_mode()
    }

    pub fn release_terminal(sc: &mut Stdout) -> io::Result<()> {
        sc.clear_all()?.execute(Show)?;
        disable_raw_mode()
    }
}

impl<'g> EventContainer<'g> for Game<'g> {
    fn add_event(&mut self, event: impl Event<'g> + 'g) {
        self.events.push(event.into_world_event());
    }
}

impl<'g> TimerContainer<'g> for Game<'g> {
    fn add_raw_timer(&mut self, timer: Timer) -> crate::timer::TimerKey {
        self.world.borrow_mut().add_raw_timer(timer)
    }
}
