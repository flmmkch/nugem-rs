use super::Scene;
use ::game::mugen::character::Character;
use ::game::mugen::character::air;
use ::game::mugen::format;
use ::game::graphics::window::Window;
use ::game::graphics::sprite_displayer;
use ::game::mugen::format::sff::SffData;
use ::game::Config;
use ::game::events;
use ::game::input;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ImageKey {
    group: u16,
    image: u16,
}

struct Player {
    pub character: Character,
    pub image_keys: HashMap<ImageKey, usize>,
    pub animations: Vec<air::Animation>,
    pub current_animation: usize,
    pub current_step: usize,
    pub tick_timer: u16,
    pub big_face: usize,
    pub small_face: usize,
    pub sprite_id: usize,
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
        Player {
            character,
            image_keys: HashMap::new(),
            animations: Vec::new(),
            current_animation: 0,
            current_step: 0,
            tick_timer: 0,
            big_face: 0,
            small_face: 0,
            sprite_id: 0,
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
    pub fn load(&mut self, window: &mut Window) {
        let mut sprite_atlas_builder = sprite_displayer::TextureAtlasBuilder::new();
        for player in self.players.iter_mut() {
            let char_data = player.character.read_data().unwrap();
            player.small_face = sprite_atlas_builder.add_surface(char_data.render_sprite(9000, 0, 0).unwrap());
            player.big_face = sprite_atlas_builder.add_surface(char_data.render_sprite(9000, 1, 0).unwrap());
            // btreemap for it to be in order
            let animations : BTreeMap<u32, air::Animation> = player.character.read_animations().into_iter().collect();
            for (_, animation) in animations {
                for ref frame in animation.steps()[0].frames() {
                    let key = ImageKey {
                        group: frame.group,
                        image: frame.image,
                    };
                    if player.image_keys.get(&key) == None {
                        if let Some(rendered_sprite) = char_data.render_sprite(key.group, key.image, 0) {
                            let image_id = sprite_atlas_builder.add_surface(rendered_sprite);
                            player.image_keys.insert(key, image_id);
                        }
                    }
                }
                player.animations.push(animation);
                break;
            }
        }
        let texture_atlas = sprite_atlas_builder.build(window.factory()).unwrap();
        let sprite_context = sprite_displayer::DrawingContext::new(window.factory());
        let mut sprite_canvas = sprite_displayer::SpritesDrawer::new();
        for i in 0..self.players.len() {
            let player = &mut self.players[i];
            let sprite_id = {
                let current_animation = &player.animations[player.current_animation];
                let current_frame = &current_animation.steps()[0].frames()[player.current_step];
                let key = ImageKey {
                    group: current_frame.group,
                    image: current_frame.image,
                };
                *player.image_keys.get(&key).unwrap()
            };
            let (w, h) = texture_atlas.dimensions(sprite_id);
            sprite_canvas.add_sprite(player.big_face, (50 + i * 300) as u32, 450, 175, 175);
            player.sprite_id = sprite_canvas.add_sprite(sprite_id, (50 + i * 300) as u32, 200, w * 2, h * 2);
        }
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
                        // quit on pressing back
                        if input_event.partial_state.back == Some(input::ButtonState::Down) {
                            return false;
                        }
                    },
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
            for i in 0..self.players.len() {
                let player = &mut self.players[i];
                let current_animation = &player.animations[player.current_animation];
                if player.tick_timer == 0 {
                    player.current_step += 1;
                    if player.current_step == current_animation.steps()[0].frames().len() {
                        player.current_step = 0;
                    }
                    let new_frame = &current_animation.steps()[0].frames()[player.current_step];
                    player.tick_timer = new_frame.ticks.unwrap_or(1);
                    // change frame
                    let sprite_id = {
                        let key = ImageKey {
                            group: new_frame.group,
                            image: new_frame.image,
                        };
                        *player.image_keys.get(&key).unwrap()
                    };
                    let (w, h) = loaded_data.texture_atlas.dimensions(sprite_id);
                    loaded_data.sprite_canvas.update_canvas(player.sprite_id, sprite_id, (50 + i * 300) as u32, 200, w * 2, h * 2);
                }
                else {
                    player.tick_timer -= 1;
                }
            }
            loaded_data.sprite_canvas.draw(&loaded_data.sprite_context, &loaded_data.texture_atlas, factory, encoder, render_target_view);
        }
    }
}
