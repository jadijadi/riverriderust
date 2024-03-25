use std::time::Duration;

use rand::Rng;

use crate::{
    entities::{DeathCause, Enemy, EntityStatus, Fuel, PlayerStatus},
    game::Game,
};

use super::{map::RiverMode, World, WorldEvent, WorldEventTrigger, WorldTimer};

fn is_the_chance(probability: f32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>() < probability
}

/// check if player hit the ground
fn check_player_status(world: &mut World) {
    let player_line = world.player.location.l as usize;
    let river_border = world.map.river_borders_index(player_line);

    if !river_border.contains(&world.player.location.c) {
        world.player.status = PlayerStatus::Dead(DeathCause::Ground);
    }

    if world.player.gas == 0 {
        world.player.status = PlayerStatus::Dead(DeathCause::Fuel);
    }
}

/// check enemy hit something
fn check_enemy_status(world: &mut World) {
    // Remove dead
    world
        .enemies
        .retain(|f| !matches!(f.status, EntityStatus::Dead));

    for enemy in world.enemies.iter_mut().rev() {
        match enemy.status {
            EntityStatus::Alive if world.player.location.hit(&enemy.location) => {
                world.player.status = PlayerStatus::Dead(DeathCause::Enemy);
            }
            EntityStatus::DeadBody => {
                enemy.status = EntityStatus::Dead;
            }
            _ => {}
        }

        for bullet in world.bullets.iter().rev() {
            if bullet.location.hit_with_margin(&enemy.location, 1, 0, 1, 0) {
                enemy.status = EntityStatus::DeadBody;
                world.player.score += 10;
            }
        }
    }
}

/// Move enemies on the river
fn move_enemies(world: &mut World) {
    world.enemies.retain_mut(|enemy| {
        enemy.location.l += 1;
        // Retain enemies within the screen
        enemy.location.l < world.maxl
    });
}

/// Move Bullets
fn move_bullets(world: &mut World) {
    for index in (0..world.bullets.len()).rev() {
        if world.bullets[index].energy == 0 || world.bullets[index].location.l <= 2 {
            world.bullets.remove(index);
        } else {
            world.bullets[index].location.l -= 2;
            world.bullets[index].energy -= 1;

            let bullet_line = world.bullets[index].location.l as usize;
            let river_border = world.map.river_borders_index(bullet_line);
            if !river_border.contains(&world.bullets[index].location.c) {
                world.bullets.remove(index);
            }
        }
    }
}

/// check if fuel is hit / moved over
fn check_fuel_status(world: &mut World) {
    // Remove dead
    world
        .fuels
        .retain(|f| !matches!(f.status, EntityStatus::Dead));

    for fuel in world.fuels.iter_mut().rev() {
        match fuel.status {
            EntityStatus::Alive if world.player.location.hit(&fuel.location) => {
                fuel.status = EntityStatus::DeadBody;
                world.player.gas += 200;
            }
            EntityStatus::DeadBody => {
                fuel.status = EntityStatus::Dead;
            }
            _ => {}
        }

        for bullet in world.bullets.iter().rev() {
            if bullet.location.hit_with_margin(&fuel.location, 1, 0, 1, 0) {
                fuel.status = EntityStatus::DeadBody;
                world.player.score += 20;
            }
        }
    }
}

/// Create a new fuel; maybe
fn create_fuel(world: &mut World) {
    // Possibility
    let river_border = world.map.river_borders_index(0);
    if is_the_chance(world.fuel_spawn_probability) {
        world.fuels.push(Fuel::new(
            world.rng.gen_range(river_border),
            0,
            EntityStatus::Alive,
        ));
    }
}

/// Create a new enemy
fn create_enemy(world: &mut World) {
    // Possibility
    let river_border = world.map.river_borders_index(0);
    if is_the_chance(world.enemy_spawn_probability) {
        world.enemies.push(Enemy::new(
            world.rng.gen_range(river_border),
            0,
            EntityStatus::Alive,
        ));
    }
}

/// Move fuels on the river
fn move_fuel(world: &mut World) {
    world.fuels.retain_mut(|fuel| {
        fuel.location.l += 1;
        // Retain fuels within the screen
        fuel.location.l < world.maxl
    });
}

impl<'g> Game<'g> {
    pub fn setup_event_handlers(&mut self) {
        // ---- Permanent event, running on every loop (is_continues: true) ----
        // check if player hit the ground
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            check_player_status,
        ));

        // check enemy hit something
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            check_enemy_status,
        ));
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            check_fuel_status,
        ));

        // move the map Downward
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            |world| world.map.update(&mut world.rng),
        ));

        // create new enemy
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            create_enemy,
        ));
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            create_fuel,
        ));

        // Move elements along map movements
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            move_enemies,
        ));

        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            move_fuel,
        ));
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            move_bullets,
        ));

        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            |world| {
                if world.player.gas >= 1 {
                    world.player.gas -= 1;
                }
            },
        ));

        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            |world| {
                world.player.traveled += 1;
            },
        ));

        // ---- Temporary events: Triggered on specified conditions (is_continues: false) ----
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::GameStarted,
            false,
            |world| {
                world.map.change_river_mode(RiverMode::ConstWidthAndCenter {
                    width: world.maxc / 2,
                    center_c: world.maxc / 2,
                });

                world.temp_popup("Warmup", Duration::from_secs(10), |world| {
                    world.temp_popup("Ready !!", Duration::from_secs(2), |world| {
                        world.temp_popup("!!! GO !!!", Duration::from_secs(1), |world| {
                            world.map.change_river_mode(RiverMode::Random {
                                min_width: 5,
                                max_width: world.maxc / 3,
                                max_center_diff: 5,
                            });

                            world.fuel_spawn_probability = 0.01;
                            world.enemy_spawn_probability = 0.1;

                            world.add_timer(
                                "duration_score",
                                WorldTimer::new(Duration::from_secs(10), true),
                                |world| {
                                    world.player.score += 10;
                                },
                            );
                        })
                    });
                });
            },
        ));
    }
}
