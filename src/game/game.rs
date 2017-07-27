use super::Config;
use sdl2;
use sdl2::Sdl;
use ::game::scene;
use ::game::graphics::window::Window;
use ::game::mugen::character;

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
        let mut events = self.sdl_context.event_pump().unwrap();
        let mut current_scene : Box<scene::Scene> = {
            let characters: Vec<character::Character> = self.config.data_paths()
                .iter()
                .map(|data_path| { character::find_characters(data_path) })
                .filter_map(|char_dir| { char_dir })
                .fold(Vec::new(), |mut v, character_dir| {
                    v.extend(character_dir);
                    v
                });
            let mut iter = characters.into_iter();
            Box::new(scene::fight::Fight::new(iter.next().unwrap(), iter.next().unwrap()))
        };
        'main: loop {
            for event in events.poll_iter() {
                match event {
                    sdl2::event::Event::Quit {..} => break 'main,
                    _ => (),
                }
            }
            current_scene.update(&mut window, &self.config);
            window.clear();
            current_scene.display(&mut window);
            window.update();
        }
    }
}
