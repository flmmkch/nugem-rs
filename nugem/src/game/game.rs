use super::Config;
use sdl2::Sdl;
use crate::game::scene;
use crate::game::events;
use crate::game::graphics::window::Window;
use crate::game::scene::Scene;
use crate::game::input;
use std::time::{Duration, Instant};

pub struct Game<'a> {
    sdl_context: &'a Sdl,
    config: Config,
}

impl<'a> Game<'a> {
    pub fn new(sdl_context: &'a Sdl) -> Game<'a> {
        let config = Config::new();
        Game {
            sdl_context,
            config,
        }
    }
    pub fn run(&self) {
        let sdl_video = self.sdl_context.video().unwrap();
        let mut window = Window::new(&self.config, &sdl_video);
        let mut input_manager = input::Manager::new(&self.sdl_context);
        let mut event_queue = events::EventQueue::new(self.sdl_context.event_pump().unwrap());
        let mut current_scene = Box::new(scene::fight::Fight::new(&self.config));
        let max_ms = Duration::from_millis(1000_u64.saturating_div(self.config.ticks_per_second() as u64));
        'main: loop {
            let iteration_start_time = Instant::now();
            event_queue.process(&mut input_manager);
            let continue_main = current_scene.update(&mut window, &mut event_queue, &self.config);
            if !continue_main {
                break 'main;
            }
            window.clear();
            current_scene.display(&mut window);
            window.update();
            // one iteration per second
            // sleep for the rest of the tick
            let iteration_duration = iteration_start_time.elapsed();
            let sleep_duration = max_ms.saturating_sub(iteration_duration);
            std::thread::sleep(sleep_duration);
        }
    }
}
