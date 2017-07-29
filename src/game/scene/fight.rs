use super::Scene;
use ::game::mugen::character::Character;
use ::game::mugen::format;
use ::game::graphics::window::Window;
use ::game::graphics::sprite_displayer;
use ::game::mugen::format::sff::SffData;
use ::game::Config;
use ::game::events;

struct Player {
    pub character: Character,
    pub char_data: format::sff::Data,
}

struct FightData {
    pub texture_atlas: sprite_displayer::SpriteTextureAtlas,
    pub sprite_context: sprite_displayer::DrawingContext,
    pub sprite_canvas: sprite_displayer::SpritesDrawer,
}

pub struct Fight {
    loaded_data: Option<FightData>,
    players: [Player; 2],
}

impl Player {
    pub fn new(character: Character) -> Player {
        let char_data = character.read_data().unwrap();
        Player {
            character,
            char_data,
        }
    }
}

impl Fight {
    pub fn new(character_p1: Character, character_p2: Character) -> Fight {
        let players = [Player::new(character_p1), Player::new(character_p2)];
        Fight {
            loaded_data: None,
            players,
        }
    }
    
    pub fn loaded(&self) -> bool {
        self.loaded_data.is_some()
    }

    fn loaded_data(&self) -> Option<&FightData> {
        self.loaded_data.as_ref()
    }

    pub fn load(&mut self, window: &mut Window) {
        let mut sprite_atlas_builder = sprite_displayer::TextureAtlasBuilder::new();
        // small faces
        for player in self.players.iter() {
            if let Some(s) = player.char_data.render_sprite(9000, 0, 0) {
                sprite_atlas_builder.add_surface(s);
            }
        }
        // big faces
        for player in self.players.iter() {
            if let Some(s) = player.char_data.render_sprite(9000, 1, 0) {
                sprite_atlas_builder.add_surface(s);
            }
        }
        let texture_atlas = sprite_atlas_builder.build(window.factory()).unwrap();
        let sprite_context = sprite_displayer::DrawingContext::new(window.factory());
        let mut sprite_canvas = sprite_displayer::SpritesDrawer::new();
        sprite_canvas.add_sprite(0, 0, 60, 75, 75);
        sprite_canvas.add_sprite(1, 0, 190, 75, 75);
        sprite_canvas.add_sprite(2, 240, 0, 350, 350);
        sprite_canvas.add_sprite(3, 240, 400, 350, 350);
        self.loaded_data = Some(FightData {
            texture_atlas,
            sprite_context,
            sprite_canvas,
        });
    }
}

impl Scene for Fight {
    fn update(&mut self, window: &mut Window, event_queue: &mut events::EventQueue, _: &Config) -> bool {
        match event_queue.pop() {
            Some(event) => {
                match event {
                    events::Event::Quit => return false,
                    events::Event::Input(input_event) => {
                        info!("{:?}", input_event);
                    },
                    _ => (),
                }
            },
            None => (),
        }
        if !self.loaded() {
            self.load(window);
        }
        true
    }

    fn display(&mut self, window: &mut Window) {
        if let Some(loaded_data) = self.loaded_data.as_mut() {
            let (mut factory, encoder, render_target_view) = window.gfx_data();
            loaded_data.sprite_canvas.draw(&loaded_data.sprite_context, &loaded_data.texture_atlas, factory, encoder, render_target_view);
        }
    }
}
