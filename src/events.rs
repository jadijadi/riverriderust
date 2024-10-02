use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};

use std::time::Duration;

use crate::{entities::PlayerStatus, world::World, WorldStatus::*};

pub fn handle_pressed_keys(world: &mut World) {
    if poll(Duration::from_millis(10)).unwrap() {
        let key = read().unwrap();

        while poll(Duration::from_millis(0)).unwrap() {
            let _ = read();
        }

        match key {
            Event::Key(event) => {
                // Let's match the keyboard events and do some actions

                match event.code {
                    KeyCode::Char('p') => {
                        if event.kind == KeyEventKind::Press {
                            world.status = match world.status {
                                Fluent => Paused,
                                Paused => Fluent,
                            };
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Esc => world.player.status = PlayerStatus::Quit,
                    _ => {}
                }

                if world.player.status == PlayerStatus::Alive {
                    match event.code {
                        KeyCode::Char('w') | KeyCode::Up => world.player.move_up(),
                        KeyCode::Char('s') | KeyCode::Down => world.player.move_down(),
                        KeyCode::Char('a') | KeyCode::Left => world.player.move_left(),
                        KeyCode::Char('d') | KeyCode::Right => world.player.move_right(),
                        KeyCode::Char(' ') => world.create_bullet(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
