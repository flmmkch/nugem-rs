pub mod game;
use game::{Config, Game};

fn main() {
    env_logger::init();
    let game = pollster::block_on(Game::new(Config::new()));
    pollster::block_on(game.run())
}
