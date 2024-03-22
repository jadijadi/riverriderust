use std::{io::Stdout, thread, time::Duration};

use rand::{rngs::ThreadRng, thread_rng};

use crate::{
    canvas::Canvas,
    entities::{Bullet, Enemy, Fuel, Location, Player, PlayerStatus},
    handle_pressed_keys,
};

use self::map::Map;

mod drawings;
mod map;
mod physics;

pub enum WorldStatus {
    Fluent,
    Paused,
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
    pub enemies: Vec<Enemy>,
    pub fuels: Vec<Fuel>,
    pub bullets: Vec<Bullet>,
    pub rng: ThreadRng, // Local rng for the whole world
}

impl World {
    pub fn new(maxc: u16, maxl: u16) -> World {
        let mut rng = thread_rng();
        World {
            status: WorldStatus::Fluent,
            canvas: Canvas::new(maxc, maxl),
            player: Player {
                location: Location::new(maxc / 2, maxl - 1),
                status: PlayerStatus::Alive,
                score: 0,
                gas: 1700,
            },
            map: Map::new(maxc, maxl, 5, maxc / 3, 1, &mut rng),
            maxc,
            maxl,
            next_left: maxc / 2 - 7,
            next_right: maxc / 2 + 7,
            enemies: Vec::new(),
            bullets: Vec::new(),
            fuels: Vec::new(),
            rng: thread_rng(),
        }
    }

    pub fn game_loop(&mut self, stdout: &mut Stdout, slowness: u64) -> Result<(), std::io::Error> {
        while self.player.status == PlayerStatus::Alive {
            handle_pressed_keys(self);
            match self.status {
                WorldStatus::Fluent => {
                    self.physics();
                    self.draw_on_canvas();
                }
                WorldStatus::Paused => self.pause_screen(),
            }

            self.canvas.draw_map(stdout)?;
            thread::sleep(Duration::from_millis(slowness));
        }

        Ok(())
    }
} // end of World implementation.
