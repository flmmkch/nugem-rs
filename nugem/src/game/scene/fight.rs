use super::Scene;
use crate::game::mugen::character::{Character, command};
use crate::game::mugen::character::air;
use crate::game::graphics::{self, surface::BitmapSurfaceRenderer};
use crate::game::Config;
use crate::game::events;
use crate::game::input::{self, DirectionState, DirectionalMotion, Directional};
use std::collections::{BTreeMap, HashMap};
use log::error;

const SCREEN_DIMENSIONS: (u32, u32) = (800, 600);

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
    pub texture_atlas: graphics::sprites::SpriteTextureAtlas,
    pub sprite_stack: graphics::sprites::SpriteStack,
}

struct CharaData {
    // TODO
    pub _character: Character,
    pub sff_data: nugem_sff::SpriteFile,
    pub animations: Vec<air::Animation>,
    // TODO use commands
    pub _commands: command::CommandConfiguration,
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
    pub fn new(_: &Config) -> Fight {
        let players = [Player::new(0), Player::new(1)];
        Fight {
            characters: Vec::new(),
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
    pub fn load(&mut self, state: &graphics::State) {
        let mut sprite_atlas_builder = graphics::sprites::SpriteTextureAtlasBuilder::new();
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
        let texture_atlas = sprite_atlas_builder.build(state).unwrap();
        let mut sprite_stack_drawer = graphics::sprites::SpriteStack::new(state.device(), state.surface_configuration().format, SCREEN_DIMENSIONS);
        sprite_stack_drawer.set_texture_atlas(&texture_atlas, state.device());
        for (player_number, player) in self.players.iter_mut().enumerate() {
            if let Some(animator) = player.animator.as_mut() {
                if let Some((group, image)) = animator.current_display_info() {
                    let key = ImageKey { group, image };
                    if let Some(&sprite_id) = player.image_keys.get(&key) {
                        let (w, h) = texture_atlas.dimensions(sprite_id).unwrap();
                        if let Some(big_face) = player.big_face.clone() {
                            sprite_stack_drawer.push_sprite(big_face, (50 + player_number * 300) as u32, 400, 175, 175);
                        }
                        player.sprite_id = sprite_stack_drawer.push_sprite(sprite_id, (50 + player_number * 300) as u32, 200, w * 2, h * 2);
                    }
                }
            }
        }
        self.loaded_data = Some(FightData {
            texture_atlas,
            sprite_stack: sprite_stack_drawer,
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
    fn load(&mut self, graphics_state: &graphics::State, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let characters_iterator = config.data_paths()
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
                    _character: character,
                    sff_data,
                    animations,
                    _commands: commands,
                })
            })
            ;
        self.characters.clear();
        self.characters.extend(characters_iterator);
        if self.characters.is_empty() {
            Err("No characters found in data directories.")?
        }
        self.load(graphics_state);
        Ok(())
    }

    fn input_event(&mut self, input_event: input::event::Event) -> Option<events::Event> {
        // quit on pressing back
        if input_event.partial_state.back == Some(input::ButtonState::Down) {
            return Some(events::Event::Quit);
        }
        if let Some(motion) = input_event.partial_state.directional {
            match motion {
                DirectionalMotion::Vertical(DirectionState::Plus) | DirectionalMotion::FullDirection(Directional::Up) => self.change_character(1),
                DirectionalMotion::Vertical(DirectionState::Minus) | DirectionalMotion::FullDirection(Directional::Down) => self.change_character(-1),
                DirectionalMotion::Horizontal(DirectionState::Plus) | DirectionalMotion::FullDirection(Directional::Forward) => self.change_animation(1),
                DirectionalMotion::Horizontal(DirectionState::Minus) | DirectionalMotion::FullDirection(Directional::Backward) => self.change_animation(-1),
                _ => (),
            }
        }
        None
    }

    fn update(&mut self, graphics_state: &graphics::State, _: &Config, _: events::EventLoopSender) -> bool {
        if !self.loaded() {
            self.load(graphics_state);
        }
        true
    }

    fn display(&mut self, graphics_state: &graphics::State) {
        if let Ok(output) = graphics_state.surface().get_current_texture() {
            let surface_texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            if let Some(loaded_data) = self.loaded_data.as_mut() {
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
                                    let (w, h) = loaded_data.texture_atlas.dimensions(sprite_id).unwrap();
                                    loaded_data.sprite_stack.update_sprite(player.sprite_id, sprite_id, (50 + i * 300) as u32, 200, w * 2, h * 2);
                                }
                            }
                        }
                    }
                }
                loaded_data.sprite_stack.apply_changes(&loaded_data.texture_atlas, graphics_state.device(), graphics_state.queue());
                loaded_data.sprite_stack.render(&surface_texture_view, graphics_state.device(), graphics_state.queue());
            }
            output.present();
        }
    }
}
