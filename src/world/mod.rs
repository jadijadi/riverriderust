use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
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
        event_handler::{EventHandler, IntoEventHandler, IntoTimerEventHandler, TimerEventHandler},
        restorable::Restorable,
    },
};

use self::map::Map;

mod drawings;
pub mod events;
pub mod map;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimerKey(String);

impl From<String> for TimerKey {
    fn from(value: String) -> Self {
        TimerKey(value)
    }
}

impl std::ops::Deref for TimerKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct WorldTimerData {
    key: TimerKey,
    duration: Duration,
    repeat: bool,
}

impl WorldTimerData {
    pub fn new(duration: Duration, repeat: bool) -> Self {
        Self {
            key: TimerKey(Uuid::new_v4().to_string()),
            duration,
            repeat,
        }
    }
}

impl WorldTimerData {
    pub fn key(&self) -> &TimerKey {
        &self.key
    }

    #[allow(dead_code)]
    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    pub fn is_repeat(&self) -> bool {
        self.repeat
    }
}

#[derive(Debug)]
pub struct WorldTimer {
    pub data: WorldTimerData,
    instant: Instant,
}

impl WorldTimer {
    pub fn new(duration: Duration, repeat: bool) -> Self {
        Self {
            data: WorldTimerData::new(duration, repeat),
            instant: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> bool {
        self.instant.elapsed() > self.data.duration
    }

    pub fn reset(&mut self) {
        log::info!("Timer {:?} reset.", self.data.key);
        self.instant = Instant::now();
    }

    fn on_elapsed_event<'g, Params>(
        &self,
        handler: impl IntoTimerEventHandler<'g, Params>,
    ) -> TimerElapsedEvent<'g> {
        TimerElapsedEvent::new(self.data.clone(), handler)
    }
}

#[derive(Clone, Copy)]
pub enum WorldStatus {
    Fluent,
    Solid,
}

#[derive(Clone)]
pub struct EventTrigger {
    trigger: Rc<dyn Fn(&World) -> bool>,
}

impl EventTrigger {
    pub fn new(trigger: impl Fn(&World) -> bool + 'static) -> Self {
        Self {
            trigger: Rc::new(trigger),
        }
    }

    pub fn is_triggered(&self, world: &World) -> bool {
        (self.trigger)(world)
    }
}

pub trait IntoEventTrigger {
    fn into_event_trigger(self) -> EventTrigger;
}

impl IntoEventTrigger for EventTrigger {
    fn into_event_trigger(self) -> EventTrigger {
        self
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum WorldEventTrigger {
    Always,
    GameStarted,
    Traveled(u16),
    TimerElapsed(TimerKey),
    DrawingExists(String),
    Signal(String),
}

impl WorldEventTrigger {
    #[allow(dead_code)]
    pub fn timer_elapsed(timer_key: impl Into<TimerKey>) -> Self {
        Self::TimerElapsed(timer_key.into())
    }

    pub fn is_triggered(&self, world: &World) -> bool {
        match self {
            WorldEventTrigger::Always => true,
            WorldEventTrigger::Traveled(distance) => &world.player.traveled >= distance,
            WorldEventTrigger::TimerElapsed(key) => world.timer_elapsed(key).unwrap_or(false),
            WorldEventTrigger::GameStarted => world.elapsed_loops <= 0,
            WorldEventTrigger::DrawingExists(key) => world.custom_drawings.contains_key(key),
            WorldEventTrigger::Signal(signal_key) => world.signaled(signal_key),
        }
    }
}

impl IntoEventTrigger for WorldEventTrigger {
    fn into_event_trigger(self) -> EventTrigger {
        EventTrigger::new(move |world| self.is_triggered(world))
    }
}

pub struct WorldEvent<'g> {
    pub trigger: EventTrigger,
    pub is_continues: bool,
    pub handler: EventHandler<'g>,
}

impl<'g> WorldEvent<'g> {
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

    fn into_world_event(self) -> WorldEvent<'g>
    where
        Self: Sized + 'g,
    {
        let trigger = self.trigger().into_event_trigger();
        let is_continues = self.is_continues();
        let handler = self.handler().into_event_handler();
        WorldEvent::new(trigger, is_continues, handler)
    }
}

impl<'g> Event<'g> for WorldEvent<'g> {
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

pub struct TimerElapsedEvent<'g> {
    timer: WorldTimerData,
    handler: TimerEventHandler<'g>,
}

impl<'g> TimerElapsedEvent<'g> {
    pub fn new<Params>(
        timer: WorldTimerData,
        handler: impl IntoTimerEventHandler<'g, Params>,
    ) -> Self {
        Self {
            timer,
            handler: handler.into_timer_event_handler(),
        }
    }
}

impl<'g> Event<'g> for TimerElapsedEvent<'g> {
    fn is_continues(&self) -> bool {
        self.timer.is_repeat()
    }

    fn trigger(&self) -> impl IntoEventTrigger {
        WorldEventTrigger::TimerElapsed(self.timer.key.clone())
    }

    fn handler(self) -> impl IntoEventHandler<'g> {
        self.handler.into_event_handler(self.timer.key)
    }
}

pub trait IntoEvent<'g> {
    fn into_event(self, handler: impl IntoEventHandler<'g>) -> impl Event<'g>;
}

impl<'g, T: Event<'g> + 'g> IntoEvent<'g> for T {
    fn into_event(self, _: impl IntoEventHandler<'g>) -> impl Event<'g> {
        self.into_world_event()
    }
}

impl<'g> IntoEvent<'g> for WorldTimer {
    fn into_event(self, handler: impl IntoEventHandler<'g>) -> impl Event<'g> {
        self.on_elapsed_event(handler.into_event_handler())
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
    pub timers: RefCell<HashMap<TimerKey, WorldTimer>>, // RefCell for interior mutability
    pub custom_drawings: HashMap<String, Box<dyn Drawable>>,

    /// Events that may be added inside game loops
    pub new_events: Vec<WorldEvent<'g>>,
    pub signals: RefCell<HashSet<String>>,
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

    fn timer_elapsed(&self, key: &TimerKey) -> Option<bool> {
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

    fn signaled(&self, signal_key: &str) -> bool {
        self.signals.borrow_mut().remove(signal_key)
    }

    fn send_signal(&mut self, signal_key: impl Into<Option<String>>) -> String {
        let signal_key =
            Into::<Option<String>>::into(signal_key).unwrap_or(Uuid::new_v4().to_string());
        self.signals.borrow_mut().insert(signal_key.clone());
        signal_key
    }

    /// Adds just a timer.
    ///
    /// You may want to use [`add_event`] to attach an event to the timer.
    pub fn add_raw_timer(&mut self, timer: WorldTimer) -> TimerKey {
        let key = timer.data.key().clone();
        self.timers.get_mut().insert(key.clone(), timer);
        key
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
        self.add_event(timer.on_elapsed_event(on_elapsed));
        self.add_raw_timer(timer);
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

    /// Adds an event handler to the [`Game`].
    ///
    /// The event is added to the game at the end of current loop and NOT instantly!
    pub fn add_event(&mut self, event: impl Event<'g> + 'g) {
        self.new_events.push(event.into_world_event());
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
            WorldTimer::new(duration, false),
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
        self.add_event(WorldEvent::new(
            WorldEventTrigger::Signal(signal_key.to_string()),
            false,
            after,
        ));
        advance(self, popups.into_iter().collect(), signal_key)
    }
} // end of World implementation.
