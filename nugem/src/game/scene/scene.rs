use ::game::graphics::window;
use ::game::Config;
use ::game::events;

pub trait Scene {
    fn update(&mut self, &mut window::Window, &mut events::EventQueue, &Config) -> bool;
    fn display(&mut self, &mut window::Window);
}
