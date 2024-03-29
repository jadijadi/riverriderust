use std::time::Duration;

use crossterm::style::{ContentStyle, Stylize};
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
fn update_player_status(world: &mut World) {
    if !world.map.is_in_river(&world.player) {
        world.player.status = PlayerStatus::Dead(DeathCause::Ground);
        return;
    }

    if world.player.fuel == 0 {
        world.player.status = PlayerStatus::Dead(DeathCause::Fuel);
    }
}

/// check enemy hit something
fn update_enemy_status(world: &mut World) {
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
                enemy.armor -= 1;
                if enemy.armor <= 0 {
                    enemy.status = EntityStatus::DeadBody;
                    world.player.score += 10;
                }
            }
        }
    }
}

/// Move enemies on the river
fn move_enemies(world: &mut World) {
    world.enemies.retain_mut(|enemy| {
        enemy.location.go_down();
        // Retain enemies within the screen
        world.container.is_upper_loc(enemy)
    });
}

/// Move Bullets
fn move_bullets(world: &mut World) {
    world.bullets.retain_mut(|bullet| {
        if bullet.energy == 0 || bullet.location.line < 2 {
            false
        } else {
            bullet.location.go_up().go_up();
            bullet.energy -= 1;

            world.map.is_in_river(bullet)
        }
    })
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
                world.player.fuel += 200;
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
    let river_border = world.map.river_borders_at(0);
    if is_the_chance(world.fuel_spawn_probability.value) {
        world.fuels.push(Fuel::new(
            (world.rng.gen_range(river_border), 0),
            EntityStatus::Alive,
        ));
    }
}

/// Create a new enemy
fn create_enemy(world: &mut World) {
    // Possibility
    let river_border = world.map.river_borders_at(0);
    if is_the_chance(world.enemy_spawn_probability.value) {
        world.enemies.push(Enemy::new(
            (world.rng.gen_range(river_border), 0),
            world.enemies_armor,
        ));
    }
}

/// Move fuels on the river
fn move_fuel(world: &mut World) {
    world.fuels.retain_mut(|fuel| {
        fuel.location.go_down();
        // Retain fuels within the screen
        world.container.is_upper_loc(fuel)
    });
}

impl<'g> Game<'g> {
    pub fn setup_event_handlers(&mut self) {
        // ---- Permanent event, running on every loop (is_continues: true) ----
        // check if player hit the ground
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            update_player_status,
        ));

        // check enemy hit something
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            update_enemy_status,
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
                if world.player.fuel >= 1 {
                    world.player.fuel -= 1;
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

        // At this point it's very simple to add stages to the game, using events.
        // - This's an example: Every 60 sec move river to center
        //      then go back to normal and increase enemies spawn chance.
        self.add_timer(
            WorldTimer::new(Duration::from_secs(60), true),
            move |timer_key, world| {
                world.map.change_river_mode(RiverMode::ConstWidthAndCenter {
                    width: world.max_c() / 3,
                    center_c: world.max_c() / 2,
                });

                world.temp_popup(
                    "More enemies ...",
                    Duration::from_secs(1),
                    |_, _| {},
                    ContentStyle::new().black().on_yellow(),
                );

                world.add_timer(
                    WorldTimer::new(Duration::from_secs(10), false),
                    move |_, world| {
                        world.reset_timer(&timer_key);
                        if world.enemy_spawn_probability.value < 1.0 {
                            world.enemy_spawn_probability.value += 0.1;
                        }
                        world.map.restore_river_mode();
                    },
                );
            },
        );

        // Improve enemies armor by 1 every 60 (so difficult)
        // self.add_timer(
        //     WorldTimer::new(Duration::from_secs(60), true),
        //     |_, world| {
        //         world.temp_popup(
        //             "Stronger enemies",
        //             Duration::from_secs(1),
        //             |_, _| {},
        //             ContentStyle::new().black().on_red(),
        //         );

        //         world.enemies_armor += 1;
        //     },
        // );

        // Update elapsed time every 1 sec
        self.add_timer(WorldTimer::new(Duration::from_secs(1), true), |_, world| {
            world.elapsed_time += 1;
        });

        // ---- Temporary events: Triggered on specified conditions (is_continues: false) ----

        // Opening events and popups
        let style = ContentStyle::new().green().on_magenta();
        self.add_event_handler(WorldEvent::new(
            WorldEventTrigger::GameStarted,
            false,
            move |world| {
                world.enemy_spawn_probability.value = 0.0;
                world.fuel_spawn_probability.value = 0.0;

                world.map.change_river_mode(RiverMode::ConstWidthAndCenter {
                    width: world.max_c() / 2,
                    center_c: world.max_c() / 2,
                });

                world.temp_popup(
                    "Warmup",
                    Duration::from_secs(5),
                    move |_, world| {
                        world.temp_popup(
                            "Ready !!",
                            Duration::from_secs(2),
                            move |_, world| {
                                world.temp_popup(
                                    "!!! GO !!!",
                                    Duration::from_secs(1),
                                    |_, world| {
                                        world.map.restore_river_mode();
                                        world.fuel_spawn_probability.restore();
                                        world.enemy_spawn_probability.restore();

                                        world.add_timer(
                                            WorldTimer::new(Duration::from_secs(10), true),
                                            |_, world| {
                                                world.player.score += 10;
                                            },
                                        );
                                    },
                                    style,
                                )
                            },
                            style,
                        );
                    },
                    style,
                );
            },
        ));
    }
}
