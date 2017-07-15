use super::Config;
use sdl2;
use gfx_window_sdl;
use gfx_core::format::{DepthStencil, Rgba8};

pub struct Game<'a> {
    sdl_context: &'a sdl2::Sdl,
    sdl_video: sdl2::VideoSubsystem,
    config: Config,
}

impl<'a> Game<'a> {
    pub fn new(sdl_context: &'a sdl2::Sdl) -> Game {
        let sdl_video = sdl_context.video().unwrap();
        let config = Config::new();
        Game {
            sdl_context,
            sdl_video,
            config,
        }
    }
    pub fn run(&self) {
        let (window, glcontext, device, factory, color_view, depth_view) = {
            let mut window_builder = self.sdl_video.window("Rugen", self.config.window_size().0, self.config.window_size().1);
            if self.config.fullscreen() {
                window_builder.fullscreen();
            }
            gfx_window_sdl::init::<Rgba8, DepthStencil>(window_builder).expect("gfx_window_sdl::init failed!")
        }; 
        let mut events = self.sdl_context.event_pump().unwrap();
        'main: loop {
            for event in events.poll_iter() {
                match event {
                    sdl2::event::Event::Quit {..} => break 'main,
                    _ => continue,
                }
            }
        }
    }
}