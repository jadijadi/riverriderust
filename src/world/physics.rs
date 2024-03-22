use crate::World;

use rand::Rng;

use crate::entities::{DeathCause, Enemy, EntityStatus, Fuel, PlayerStatus};

impl World {
    /// check if player hit the ground
    fn check_player_status(&mut self) {
        let player_line = self.player.location.l as usize;
        let (left_border, right_border) = self.map.river_borders_index(player_line);
        if self.player.location.c < left_border || self.player.location.c >= right_border {
            self.player.status = PlayerStatus::Dead(DeathCause::Ground);
        }

        if self.player.gas == 0 {
            self.player.status = PlayerStatus::Dead(DeathCause::Fuel);
        }
    }

    /// check enemy hit something
    fn check_enemy_status(&mut self) {
        // Remove dead
        self.enemies
            .retain(|f| !matches!(f.status, EntityStatus::Dead));

        for enemy in self.enemies.iter_mut().rev() {
            match enemy.status {
                EntityStatus::Alive if self.player.location.hit(&enemy.location) => {
                    self.player.status = PlayerStatus::Dead(DeathCause::Enemy);
                }
                EntityStatus::DeadBody => {
                    enemy.status = EntityStatus::Dead;
                }
                _ => {}
            }

            for bullet in self.bullets.iter().rev() {
                if bullet.location.hit_with_margin(&enemy.location, 1, 0, 1, 0) {
                    enemy.status = EntityStatus::DeadBody;
                    self.player.score += 10;
                }
            }
        }
    }

    /// Update the map
    fn update_map(&mut self) {
        self.map.update(&mut self.rng)
        // use std::cmp::Ordering::*;

        // // move the map downward using VecDeque
        // self.map.pop_back();
        // let (mut left, mut right) = self.map[0];
        // match self.next_left.cmp(&left) {
        //     Greater => left += 1,
        //     Less => left -= 1,
        //     Equal => {}
        // };

        // match self.next_right.cmp(&right) {
        //     Greater => right += 1,
        //     Less => right -= 1,
        //     Equal => {}
        // };

        // if self.next_left == self.map[0].0 && self.rng.gen_range(0..10) >= 7 {
        //     self.next_left = self
        //         .rng
        //         .gen_range(self.next_left.saturating_sub(5)..self.next_left + 5);
        //     if self.next_left == 0 {
        //         self.next_left = 1;
        //     }
        // }

        // if self.next_right == self.map[0].1 && self.rng.gen_range(0..10) >= 7 {
        //     self.next_right = self.rng.gen_range(self.next_right - 5..self.next_right + 5);
        //     if self.next_right > self.maxc {
        //         self.next_right = Wrapping(self.maxc).0 - 1;
        //     }
        // }

        // if self.next_right.abs_diff(self.next_left) < 3 {
        //     self.next_right += 3;
        // }

        // self.map.push_front((left, right))
    }

    /// Move enemies on the river
    fn move_enemies(&mut self) {
        self.enemies.retain_mut(|enemy| {
            enemy.location.l += 1;
            // Retain enemies within the screen
            enemy.location.l < self.maxl
        });
    }

    /// Move Bullets
    fn move_bullets(&mut self) {
        for index in (0..self.bullets.len()).rev() {
            if self.bullets[index].energy == 0 || self.bullets[index].location.l <= 2 {
                self.bullets.remove(index);
            } else {
                self.bullets[index].location.l -= 2;
                self.bullets[index].energy -= 1;

                let bullet_line = self.bullets[index].location.l as usize;
                let (left_border, right_border) = self.map.river_borders_index(bullet_line);
                if self.bullets[index].location.c < left_border
                    || self.bullets[index].location.c >= right_border
                {
                    self.bullets.remove(index);
                }
            }
        }
    }

    /// check if fuel is hit / moved over
    fn check_fuel_status(&mut self) {
        // Remove dead
        self.fuels
            .retain(|f| !matches!(f.status, EntityStatus::Dead));

        for fuel in self.fuels.iter_mut().rev() {
            match fuel.status {
                EntityStatus::Alive if self.player.location.hit(&fuel.location) => {
                    fuel.status = EntityStatus::DeadBody;
                    self.player.gas += 200;
                }
                EntityStatus::DeadBody => {
                    fuel.status = EntityStatus::Dead;
                }
                _ => {}
            }

            for bullet in self.bullets.iter().rev() {
                if bullet.location.hit_with_margin(&fuel.location, 1, 0, 1, 0) {
                    fuel.status = EntityStatus::DeadBody;
                    self.player.score += 20;
                }
            }
        }
    }

    /// Create a new fuel; maybe
    fn create_fuel(&mut self) {
        // Possibility
        let (left_border, right_border) = self.map.river_borders_index(0);
        if self.rng.gen_range(0..100) >= 99 {
            self.fuels.push(Fuel::new(
                self.rng.gen_range(left_border..right_border),
                0,
                EntityStatus::Alive,
            ));
        }
    }

    /// Create a new enemy
    fn create_enemy(&mut self) {
        // Possibility
        let (left_border, right_border) = self.map.river_borders_index(0);
        if self.rng.gen_range(0..10) >= 9 {
            self.enemies.push(Enemy::new(
                self.rng.gen_range(left_border..right_border),
                0,
                EntityStatus::Alive,
            ));
        }
    }

    /// Move fuels on the river
    fn move_fuel(&mut self) {
        self.fuels.retain_mut(|fuel| {
            fuel.location.l += 1;
            // Retain fuels within the screen
            fuel.location.l < self.maxl
        });
    }

    pub(super) fn physics(&mut self) {
        // check if player hit the ground
        self.check_player_status();

        // check enemy hit something
        self.check_enemy_status();
        self.check_fuel_status();

        // move the map Downward
        self.update_map();

        // create new enemy
        self.create_enemy();
        self.create_fuel();

        // Move elements along map movements
        self.move_enemies();
        self.move_fuel();
        self.move_bullets();

        if self.player.gas >= 1 {
            self.player.gas -= 1;
        }
    }
}
