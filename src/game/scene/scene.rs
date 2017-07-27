use ::game::graphics::window::Window;
use ::game::Config;

pub trait Scene {
    fn update(&mut self, &mut Window, &Config) -> bool;
    fn display(&mut self, &mut Window);
}
