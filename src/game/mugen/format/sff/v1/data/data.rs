use std::collections::BTreeMap;
use std::io::Cursor;
use ::game::graphics::surface::BitmapSurface;
use std::borrow::{Borrow, Cow};
use super::pcx;
use super::{Color, Palette, PALETTE_COLOR_COUNT};

#[derive(Debug)]
pub struct Sprite {
    /// Image axis coordinates: X, Y
    pub axis: (u16, u16),
    /// Index of the previous copy of the sprites (only for linked sprites: those that have no data)
    pub linked_index: u16,
    /// True if it uses the same palette as the previous sprite
    pub uses_shared_palette: bool,
    /// PCX data of the sprite, with the palette potentially at the end
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Group(pub BTreeMap<u16, usize>);

#[derive(Debug)]
pub struct Data {
    sprites: Vec<Sprite>,
    groups: BTreeMap<u16, Group>,
    palettes: Vec<Palette>,
    shared_palette: bool,
}

impl Data {
    pub fn new(sprites: Vec<Sprite>, groups: BTreeMap<u16, Group>, palettes: Vec<Palette>, shared_palette: bool) -> Data {
        Data {
            sprites,
            groups,
            palettes,
            shared_palette,
        }
    }
    pub fn palettes(&self) -> &[Palette] {
        &self.palettes[..]
    }
    fn sprite_palette<'a>(&self, sprite_index: usize, general_palette: &'a Palette) -> Cow<'a, Palette> {
        let mut result = Cow::Borrowed(general_palette);
        for i in 0..sprite_index+1 {
            let sprite = &self.sprites[sprite_index - i];
            if self.shared_palette && sprite.uses_shared_palette {
                break;
            }
            else {
                if (sprite.data.len() > 768) && (sprite.data[sprite.data.len() - 768 - 1] == 0x0C)  {
                    // read the palette data from the sprite data
                    let palette_data_index = sprite.data.len() - 768;
                    for j in 0..PALETTE_COLOR_COUNT {
                        let r = sprite.data[palette_data_index + j * 3];
                        let g = sprite.data[palette_data_index + j * 3 + 1];
                        let b = sprite.data[palette_data_index + j * 3 + 2];
                        result.to_mut().colors[j] = Color::Rgb(r, g, b);
                    }
                    break;
                }
            }
        }
        result
    }
    fn sprite_linked_index_surface(&self, linked_index: u16, palette: &Palette) -> Option<BitmapSurface> {
        // if the specified sprite uses a linked index
        let actual_index = linked_index as usize;
        if actual_index < self.sprites.len() {
            self.sprite_real_index_surface(actual_index, palette)
        }
        else {
            None
        }
    }
    fn sprite_real_index_surface(&self, real_index: usize, palette: &Palette) -> Option<BitmapSurface> {
        let specified_sprite = &self.sprites[real_index];
        if specified_sprite.data.len() > 0 {
            let palette = self.sprite_palette(real_index, palette);
            let cursor = Cursor::new(&specified_sprite.data[..]);
            match pcx::read_pcx_surface(cursor, palette.borrow()) {
                Ok(surface) => Some(surface),
                Err(e) => {
                    println!("Error reading PCX surface: {:?}", e);
                    None
                }
            }
        }
        else {
            self.sprite_linked_index_surface(specified_sprite.linked_index, palette)
        }
    }
    pub fn sprite_surface(&self, group_index: u16, image_index: u16, palette: &Palette) -> Option<BitmapSurface> {
        // first get the specified sprite
        let specified_real_index = {
            match self.groups.get(&group_index) {
                None => return None,
                Some(ref group) => {
                    match group.0.get(&image_index) {
                        None => return None,
                        Some(sprite_index) => *sprite_index,
                    }
                },
            }
        };
        let specified_sprite = &self.sprites[specified_real_index];
        if specified_sprite.data.len() > 0  {
            self.sprite_real_index_surface(specified_real_index, palette)
        }
        else {
            self.sprite_linked_index_surface(specified_sprite.linked_index, palette)
        }
    } 
}
