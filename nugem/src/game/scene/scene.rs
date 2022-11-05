use crate::game::{ events, graphics, input };
use crate::game::Config;

pub trait Scene {
    fn load(&mut self, graphics_state: &graphics::State, config: &Config) -> Result<(), Box<dyn std::error::Error>>;
    fn update(&mut self, graphics_state: &graphics::State, config: &Config, event_loop_sender: events::EventLoopSender) -> bool;
    fn display(&mut self, graphics_state: &graphics::State);
    fn input_event(&mut self, _input_event: input::event::Event) -> Option<events::Event> {
        None
    }
}
