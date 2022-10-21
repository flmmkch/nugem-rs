use super::Config;
use sdl2::Sdl;
use ::game::scene;
use ::game::events;
use ::game::graphics::window::Window;
use ::game::scene::Scene;
use ::game::input;
use game_time::{GameClock, FrameCounter, FrameCount};
use game_time::framerate::RunningAverageSampler;
use game_time::step;

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
        let mut clock = GameClock::new();
        let mut counter = FrameCounter::new(self.config.ticks_per_second() as f64, RunningAverageSampler::with_max_samples(self.config.ticks_per_second() * 15 / 10));
        'main: loop {
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
            let sim_time = clock.tick(&step::FixedStep::new(&counter));
            counter.tick(&sim_time);
        }
    }
}
