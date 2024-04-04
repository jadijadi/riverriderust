use game::Game;
use std::io::stdout;
use utilities::stout_ext::StdoutExt;

use crossterm::{
    cursor::{Hide, Show},
    terminal::{disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand,
};
use world::World;

mod canvas;
mod entities;
mod events;
mod game;
mod utilities;
mod world;

fn main() -> std::io::Result<()> {
    // init the screen
    let mut sc = stdout();
    let (maxc, maxl) = size().unwrap();
    sc.execute(Hide)?;
    enable_raw_mode()?;

    // init the world
    let slowness = 80;

    let mut game = Game::new(maxc, maxl);

    // Events that are running forever once in each loop
    game.setup_event_handlers();

    // show welcoming banner
    game.welcome_screen(&mut sc)?;

    // Main game loop
    // - Events
    // - Physics
    // - Drawing
    // TODO:
    game.game_loop(&mut sc, slowness)?;

    // game is finished
    game.clear_screen(&mut sc)?;
    game.goodbye_screen(&mut sc)?;

    sc.clear_all()?.execute(Show)?;
    disable_raw_mode()?;
    Ok(())
}
