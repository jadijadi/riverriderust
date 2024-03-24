use std::{
    cell::RefCell,
    collections::HashMap,
    io::Stdout,
    thread,
    time::{self, Duration, Instant},
};

use rand::{rngs::ThreadRng, thread_rng};

use crate::{
    canvas::Canvas,
    drawable::Drawable,
    entities::{Bullet, Enemy, Fuel, Location, Player, PlayerStatus},
    events::handle_pressed_keys,
};

use self::map::Map;

mod drawings;
pub mod events;
pub mod map;

pub struct WorldTimer {
    duration: Duration,
    repeat: bool,
    instant: time::Instant,
}

impl WorldTimer {
    pub fn new(duration: Duration, repeat: bool) -> Self {
        Self {
            repeat,
            duration,
            instant: time::Instant::now(),
        }
    }
}

pub enum WorldStatus {
    Fluent,
    Paused,
}

#[allow(dead_code)]
pub enum WorldEventTrigger {
    GameStarted,
    Anything,
    Traveled(u16),
    TimerElapsed(String),
    Custom(Box<dyn Fn(&World) -> bool>),
}

impl WorldEventTrigger {
    #[allow(dead_code)]
    pub fn custom(trigger: impl Fn(&World) -> bool + 'static) -> Self {
        Self::Custom(Box::new(trigger))
    }
}

pub struct WorldEvent {
    trigger: WorldEventTrigger,
    is_continues: bool,
    handler: &'static dyn Fn(&mut World),
}

impl WorldEvent {
    /// Will create a continues event handler.
    pub fn new(
        trigger: WorldEventTrigger,
        is_continues: bool,
        handler: &'static dyn Fn(&mut World),
    ) -> Self {
        Self {
            trigger,
            handler,
            is_continues,
        }
    }

    pub fn is_continues(&self) -> bool {
        self.is_continues
    }
}

pub struct World {
    canvas: Canvas,
    pub status: WorldStatus,
    pub player: Player,
    pub map: Map,
    pub maxc: u16,
    pub maxl: u16,
    pub next_right: u16,
    pub next_left: u16,

    pub enemy_spawn_probability: f32,
    pub fuel_spawn_probability: f32,
    pub enemies: Vec<Enemy>,
    pub fuels: Vec<Fuel>,
    pub bullets: Vec<Bullet>,
    pub rng: ThreadRng, // Local rng for the whole world

    elapsed_loops: usize,
    timers: RefCell<HashMap<String, WorldTimer>>, // RefCell for interior mutability
    events: Vec<WorldEvent>,
    custom_drawings: HashMap<String, Box<dyn Drawable>>,
}

impl World {
    pub fn new(maxc: u16, maxl: u16) -> World {
        World {
            elapsed_loops: 0,
            status: WorldStatus::Fluent,
            canvas: Canvas::new(maxc, maxl),
            player: Player {
                location: Location::new(maxc / 2, maxl - 1),
                status: PlayerStatus::Alive,
                score: 0,
                gas: 1700,
                traveled: 0,
            },
            map: Map::new(maxc, maxl, 5, maxc / 3, 2, 5),
            maxc,
            maxl,
            next_left: maxc / 2 - 7,
            next_right: maxc / 2 + 7,
            enemies: Vec::new(),
            bullets: Vec::new(),
            fuels: Vec::new(),
            rng: thread_rng(),
            timers: RefCell::new(HashMap::new()),
            events: Vec::new(),
            custom_drawings: HashMap::new(),
            enemy_spawn_probability: 0.0,
            fuel_spawn_probability: 0.0,
        }
    }

    pub fn add_drawing(&mut self, key: impl Into<String>, drawing: impl Drawable + 'static) {
        self.custom_drawings.insert(key.into(), Box::new(drawing));
    }

    pub fn clear_drawing(&mut self, key: &str) {
        self.custom_drawings.remove(key);
    }

    pub fn add_timer(
        &mut self,
        key: impl Into<String>,
        timer: WorldTimer,
        on_elapsed: &'static dyn Fn(&mut World),
    ) {
        let is_repeat = timer.repeat;
        let key: String = key.into();
        self.timers.get_mut().insert(key.clone(), timer);
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::TimerElapsed(key),
            is_repeat,
            on_elapsed,
        ));
    }

    pub fn add_event_handler(&mut self, event: WorldEvent) {
        self.events.push(event);
    }

    fn timer_elapsed(&self, key: &str) -> Option<bool> {
        let mut timers = self.timers.borrow_mut();
        let timer = timers.get_mut(key)?;

        if timer.instant.elapsed() <= timer.duration {
            // Not expired -> keep
            Some(false)
        } else {
            if timer.repeat {
                // Expired but repeat -> keep
                // Reset instant
                timer.instant = Instant::now();
                Some(true)
            } else {
                // Expired and no repeat -> remove
                timers.remove(key);
                Some(true)
            }
        }
    }

    fn event_triggered(&self, event: &WorldEventTrigger) -> bool {
        match event {
            WorldEventTrigger::Anything => true,
            WorldEventTrigger::Traveled(distance) => &self.player.traveled >= distance,
            WorldEventTrigger::TimerElapsed(key) => self.timer_elapsed(key).unwrap_or(false),
            WorldEventTrigger::GameStarted => self.elapsed_loops <= 0,
            WorldEventTrigger::Custom(trigger) => trigger(self),
        }
    }

    fn handle_events(&mut self) {
        let mut events_index_to_clean: Vec<usize> = vec![];

        // Running events
        let triggered_events: Vec<&'static dyn Fn(&mut World)> = self
            .events
            .iter()
            .enumerate()
            .filter(|(idx, event)| {
                if self.event_triggered(&event.trigger) {
                    if !event.is_continues() {
                        events_index_to_clean.push(*idx);
                    };

                    return true;
                }
                false
            })
            .map(|(_, handlers)| handlers.handler)
            .collect();

        for handler in triggered_events {
            handler(self)
        }

        // Remove triggered un-continues events
        for idx in events_index_to_clean {
            self.events.remove(idx);
        }
    }

    pub fn game_loop(&mut self, stdout: &mut Stdout, slowness: u64) -> Result<(), std::io::Error> {
        while self.player.status == PlayerStatus::Alive {
            handle_pressed_keys(self)?;

            match self.status {
                WorldStatus::Fluent => {
                    self.handle_events();
                    // Draw drawings on canvas first
                    self.draw_on_canvas();
                }
                WorldStatus::Paused => self.pause_screen(),
            }

            // Draw canvas map into stdout.
            self.canvas.draw_map(stdout)?;

            thread::sleep(Duration::from_millis(slowness));
            self.elapsed_loops += 1;
        }

        Ok(())
    }
} // end of World implementation.
