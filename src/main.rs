extern crate sdl2;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_window_sdl;
extern crate gfx_device_gl;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate game_time;
#[macro_use]
extern crate nom;
extern crate byteorder;

pub mod game;
use game::Game;

fn main() {
    env_logger::init().unwrap();
    let sdl_context = sdl2::init().expect("Unable to initialize SDL2 context.");
    let game = Game::new(&sdl_context);
    game.run();
}
