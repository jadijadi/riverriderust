use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

use crossterm::style::ContentStyle;
use rand::{rngs::ThreadRng, thread_rng};
use uuid::Uuid;

use crate::{
    canvas::Canvas,
    entities::{Entity, Player},
    events::{
        handlers::{EventHandler, IntoEventHandler, IntoTimerEventHandler},
        setup::{EventContainer, EventSetup, TimerContainer},
        triggers::WorldEventTrigger,
        Event, WorldBuilder,
    },
    timer::{Timer, TimerEventSetup, TimerKey},
    utilities::{container::Container, drawable::Drawable, restorable::Restorable},
};

use self::map::Map;

mod drawings;
pub mod map;

#[derive(Clone, Copy)]
pub enum WorldStatus {
    Fluent,
    Solid,
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
    pub timers: RefCell<HashMap<TimerKey, Timer>>, // RefCell for interior mutability
    pub custom_drawings: HashMap<String, Box<dyn Drawable>>,

    /// Events that may be added inside game loops
    pub new_events: Vec<WorldBuilder<'g>>,
    pub signals: RefCell<HashSet<String>>,
}

impl<'g> World<'g> {
    pub(crate) fn timer_elapsed(&self, key: &TimerKey) -> Option<bool> {
        let mut timers = self.timers.borrow_mut();
        let timer = match timers.get_mut(key) {
            Some(timer) => timer,
            None => {
                log::warn!("Checking for non-existence timer {key:?}");
                return None;
            }
        };

        if !timer.elapsed() {
            // Not expired -> keep
            Some(false)
        } else {
            log::info!("{key:?} elapsed.");
            if timer.data.is_repeat() {
                // Expired but repeat -> keep
                // Reset instant
                timer.reset();
                Some(true)
            } else {
                // Expired and no repeat -> remove
                timers.remove(key);
                Some(true)
            }
        }
    }

    pub(crate) fn signaled(&self, signal_key: &str) -> bool {
        self.signals.borrow_mut().remove(signal_key)
    }

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
            signals: RefCell::new(HashSet::new()),
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

    pub fn send_signal(&mut self, signal_key: impl Into<Option<String>>) -> String {
        let signal_key =
            Into::<Option<String>>::into(signal_key).unwrap_or(Uuid::new_v4().to_string());
        self.signals.borrow_mut().insert(signal_key.clone());
        signal_key
    }

    pub fn setup_event(&mut self, setup: impl EventSetup<'g, World<'g>>) {
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
        self.setup_event(TimerEventSetup::new(timer, on_elapsed))
    }

    /// Manually reset a timer.
    pub fn reset_timer(&mut self, timer_key: &TimerKey) -> Option<bool> {
        let timer = self.timers.get_mut().get_mut(timer_key)?;
        timer.reset();
        Some(true)
    }

    /// Adds a custom drawing to the screen. The `key` parameter is optional.
    /// If you leave it as None, a random key is generated.
    ///
    /// Return value is the key (_either specified or generated_)
    ///
    /// Drawing can then be cleared using guess what?
    pub fn add_drawing(
        &mut self,
        drawing: impl Drawable + 'static,
        key: impl Into<Option<String>>,
    ) -> String {
        let key: Option<String> = key.into();
        let key = key.unwrap_or(Uuid::new_v4().to_string());
        self.custom_drawings.insert(key.clone(), Box::new(drawing));
        key
    }

    /// Clears a previously added drawing.
    pub fn clear_drawing(&mut self, key: &str) {
        self.custom_drawings.remove(key);
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
        let key = self.add_drawing(self.popup(message, style), None);
        log::info!("Added drawing with key {key}");
        let after = after.into_timer_event_handler();
        self.add_timer(
            Timer::new(duration, false),
            move |timer_key, w: &mut World| {
                log::info!(
                    "Called popup timer elapsed with timer key {timer_key:?} for popup {key}."
                );
                w.clear_drawing(&key);
                after.handle(timer_key, w);
            },
        );
    }

    /// Adds a series of popups that are shown one ofter the other.
    /// And a job that is triggered when the series reaches its end and latest
    /// popup gone off.
    pub fn popup_series(
        &mut self,
        popups: impl IntoIterator<Item = (String, Duration, ContentStyle)>,
        after: impl IntoEventHandler<'g>,
    ) {
        fn advance(
            world: &mut World,
            mut popups: VecDeque<(String, Duration, ContentStyle)>,
            signal_key: Uuid,
        ) {
            if let Some((message, duration, style)) = popups.pop_front() {
                let handler = if popups.is_empty() {
                    EventHandler::new(move |world| {
                        world.send_signal(signal_key.to_string());
                    })
                } else {
                    EventHandler::new(move |world| advance(world, popups.clone(), signal_key))
                };
                world.temp_popup(message, duration, handler, style)
            }
        }

        let signal_key = Uuid::new_v4();
        // Waiting for a signal that is triggered when latest popup goes off.
        self.add_event(
            WorldBuilder::new(WorldEventTrigger::Signal(signal_key.to_string()))
                .with_handler(after),
        );
        advance(self, popups.into_iter().collect(), signal_key)
    }
} // end of World implementation.

impl<'g> EventContainer<'g> for World<'g> {
    fn add_event(&mut self, event: impl Event<'g> + 'g) {
        self.new_events.push(event.into_world_event());
    }
}

impl<'g> TimerContainer<'g> for World<'g> {
    fn add_raw_timer(&mut self, timer: Timer) -> TimerKey {
        let key = timer.data.key().clone();
        self.timers.get_mut().insert(key.clone(), timer);
        key
    }
}
