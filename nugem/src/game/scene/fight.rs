use super::Scene;
use crate::game::graphics::surface::BitmapSurfaceRenderer;
use crate::game::mugen::character::{Character, command};
use crate::game::mugen::character::air;
use crate::game::graphics::window::Window;
use crate::game::graphics::sprite_displayer;
use crate::game::Config;
use crate::game::events;
use crate::game::input::{self, DirectionState, DirectionalMotion, Directional};
use std::collections::{BTreeMap, HashMap};
use log::error;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ImageKey {
    group: u16,
    image: u16,
}

struct Player {
    pub character_id: usize,
    pub image_keys: HashMap<ImageKey, usize>,
    pub current_animation: usize,
    pub animator: Option<air::Animator>,
    pub big_face: Option<usize>,
    pub small_face: Option<usize>,
    pub sprite_id: usize,
}

struct FightData {
    pub texture_atlas: sprite_displayer::SpriteTextureAtlas,
    pub sprite_context: sprite_displayer::DrawingContext,
    pub sprite_canvas: sprite_displayer::SpritesDrawer,
}

struct CharaData {
    pub character: Character,
    pub sff_data: nugem_sff::SpriteFile,
    pub animations: Vec<air::Animation>,
    pub commands: Vec<command::Command>,
}

pub struct Fight {
    characters: Vec<CharaData>,
    loaded_data: Option<FightData>,
    players: [Player; 2],
}

impl Player {
    pub fn new(character_id: usize) -> Player {
        Player {
            character_id,
            image_keys: HashMap::new(),
            current_animation: 0,
            animator: None,
            big_face: None,
            small_face: None,
            sprite_id: 0,
        }
    }
}

impl Fight {
    pub fn new(config: &Config) -> Fight {
        let characters: Vec<_> = config.data_paths()
            .iter()
            .flat_map(|data_path| { crate::game::mugen::character::directory_reader::read_directory_characters(data_path) })
            .filter_map(|mut character: Character| -> Option<CharaData> {
                let sff_data = match character.read_data() {
                    Ok(sff_data) => sff_data,
                    Err(err) => {
                        log::error!("Error loading sprite data for {0}: {1}", character.name(), err);
                        None?
                    }
                };
                let commands = match character.read_commands() {
                    Ok(commands) => commands,
                    Err(err) => {
                        log::error!("Error loading command data for {0}: {1}", character.name(), err);
                        None?
                    }
                };
                let animations : Vec<_> = {
                    // first use a btreemap for the animations to be in order
                    character
                        .read_animations()
                        .into_iter()
                        .collect::<BTreeMap<u32, air::Animation>>()
                        .into_iter()
                        .map(|(_, animation)| { animation })
                        .collect()
                };
                Some(CharaData {
                    character,
                    sff_data,
                    animations,
                    commands,
                })
            })
            .collect();
        if characters.is_empty() {
            panic!("No characters found in data directories.");
        }
        let players = [Player::new(0), Player::new(1)];
        Fight {
            characters,
            loaded_data: None,
            players,
        }
    }
    pub fn loaded(&self) -> bool {
        self.loaded_data.is_some()
    }
    pub fn unload(&mut self) {
        self.loaded_data = None;
    }
    pub fn load(&mut self, window: &mut Window) {
        let mut sprite_atlas_builder = sprite_displayer::TextureAtlasBuilder::new();
        for player in self.players.iter_mut() {
            let chara_data = self.characters.get(player.character_id).expect(&format!("Failed to load character {0}", player.character_id));
            let palette_index = 0;
            let render_sff_sprite = |group_index, image_index, palette_index| -> Result<_, nugem_sff::RenderingError<<BitmapSurfaceRenderer as nugem_sff::bitmap::BitmapRenderer>::Error>> {
                let renderer: BitmapSurfaceRenderer = chara_data.sff_data.render_sprite((), group_index, image_index, palette_index)?;
                Ok(renderer.take())
            };
            player.animator = Some(air::Animator::new(chara_data.animations[player.current_animation].clone()));
            player.small_face = render_sff_sprite(9000, 0, palette_index).map(|s| sprite_atlas_builder.add_surface(s)).ok();
            player.big_face = render_sff_sprite(9000, 1, palette_index).map(|s| sprite_atlas_builder.add_surface(s)).ok();
            {
                player.image_keys.clear();
                let animator = player.animator.as_ref().unwrap();
                for step in animator.animation().steps() {
                    for frame in step.frames() {
                        let key = ImageKey {
                            group: frame.group,
                            image: frame.image,
                        };
                        if player.image_keys.get(&key) == None {
                            match chara_data.sff_data.render_sprite::<BitmapSurfaceRenderer>((), key.group, key.image, palette_index) {
                                Ok(renderer) => {
                                    let image_id = sprite_atlas_builder.add_surface(renderer.take());
                                    player.image_keys.insert(key, image_id);
                                },
                                Err(err) => {
                                    error!("Unable to render sprite from group {0}, image {1}, palette {2}: {err}", key.group, key.image, palette_index);
                                },
                            }
                        }
                    }
                }
            }
        }
        let texture_atlas = sprite_atlas_builder.build(window.factory()).unwrap();
        let sprite_context = sprite_displayer::DrawingContext::new(window.factory());
        let mut sprite_canvas = sprite_displayer::SpritesDrawer::new();
        for i in 0..self.players.len() {
            let player = &mut self.players[i];
            if let Some(animator) = player.animator.as_mut() {
                if let Some((group, image)) = animator.current_display_info() {
                    let key = ImageKey { group, image };
                    if let Some(&sprite_id) = player.image_keys.get(&key) {
                        let (w, h) = texture_atlas.dimensions(sprite_id);
                        if let Some(big_face) = player.big_face.clone() {
                            sprite_canvas.add_sprite(big_face, (50 + i * 300) as u32, 450, 175, 175);
                        }
                        player.sprite_id = sprite_canvas.add_sprite(sprite_id, (50 + i * 300) as u32, 200, w * 2, h * 2);
                    }
                }
            }
        }
        self.loaded_data = Some(FightData {
            texture_atlas,
            sprite_context,
            sprite_canvas,
        });
    }
    fn wheel_selection(current: usize, max: usize, move_by: isize) -> usize {
        if move_by > 0 {
            (current + move_by.abs() as usize) % max
        }
        else if move_by < 0 {
            (current as isize - move_by.abs() + max as isize) as usize % max
        }
        else {
            current
        }
    }
    fn change_animation(&mut self, by: isize) {
        let player = &mut self.players[0];
        let animation_count = self.characters[player.character_id].animations.len();
        player.current_animation = Self::wheel_selection(player.current_animation, animation_count, by);
        self.unload();
    }
    fn change_character(&mut self, by: isize) {
        let character_count = self.characters.len();
        let player = &mut self.players[0];
        player.current_animation = 0;
        player.character_id = Self::wheel_selection(player.character_id, character_count, by);
        self.unload();
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
                        log::debug!("Got input event {input_event:?}");
                        if let Some(motion) = input_event.partial_state.directional {
                            match motion {
                                DirectionalMotion::Vertical(DirectionState::Plus) | DirectionalMotion::FullDirection(Directional::Up) => self.change_character(1),
                                DirectionalMotion::Vertical(DirectionState::Minus) | DirectionalMotion::FullDirection(Directional::Down) => self.change_character(-1),
                                DirectionalMotion::Horizontal(DirectionState::Plus) | DirectionalMotion::FullDirection(Directional::Forward) => self.change_animation(1),
                                DirectionalMotion::Horizontal(DirectionState::Minus) | DirectionalMotion::FullDirection(Directional::Backward) => self.change_animation(-1),
                                _ => (),
                            }
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
            let (factory, encoder, render_target_view) = window.gfx_data();
            for i in 0..self.players.len() {
                let player = &mut self.players[i];
                if let Some(animator) = player.animator.as_mut() {
                    if animator.tick() {
                        if let Some((group, image)) = animator.current_display_info() {
                            // change frame
                            let key = ImageKey {
                                group,
                                image,
                            };
                            if let Some(&sprite_id) = player.image_keys.get(&key) {
                                let (w, h) = loaded_data.texture_atlas.dimensions(sprite_id);
                                loaded_data.sprite_canvas.update_canvas(player.sprite_id, sprite_id, (50 + i * 300) as u32, 200, w * 2, h * 2);
                            }
                        }
                    }
                }
            }
            loaded_data.sprite_canvas.draw(&loaded_data.sprite_context, &loaded_data.texture_atlas, factory, encoder, render_target_view);
        }
    }
}
