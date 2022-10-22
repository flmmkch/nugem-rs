use std::collections::BTreeMap;
use std::io::Cursor;
use std::borrow::Cow;
use crate::bitmap::BitmapRenderer;
use crate::v1::RenderingError;

use super::pcx;
use super::{Color, Palette, PALETTE_COLOR_COUNT};
use crate::SffData;

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
    pub fn new<Palettes: IntoIterator<Item = Palette>>(sprites: Vec<Sprite>, groups: BTreeMap<u16, Group>, palettes: Palettes, shared_palette: bool) -> Data {
        let palettes = palettes.into_iter().collect();
        Data {
            sprites,
            groups,
            palettes,
            shared_palette,
        }
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
    fn render_sprite_linked_index_surface<R: BitmapRenderer>(&self, renderer_params: R::Initializer, linked_index: u16, palette: &Palette) -> Result<R, RenderingError<R::Error>> {
        // if the specified sprite uses a linked index
        let actual_index = linked_index as usize;
        if actual_index < self.sprites.len() {
            self.render_sprite_real_index_surface(renderer_params, actual_index, palette)
        }
        else {
            Err(RenderingError::InvalidLinkedSpriteNumber { invalid_index: linked_index, sprite_count: self.sprites.len() })
        }
    }
    fn render_sprite_real_index_surface<R: BitmapRenderer>(&self, renderer_params: R::Initializer, real_index: usize, palette: &Palette) -> Result<R, RenderingError<R::Error>> {
        let specified_sprite = &self.sprites[real_index];
        if specified_sprite.data.len() > 0 {
            let palette = self.sprite_palette(real_index, palette);
            let cursor = Cursor::new(&specified_sprite.data[..]);
            pcx::read_pcx_surface(cursor, renderer_params, &palette)
        }
        else {
            self.render_sprite_linked_index_surface(renderer_params, specified_sprite.linked_index, palette)
        }
    }
    pub fn render_sprite_surface<R: BitmapRenderer>(&self, renderer_params: R::Initializer, group_index: u16, image_index: u16, palette: &Palette) -> Result<R, RenderingError<R::Error>> {
        // first get the specified sprite
        let sprite_group = self.groups.get(&group_index).ok_or_else(|| RenderingError::InvalidSpriteGroupNumber { invalid_index: group_index, sprite_group_count: self.groups.len() })?;
        let specified_real_index = *sprite_group.0.get(&image_index).ok_or_else(|| RenderingError::InvalidImageNumber { invalid_index: group_index, image_count: sprite_group.0.len() })?;
        let specified_sprite = &self.sprites[specified_real_index];
        if specified_sprite.data.len() > 0  {
            self.render_sprite_real_index_surface(renderer_params, specified_real_index, palette)
        }
        else {
            self.render_sprite_linked_index_surface(renderer_params, specified_sprite.linked_index, palette)
        }
    }
}

impl SffData for Data {
    fn version_bytes() -> &'static [u8; 4] {
        &[0, 1, 0, 1]
    }
    fn palette_count(&self) -> usize {
        self.palettes.len()
    }
    fn render_sprite<R: BitmapRenderer>(&self, renderer_params: R::Initializer, group_index: u16, image_index: u16, palette_index: usize) -> Result<R, crate::RenderingError<R::Error>> {
        let palette = &self.palettes.get(palette_index).ok_or(RenderingError::PaletteNotFound(palette_index))?;
        self.render_sprite_surface(renderer_params, group_index, image_index, palette).map_err(Into::into)
    }
}
