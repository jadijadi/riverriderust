use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};

use std::time::Duration;

use crate::{
    entities::{Bullet, PlayerStatus},
    world::World,
};

pub fn handle_pressed_keys(world: &mut World) -> std::io::Result<()> {
    if poll(Duration::from_millis(10))? {
        let key = read()?;

        while poll(Duration::from_millis(0))? {
            let _ = read();
        }

        match key {
            Event::Key(event) => {
                // I'm reading from keyboard into event
                match event.code {
                    // Movements
                    KeyCode::Char('w') | KeyCode::Up
                        if world.player.status == PlayerStatus::Alive
                            && world.player.location.line > 1 =>
                    {
                        world.player.go_up();
                    }
                    KeyCode::Char('s') | KeyCode::Down
                        if world.player.status == PlayerStatus::Alive
                            && world.player.location.line < world.max_l() - 1 =>
                    {
                        world.player.go_down();
                    }
                    KeyCode::Char('a') | KeyCode::Left
                        if world.player.status == PlayerStatus::Alive
                            && world.player.location.column > 1 =>
                    {
                        world.player.go_left();
                    }
                    KeyCode::Char('d') | KeyCode::Right
                        if world.player.status == PlayerStatus::Alive
                            && world.player.location.column < world.max_c() - 1 =>
                    {
                        world.player.go_right();
                    }

                    // Other events
                    KeyCode::Char('q') => world.player.status = PlayerStatus::Quit,
                    KeyCode::Char('p') if event.kind == KeyEventKind::Press => {
                        use crate::world::WorldStatus::*;
                        world.status = match world.status {
                            Fluent => Solid,
                            Solid => Fluent,
                        };
                    }
                    KeyCode::Char(' ') if event.kind == KeyEventKind::Press => {
                        if world.player.status == PlayerStatus::Alive && world.bullets.is_empty() {
                            let new_bullet = Bullet::new(&world.player.location, world.max_l() / 4);
                            world.bullets.push(new_bullet);
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
