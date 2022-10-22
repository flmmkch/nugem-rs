pub mod game;
use game::Game;

fn main() {
    env_logger::init();
    let sdl_context = sdl2::init().expect("Unable to initialize SDL2 context.");
    let game = Game::new(&sdl_context);
    game.run();
}
