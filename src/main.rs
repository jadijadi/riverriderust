use events::setup_event_handlers;
use log::LevelFilter;
use riverraid::game::Game;
use std::io::stdout;

mod events;

fn main() -> std::io::Result<()> {
    // Setup logger
    simple_logging::log_to_file("riverraid.log", LevelFilter::Info)?;

    // init the screen
    let mut sc = stdout();

    Game::prepare_terminal(&mut sc)?;

    // init the world
    let slowness = 80;
    let mut game = Game::new();

    // Events that are running forever once in each loop
    setup_event_handlers(&mut game);

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

    Game::release_terminal(&mut sc)?;
    Ok(())
}
