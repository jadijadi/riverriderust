use std::time::Duration;

use crossterm::style::{ContentStyle, Stylize};
use rand::Rng;

use crate::{
    entities::{DeathCause, Enemy, Entity, EntityStatus, EntityType, Fuel, PlayerStatus},
    game::Game,
    utilities::event_handler::{EventHandler, LeaveAlone, TimerEventHandler},
};

use super::{
    map::{MapUpdater, RiverMode},
    World, WorldEvent, WorldEventTrigger, WorldTimer,
};

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
fn update_entities_status(world: &mut World) {
    // Remove dead
    world.entities.retain(|f| {
        if let EntityType::Enemy(_) = f.entity_type {
            if f.status == EntityStatus::Dead {
                return false;
            }
        }
        true
    });

    for entity in world.entities.iter_mut().rev() {
        match entity.status {
            EntityStatus::Alive if world.player.location.hit(&entity.location) => {
                match entity.entity_type {
                    EntityType::Enemy(_) => {
                        world.player.status = PlayerStatus::Dead(DeathCause::Enemy);
                    }
                    EntityType::Fuel(_) => {
                        entity.status = EntityStatus::DeadBody;
                        world.player.fuel += 200;
                    }
                }
            }
            EntityStatus::DeadBody => {
                entity.status = EntityStatus::Dead;
            }
            _ => {}
        }

        for bullet in world.player.bullets.iter().rev() {
            if bullet
                .location
                .hit_with_margin(&entity.location, 1, 0, 1, 0)
            {
                match &mut entity.entity_type {
                    EntityType::Enemy(enemy) => {
                        enemy.armor -= 1;
                        if enemy.armor <= 0 {
                            entity.status = EntityStatus::DeadBody;
                            world.player.score += 10;
                        }
                    }
                    EntityType::Fuel(_) => {
                        entity.status = EntityStatus::DeadBody;
                        world.player.score += 20;
                    }
                }
            }
        }
    }
}

/// Move enemies on the river
fn move_entities(world: &mut World) {
    world.entities.retain_mut(|entity| {
        entity.location.go_down();
        // Retain enemies within the screen
        world.container.is_upper_loc(entity)
    });
}

/// Move Bullets
fn move_bullets(world: &mut World) {
    world.player.bullets.retain_mut(|bullet| {
        if bullet.energy == 0 || bullet.location.line < 2 {
            false
        } else {
            bullet.location.go_up().go_up();
            bullet.energy -= 1;

            world.map.is_in_river(bullet)
        }
    })
}

/// Create a new fuel; maybe
fn create_random_entities(world: &mut World) {
    // Possibility
    let river_border = world.map.river_borders_at(0);

    if is_the_chance(world.fuel_spawn_probability.value) {
        world.entities.push(Entity::new(
            (world.rng.gen_range(river_border.clone()), 0),
            Fuel,
        ));
    }

    if is_the_chance(world.enemy_spawn_probability.value) {
        world.entities.push(Entity::new(
            (world.rng.gen_range(river_border), 0),
            Enemy::new(world.enemies_armor),
        ));
    }
}

impl<'g> Game<'g> {
    pub fn setup_event_handlers(&mut self) {
        // ---- Permanent event, running on every loop (is_continues: true) ----
        // check if player hit the ground
        self.add_event(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            update_player_status,
        ));

        // check enemy hit something
        self.add_event(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            update_entities_status,
        ));

        // move the map Downward
        self.add_event(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            MapUpdater, // Exclusive type (implements EventHandler) to update map
        ));

        self.add_event(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            create_random_entities,
        ));

        // Move elements along map movements
        self.add_event(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            move_entities,
        ));

        self.add_event(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            move_bullets,
        ));

        self.add_event(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            EventHandler::new(|world| {
                if world.player.fuel >= 1 {
                    world.player.fuel -= 1;
                }
            }),
        ));

        self.add_event(WorldEvent::new(
            WorldEventTrigger::Anything,
            true,
            // Instead of using EventHandler::new(...)
            |world: &mut World| {
                world.player.traveled += 1;
            },
        ));

        // At this point it's very simple to add stages to the game, using events.
        // - This's an example: Every 60 sec move river to center
        //      then go back to normal and increase enemies spawn chance.
        self.add_timer(
            WorldTimer::new(Duration::from_secs(60), true),
            TimerEventHandler::new(move |timer_key, world| {
                world.map.change_river_mode(RiverMode::ConstWidthAndCenter {
                    width: world.max_c() / 3,
                    center_c: world.max_c() / 2,
                });

                world.temp_popup(
                    "More enemies ...",
                    Duration::from_secs(1),
                    LeaveAlone,
                    ContentStyle::new().black().on_yellow(),
                );

                world.add_timer(
                    WorldTimer::new(Duration::from_secs(10), false),
                    // Instead of using TimerEventHandler::new(...)
                    move |world: &mut World| {
                        world.reset_timer(&timer_key);
                        if world.enemy_spawn_probability.value < 1.0 {
                            world.enemy_spawn_probability.value += 0.1;
                        }
                        world.map.restore_river_mode();
                    },
                );
            }),
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
        self.add_timer(
            WorldTimer::new(Duration::from_secs(1), true),
            |world: &mut World| {
                world.elapsed_time += 1;
            },
        );

        // ---- Temporary events: Triggered on specified conditions (is_continues: false) ----

        // Opening events and popups
        let style = ContentStyle::new().green().on_magenta();
        self.add_event(WorldEvent::new(
            WorldEventTrigger::GameStarted,
            false,
            move |world: &mut World| {
                world.enemy_spawn_probability.value = 0.0;
                world.fuel_spawn_probability.value = 0.0;

                world.map.change_river_mode(RiverMode::ConstWidthAndCenter {
                    width: world.max_c() / 2,
                    center_c: world.max_c() / 2,
                });

                world.temp_popup(
                    "Warmup",
                    Duration::from_secs(5),
                    move |world: &mut World| {
                        world.temp_popup(
                            "Ready !!",
                            Duration::from_secs(2),
                            move |world: &mut World| {
                                world.temp_popup(
                                    "!!! GO !!!",
                                    Duration::from_secs(1),
                                    |world: &mut World| {
                                        world.map.restore_river_mode();
                                        world.fuel_spawn_probability.restore();
                                        world.enemy_spawn_probability.restore();

                                        world.add_timer(
                                            WorldTimer::new(Duration::from_secs(10), true),
                                            |_, world: &mut World| {
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
