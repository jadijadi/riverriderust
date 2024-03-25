use std::{
    cell::RefCell,
    collections::HashMap,
    time::{self, Duration, Instant},
};

use rand::{rngs::ThreadRng, thread_rng};
use uuid::Uuid;

use crate::{
    canvas::Canvas,
    drawable::Drawable,
    entities::{Bullet, Enemy, Fuel, Location, Player, PlayerStatus},
};

use self::map::Map;

mod drawings;
pub mod events;
pub mod map;

pub struct WorldTimer {
    pub duration: Duration,
    pub repeat: bool,
    pub instant: time::Instant,
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

#[derive(Clone, Copy)]
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
    DrawingExists(String),
    Custom(Box<dyn Fn(&World) -> bool>),
}

impl WorldEventTrigger {
    pub fn timer_elapsed(timer_key: impl Into<String>) -> Self {
        Self::TimerElapsed(timer_key.into())
    }

    #[allow(dead_code)]
    pub fn custom(trigger: impl Fn(&World) -> bool + 'static) -> Self {
        Self::Custom(Box::new(trigger))
    }

    pub fn is_triggered(&self, world: &World) -> bool {
        match self {
            WorldEventTrigger::Anything => true,
            WorldEventTrigger::Traveled(distance) => &world.player.traveled >= distance,
            WorldEventTrigger::TimerElapsed(key) => world.timer_elapsed(key).unwrap_or(false),
            WorldEventTrigger::GameStarted => world.elapsed_loops <= 0,
            WorldEventTrigger::Custom(trigger) => trigger(world),
            WorldEventTrigger::DrawingExists(key) => world.custom_drawings.contains_key(key),
        }
    }
}

pub struct WorldEvent<'g> {
    pub trigger: WorldEventTrigger,
    pub is_continues: bool,
    pub handler: Box<dyn Fn(&mut World) + 'g>,
}

impl<'g> WorldEvent<'g> {
    /// Will create a continues event handler.
    pub fn new(
        trigger: WorldEventTrigger,
        is_continues: bool,
        handler: impl Fn(&mut World) + 'g,
    ) -> Self {
        Self {
            trigger,
            handler: Box::new(handler),
            is_continues,
        }
    }
}

pub struct World<'g> {
    pub canvas: Canvas,
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

    pub elapsed_loops: usize,
    pub timers: RefCell<HashMap<String, WorldTimer>>, // RefCell for interior mutability
    pub custom_drawings: HashMap<String, Box<dyn Drawable>>,

    /// Events that may be added inside game loops
    pub new_events: Vec<WorldEvent<'g>>,
}

impl<'g> World<'g> {
    pub fn new(maxc: u16, maxl: u16) -> World<'g> {
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
            custom_drawings: HashMap::new(),
            enemy_spawn_probability: 0.0,
            fuel_spawn_probability: 0.0,
            new_events: Vec::new(),
        }
    }

    pub fn timer_elapsed(&self, key: &str) -> Option<bool> {
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

    pub fn add_timer(
        &mut self,
        key: impl Into<String>,
        timer: WorldTimer,
        on_elapsed: impl Fn(&mut World) + 'g,
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

    pub fn add_drawing(&mut self, key: impl Into<String>, drawing: impl Drawable + 'static) {
        self.custom_drawings.insert(key.into(), Box::new(drawing));
    }

    pub fn clear_drawing(&mut self, key: &str) {
        self.custom_drawings.remove(key);
    }

    pub fn add_event_handler(&mut self, event: WorldEvent<'g>) {
        self.new_events.push(event);
    }

    pub fn temp_popup(
        &mut self,
        message: impl Into<String>,
        duration: Duration,
        after: impl Fn(&mut World) + 'g,
    ) {
        let key = Uuid::new_v4().to_string();
        self.add_drawing(&key, self.popup(message));
        self.add_timer(&key.clone(), WorldTimer::new(duration, false), move |w| {
            w.clear_drawing(&key);
            after(w);
        });
    }
} // end of World implementation.
