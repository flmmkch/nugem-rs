extern crate sdl2;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_window_sdl;

mod game;
use game::Game;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let game = Game::new(&sdl_context);
    game.run();
}
