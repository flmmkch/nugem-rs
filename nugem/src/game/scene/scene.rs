use crate::game::graphics::window;
use crate::game::Config;
use crate::game::events;

pub trait Scene {
    fn update(&mut self, window: &mut window::Window, events: &mut events::EventQueue, config: &Config) -> bool;
    fn display(&mut self, window: &mut window::Window);
}
