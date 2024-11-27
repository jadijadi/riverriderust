use std::{io, time::Duration};

use crossterm::{
    event::{poll, read},
    style::{ContentStyle, Stylize},
};
use rand::Rng;

use riverraid::{
    entities::{Bullet, DeathCause, Enemy, Entity, EntityStatus, EntityType, Fuel, PlayerStatus},
    events::{
        handlers::{EventHandler, IntoEventHandler, TimerEventHandler},
        setup::IntoEventSetup,
        triggers::{IntoEventTrigger, WorldEventTrigger},
        Event, LeaveAlone, WorldBuilder,
    },
    game::Game,
    timer::Timer,
    world::{map::RiverMode, World, WorldStatus},
};

fn is_the_chance(probability: f32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>() < probability
}

fn keyboard_events(world: &mut World) -> io::Result<()> {
    if poll(Duration::from_millis(10))? {
        let key = read()?;

        while poll(Duration::from_millis(0))? {
            let _ = read();
        }

        match key {
            crossterm::event::Event::Key(event) => {
                // I'm reading from keyboard into event;
                match event.code {
                    // Movements
                    crossterm::event::KeyCode::Char('w') | crossterm::event::KeyCode::Up
                        if world.player.status == PlayerStatus::Alive
                            && world.player.location.line > 1 =>
                    {
                        world.player.go_up();
                    }
                    crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Down
                        if world.player.status == PlayerStatus::Alive
                            && world.player.location.line < world.max_l() - 1 =>
                    {
                        world.player.go_down();
                    }
                    crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Left
                        if world.player.status == PlayerStatus::Alive
                            && world.player.location.column > 1 =>
                    {
                        world.player.go_left();
                    }
                    crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Right
                        if world.player.status == PlayerStatus::Alive
                            && world.player.location.column < world.max_c() - 1 =>
                    {
                        world.player.go_right();
                    }

                    // Other events
                    crossterm::event::KeyCode::Char('q') => {
                        world.player.status = PlayerStatus::Quit
                    }
                    crossterm::event::KeyCode::Char('p')
                        if event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        use WorldStatus::*;
                        world.status = match world.status {
                            Fluent => Solid,
                            Solid => Fluent,
                        };
                    }
                    crossterm::event::KeyCode::Char(' ')
                        if event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        if world.player.status == PlayerStatus::Alive
                            && world.player.bullets.is_empty()
                        {
                            world
                                .player
                                .bullets
                                .push(Bullet::new(&world.player.location, world.max_l() / 4));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub struct PlayerStatusUpdater;

// Using this trait we can define everything about our event in one place.
// And the type which this trait is implemented for, can be directly used in add_event(...) method.
// Eg: game.add_event(PlayerStatusUpdater)
impl<'g> Event<'g> for PlayerStatusUpdater {
    fn continues(&self) -> bool {
        true
    }

    fn trigger(&self) -> impl IntoEventTrigger {
        WorldEventTrigger::Always
    }

    fn handler(self) -> impl IntoEventHandler<'g> {
        |world: &mut World| {
            if !world.map.is_in_river(&world.player) {
                world.player.status = PlayerStatus::Dead(DeathCause::Ground);
                return;
            }

            if world.player.fuel == 0 {
                world.player.status = PlayerStatus::Dead(DeathCause::Fuel);
            }
        }
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

pub fn setup_event_handlers(game: &mut Game) {
    // ---- Permanent event, running on every loop (is_continues: true) ----

    // Handle keyboard events.
    game.setup_event(
        WorldBuilder::new(WorldEventTrigger::Always)
            .is_continues()
            .with_handler(EventHandler::new(|world| {
                // Ignore errors.
                let _ = keyboard_events(world);
            })),
    );

    // check if player hit the ground
    game.setup_event(PlayerStatusUpdater);

    // check enemy hit something
    game.setup_event(
        WorldBuilder::new(WorldEventTrigger::Always)
            .is_continues()
            .with_handler(update_entities_status),
    );

    game.setup_event(
        WorldBuilder::new(WorldEventTrigger::Always)
            .is_continues()
            .with_handler(create_random_entities),
    );

    // Move elements along map movements
    game.setup_event(
        WorldBuilder::new(WorldEventTrigger::Always)
            .is_continues()
            .with_handler(move_entities),
    );

    game.setup_event(
        WorldBuilder::new(WorldEventTrigger::Always)
            .is_continues()
            .with_handler(move_bullets),
    );

    game.setup_event(
        WorldBuilder::new(WorldEventTrigger::Always)
            .is_continues()
            .with_handler(EventHandler::new(|world| {
                if world.player.fuel >= 1 {
                    world.player.fuel -= 1;
                }
            })),
    );

    game.setup_event(
        WorldBuilder::new(WorldEventTrigger::Always)
            .is_continues()
            .with_handler(|world: &mut World| {
                world.player.traveled += 1;
            }),
    );

    // At this point it's very simple to add stages to the game, using events.
    // - This's an example: Every 60 sec move river to center
    //      then go back to normal and increase enemies spawn chance.
    game.add_timer(
        Timer::new(Duration::from_secs(60), true),
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
                Timer::new(Duration::from_secs(10), false),
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
    // game.add_timer(
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
    game.setup_event(Timer::new(Duration::from_secs(1), true).into_event_setup(
        |world: &mut World| {
            world.elapsed_time += 1;
        },
    ));

    // ---- Temporary events: Triggered on specified conditions (is_continues: false) ----

    // Opening events and popups
    let style = ContentStyle::new().green().on_magenta();
    game.setup_event(
        WorldBuilder::new(WorldEventTrigger::GameStarted).with_handler(move |world: &mut World| {
            world.enemy_spawn_probability.value = 0.0;
            world.fuel_spawn_probability.value = 0.0;

            world.map.change_river_mode(RiverMode::ConstWidthAndCenter {
                width: world.max_c() / 2,
                center_c: world.max_c() / 2,
            });

            world.popup_series(
                [
                    ("Warmup".to_string(), Duration::from_secs(5), style),
                    ("Ready !!".to_string(), Duration::from_secs(2), style),
                    ("!!! GO !!!".to_string(), Duration::from_secs(1), style),
                ],
                |world: &mut World| {
                    world.map.restore_river_mode();
                    world.fuel_spawn_probability.restore();
                    world.enemy_spawn_probability.restore();

                    world.add_timer(
                        Timer::new(Duration::from_secs(10), true),
                        |_, world: &mut World| {
                            world.player.score += 10;
                        },
                    );
                },
            );
        }),
    );
}
