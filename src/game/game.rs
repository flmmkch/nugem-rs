use super::Config;
use sdl2;
use sdl2::Sdl;
use ::game::graphics::window::Window;
use ::game::mugen::character;
use ::game::graphics::sprite_displayer;

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
        let characters: Vec<character::Character> = self.config.data_paths()
            .iter()
            .map(|data_path| { character::find_characters(data_path) })
            .filter_map(|char_dir| { char_dir })
            .fold(Vec::new(), |mut v, character_dir| {
                v.extend(character_dir);
                v
            });
        // Debug: faces for the characters
        let mut sprite_atlas_builder = sprite_displayer::TextureAtlasBuilder::new();
        let mut character_faces = Vec::new();
        for character in characters.iter() {
            use ::game::mugen::format::sff;
            if let sff::Data::V1(ref d) = *character.sff_data() {
                let palette = &d.palettes()[0];
                if let Some(s) = d.sprite_surface(9000, 0, palette) {
                    character_faces.push(sprite_atlas_builder.add_surface(s));
                }
            }
        }
        let selected_character = 3;
        let sprite_atlas = sprite_atlas_builder.build(window.factory()).unwrap();
        let sprite_context = sprite_displayer::DrawingContext::new(window.factory());
        let sprite_canvas = {
            let mut sprite_canvas = sprite_displayer::SpritesDrawer::new(&sprite_atlas);
            for i in 0..character_faces.len() {
                let h = 20 + (i as u32) * 140;
                sprite_canvas.add_sprite(character_faces[i], 60, h, 120, 120);
            }
            sprite_canvas.add_sprite(selected_character, 500, 150, 240, 240);
            sprite_canvas
        };
        'main: loop {
            for event in events.poll_iter() {
                match event {
                    sdl2::event::Event::Quit {..} => break 'main,
                    _ => (),
                }
            }
            window.clear();
            {
                let (factory, encoder, render_target_view) = window.gfx_data();
                sprite_canvas.draw(&sprite_context, factory, encoder, render_target_view);
            }
            window.update();
        }
    }
}