use std::{
    cell::RefCell,
    collections::HashMap,
    time::{Duration, Instant},
};

use crossterm::style::ContentStyle;
use rand::{rngs::ThreadRng, thread_rng};
use uuid::Uuid;

use crate::{
    canvas::Canvas,
    entities::{Entity, Player},
    utilities::{
        container::Container,
        drawable::Drawable,
        event_handler::{EventHandler, IntoEventHandler, IntoTimerEventHandler, TimerKey},
        restorable::Restorable,
    },
};

use self::map::Map;

mod drawings;
pub mod events;
pub mod map;

pub struct WorldTimer {
    pub duration: Duration,
    pub repeat: bool,
    pub instant: Instant,
}

impl WorldTimer {
    pub fn new(duration: Duration, repeat: bool) -> Self {
        Self {
            repeat,
            duration,
            instant: Instant::now(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum WorldStatus {
    Fluent,
    Solid,
}

#[allow(dead_code)]
pub enum WorldEventTrigger<'g> {
    GameStarted,
    Anything,
    Traveled(u16),
    TimerElapsed(TimerKey),
    DrawingExists(String),
    Custom(Box<dyn Fn(&World) -> bool + 'g>),
}

impl<'g> WorldEventTrigger<'g> {
    #[allow(dead_code)]
    pub fn timer_elapsed(timer_key: impl Into<TimerKey>) -> Self {
        Self::TimerElapsed(timer_key.into())
    }

    #[allow(dead_code)]
    pub fn custom(trigger: impl Fn(&World) -> bool + 'g) -> Self {
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
    pub trigger: WorldEventTrigger<'g>,
    pub is_continues: bool,
    pub handler: EventHandler<'g>,
}

impl<'g> WorldEvent<'g> {
    pub fn new(
        trigger: WorldEventTrigger<'g>,
        is_continues: bool,
        handler: impl IntoEventHandler<'g>,
    ) -> Self {
        Self {
            trigger,
            handler: handler.into_event_handler(),
            is_continues,
        }
    }
}

/// The [`World`]! Contains everything except events.
pub struct World<'g> {
    pub canvas: Canvas,
    pub status: WorldStatus,
    pub player: Player,
    pub map: Map,
    pub container: Container<u16>,

    pub enemies_armor: u16,
    pub enemy_spawn_probability: Restorable<f32>,
    pub fuel_spawn_probability: Restorable<f32>,

    pub entities: Vec<Entity>,
    pub rng: ThreadRng, // Local rng for the whole world

    pub elapsed_time: usize,
    pub elapsed_loops: usize,
    pub timers: RefCell<HashMap<String, WorldTimer>>, // RefCell for interior mutability
    pub custom_drawings: HashMap<String, Box<dyn Drawable>>,

    /// Events that may be added inside game loops
    pub new_events: Vec<WorldEvent<'g>>,
}

impl<'g> World<'g> {
    pub fn new(maxc: u16, maxl: u16) -> World<'g> {
        World {
            elapsed_time: 0,
            elapsed_loops: 0,
            status: WorldStatus::Fluent,
            canvas: Canvas::new(maxc, maxl),
            player: Player::new((maxc / 2, maxl - 1), 1700),
            map: Map::new(maxc, maxl, 5, maxc / 3, 2, 5),
            container: Container::new(0..maxl, 0..maxc),
            entities: Vec::new(),
            rng: thread_rng(),
            timers: RefCell::new(HashMap::new()),
            custom_drawings: HashMap::new(),
            enemies_armor: 1,
            enemy_spawn_probability: 0.1.into(),
            fuel_spawn_probability: 0.01.into(),
            new_events: Vec::new(),
        }
    }

    pub fn max_l(&self) -> u16 {
        self.container.lines().end
    }

    pub fn max_c(&self) -> u16 {
        self.container.columns().end
    }

    pub fn enemies(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter().filter(|e| e.entity_type.is_enemy())
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

    /// Adds just a timer.
    ///
    /// You may want to use [`add_event`] to attach an event to the timer.
    pub fn add_raw_timer(&mut self, timer: WorldTimer) -> TimerKey {
        let key: String = Uuid::new_v4().to_string();
        self.timers.get_mut().insert(key.clone(), timer);
        TimerKey::new(key)
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
        timer: WorldTimer,
        on_elapsed: impl IntoTimerEventHandler<'g, Params>,
    ) {
        let is_repeat = timer.repeat;
        let timer_key = self.add_raw_timer(timer);

        self.add_event(WorldEvent::new(
            WorldEventTrigger::TimerElapsed(timer_key.clone()),
            is_repeat,
            on_elapsed.into_event_handler(timer_key),
        ));
    }

    /// Manually reset a timer.
    pub fn reset_timer(&mut self, timer_key: &str) -> Option<bool> {
        let timer = self.timers.get_mut().get_mut(timer_key)?;
        timer.instant = Instant::now();
        Some(true)
    }

    /// Adds a custom drawing to the screen.
    ///
    /// Drawing can then be cleared using guess what?
    pub fn add_drawing(&mut self, key: impl Into<String>, drawing: impl Drawable + 'static) {
        self.custom_drawings.insert(key.into(), Box::new(drawing));
    }

    /// Clears a previously added drawing.
    pub fn clear_drawing(&mut self, key: &str) {
        self.custom_drawings.remove(key);
    }

    /// Adds an event handler to the [`Game`].
    ///
    /// The event is added to the game at the end of current loop and NOT instantly!
    pub fn add_event(&mut self, event: WorldEvent<'g>) {
        self.new_events.push(event);
    }

    /// Shows a temporary popup with custom style and a job after its disposal.
    ///
    /// The job after popup is a [`TimerEventHandler`] which can accepts both
    /// [`TimerKey`] and [`&mut World`] or just [`&mut World`] or anything that
    /// implements [`IntoTimerEventHandler`].
    ///
    /// ## Example
    /// ```rust
    /// world.temp_popup("Hello World!", Duration::from_secs(5), LeaveAlone, None)
    /// ```
    /// [`LeaveAlone`] is an [`IntoTimerEventHandler`] which dose nothing.
    pub fn temp_popup<Params>(
        &mut self,
        message: impl Into<String>,
        duration: Duration,
        after: impl IntoTimerEventHandler<'g, Params>,
        style: impl Into<Option<ContentStyle>>,
    ) {
        let key = Uuid::new_v4().to_string();
        self.add_drawing(&key, self.popup(message, style));
        let handler = after.into_timer_event_handler();
        self.add_timer(
            WorldTimer::new(duration, false),
            move |timer_key, w: &mut World| {
                w.clear_drawing(&key);
                handler.handle(timer_key, w);
            },
        );
    }
} // end of World implementation.
