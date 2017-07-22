extern crate sdl2;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_window_sdl;
extern crate gfx_device_gl;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod game;
use game::Game;

fn main() {
    let sdl_context = sdl2::init().expect("Unable to initialize SDL2 context.");
    let game = Game::new(&sdl_context);
    game.run();
}
