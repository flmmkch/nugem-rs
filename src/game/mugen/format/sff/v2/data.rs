use std::collections::BTreeMap;
use ::game::graphics::surface::{BitmapSurface, BitmapPixel};
use super::super::SffData;

#[derive(Debug)]
pub enum ImageFormat {
    Raw,
    Invalid,
    RLE8,
    RLE5,
    LZ5
}

#[derive(Debug)]
pub struct SpriteInfo {
    /// dimension of the sprite: width, height
    pub size: (u16, u16),
    /// axis translation of the sprite: x, y
    pub axis: (u16, u16),
    /// linked index to the actual sprite image
    pub linked_index: u16,
    /// image format
    pub format: ImageFormat,
    /// color depth
    pub color_depth: u8,
    /// offset of the sprite data into the ldata or tdata
    pub data_offset: u32,
    /// length of the sprite data
    pub data_length: u32,
    /// index of the palette used for the sprite
    pub palette_index: u16,
    /// True if the sprite data is in the tdata block, false if it is in the ldata block
    pub uses_tdata: bool,
}

#[derive(Debug)]
pub struct GroupInfo(pub BTreeMap<u16, usize>);

#[derive(Debug)]
pub struct PaletteInfo {
    /// Number of colors
    pub colors: u16,
    pub linked_index: u16,
    pub ldata_offset: u32,
    pub ldata_length: u32,
}

#[derive(Debug)]
pub struct Data {
    sprites: Vec<SpriteInfo>,
    groups: BTreeMap<u16, GroupInfo>,
    palettes: Vec<PaletteInfo>,
    ldata: Vec<u8>,
    tdata: Vec<u8>,
}

impl Data {
    pub fn new(sprites: Vec<SpriteInfo>, groups: BTreeMap<u16, GroupInfo>, palettes: Vec<PaletteInfo>, ldata: Vec<u8>, tdata: Vec<u8>) -> Data {
        Data {
            sprites,
            groups,
            palettes,
            ldata,
            tdata,
        }
    }
    pub fn linked_sprite(&self, sprite_number: usize) -> &SpriteInfo {
        let sprite = &self.sprites[sprite_number];
        if (sprite.linked_index > 0) && ((sprite.linked_index as usize) < self.sprites.len()) {
            self.linked_sprite(sprite.linked_index as usize)
        }
        else {
            &sprite
        }
    }
    pub fn linked_palette(&self, palette_number: usize) -> &PaletteInfo {
        let palette = &self.palettes[palette_number];
        if (palette.linked_index > 0) && ((palette.linked_index as usize) < self.palettes.len()) {
            self.linked_palette(palette.linked_index as usize)
        }
        else {
            &palette
        }
    }
    pub fn sprite_index_surface(&self, sprite_number: usize, palette_number: usize) -> Option<BitmapSurface> {
        let displayed_sprite = self.linked_sprite(sprite_number);
        let palette_used = {
            // if the sprite indicates a palette number other than 0, it's trying to force that one
            if displayed_sprite.palette_index > 0 {
                self.linked_palette(displayed_sprite.palette_index as usize)
            }
            else {
                self.linked_palette(palette_number)
            }
        };
        self.sprite_surface(displayed_sprite, palette_used)
    }
    pub fn sprite_surface(&self, sprite_info: &SpriteInfo, palette_info: &PaletteInfo) -> Option<BitmapSurface> {
        let mut surface = BitmapSurface::new(sprite_info.size.0 as u32, sprite_info.size.1 as u32);
        let output_colored_pixel = |color: u8, pixel_index: usize, surface: &mut BitmapSurface| {
            let bitmap_pixel = {
                if color > 0 {
                    // 4 bytes per color: 3 bytes for the 8-bit RGB values and an unused one last
                    let color_index = (palette_info.ldata_offset as usize) + (color as usize) * 4;
                    let color_array = &self.ldata[color_index..color_index + 4];
                    let (r, g, b) = (color_array[0], color_array[1], color_array[2]);
                    BitmapPixel::new(r, g, b, u8::max_value())
                }
                else {
                    BitmapPixel::new(0, 0, 0, 0)
                }
            };
            surface.pixels_mut()[pixel_index] = bitmap_pixel;
        };
        let sprite_data = {
            let data_block = {
                if sprite_info.uses_tdata {
                    &self.tdata
                }
                else {
                    &self.ldata
                }
            };
            // the first 4 bytes are the size of uncompressed data, so skip them
            &data_block[((sprite_info.data_offset as usize) + 4)..((sprite_info.data_offset + sprite_info.data_length) as usize)]
        };
        match sprite_info.format {
            ImageFormat::Raw => {
                // raw, uncompressed image
                for index in 0..sprite_data.len() {
                    output_colored_pixel(sprite_data[index], index, &mut surface);
                }
            },
            ImageFormat::RLE8 => {
                // Run-Length Encoding with an 8-bit-per-pixel pixmap
                let mut pixel_index = 0;
                let mut data_index = 0;
                while data_index < sprite_data.len() {
                    let first_byte = sprite_data[data_index];
                    if (first_byte & 0xC0) == 0x40 {
                        // RLE control packet
                        let run_length = first_byte & 0x3F;
                        data_index += 1;
                        let color_byte = sprite_data[data_index];
                        // output the color for the run length
                        for _ in 0..run_length {
                            output_colored_pixel(color_byte, pixel_index, &mut surface);
                            pixel_index += 1;
                        }
                    }
                    else {
                        // output the raw pixel
                        output_colored_pixel(first_byte, pixel_index, &mut surface);
                        pixel_index += 1;
                    }
                    data_index += 1;
                }
            },
            ImageFormat::RLE5 => {
                // Run-Length Encoding with a 5-bit-per-pixel pixmap
                let mut pixel_index = 0;
                let mut data_index = 0;
                while data_index < sprite_data.len() {
                    let data_length = {
                        let run_length = sprite_data[data_index];
                        data_index += 1;
                        let (color, data_length) = {
                            let data_length_byte = sprite_data[data_index];
                            let color = {
                                if (data_length_byte & 0x80) > 0 {
                                    // testing the color bit
                                    data_index += 1;
                                    sprite_data[data_index]
                                }
                                else {
                                    // if no color bit: transparency
                                    0
                                }
                            };
                            // the actual data length is the rest of the byte (without the color bit)
                            (color, data_length_byte & 0x7F)
                        };
                        // output the bytes
                        for _ in 0..run_length {
                            output_colored_pixel(color, pixel_index, &mut surface);
                            pixel_index += 1;
                        }
                        data_length
                    };
                    // reprocess the output according to the data length
                    for _ in 0..data_length {
                        data_index += 1;
                        let (color, run_length) = {
                            let data_byte = sprite_data[data_index];
                            (data_byte & 0x1F, data_byte >> 5)
                        };
                        for _ in 0..run_length {
                            output_colored_pixel(color, pixel_index, &mut surface);
                            pixel_index += 1;
                        }
                    }
                    data_index += 1;
                }
            },
            ImageFormat::LZ5 => {
                // LZ5 compression
                // see the documentation on LZ5 compression: https://web.archive.org/web/20141230125932/http://elecbyte.com/wiki/index.php/LZ5
                let mut short_lz_packets : u32 = 1;
                let mut recycled_bits : u8 = 0;
                let mut pixel_index = 0;
                let mut data_index = 0;
                'data_loop: while data_index < sprite_data.len() {
                    let control_packet = sprite_data[data_index];
                    for packet_index in 0..8 {
                        data_index += 1;
                        // break if we are above the data size
                        if data_index >= sprite_data.len() {
                            break 'data_loop;
                        }
                        let lz_packet = {
                            // flag for the type of this packet: n-th bit of the control packet byte
                            // if the flag is 0, then it is a Run-Length Encoding packet
                            // if the flag is 1, then it is a LZ packet
                            let flag = control_packet & (1 << packet_index);
                            flag > 0
                        };
                        if lz_packet {
                            // LZ packet, short or long
                            let lz_packet_byte = sprite_data[data_index];
                            let mut copy_length = (lz_packet_byte as usize) & 0x3F; // bits 0-5
                            let mut offset;
                            if copy_length > 0 {
                                // short LZ packet if initial copy length is not null
                                copy_length += 1; // see the docs
                                // Recycled bits work:
                                // bits 6-7: recycled bits of short LZ packet 4k + 1
                                // bits 4-5: recycled bits of short LZ packet 4k + 2
                                // bits 2-3: recycled bits of short LZ packet 4k + 3
                                // bits 0-1: recycled bits of short LZ packet 4k + 4
                                let new_recycled_bits = (lz_packet_byte & 0xC0) >> 6; // top 2 bits
                                recycled_bits = new_recycled_bits | (recycled_bits << 2);
                                if short_lz_packets % 4 == 0 {
                                    // use the recycled bits then
                                    offset = (recycled_bits as usize) + 1;
                                    recycled_bits = 0;
                                }
                                else {
                                    data_index += 1;
                                    offset = (sprite_data[data_index] as usize) + 1;
                                }
                                short_lz_packets += 1;
                            }
                            else {
                                // long LZ packet
                                // since the 0x3F-masked bits are null, there are only the 0xC0-masked bits left
                                // take the highest 2 bits of the 10-bit offset
                                offset = 1 + ((sprite_data[data_index] as usize) << 2);
                                data_index += 1;
                                // take the lowest 8 bits of the 10-bit offset, plus one
                                offset = offset | (1 + (sprite_data[data_index] as usize));
                                data_index += 1;
                                // value range of the copy length for a long LZ packet: 8 to 263
                                copy_length = (sprite_data[data_index] as usize) + 3;
                            }
                            // to be safer: check if the pixel index is over the surface limit
                            if pixel_index + copy_length > surface.pixels().len() {
                                copy_length = surface.pixels().len() - pixel_index;
                            }
                            // actually write the surface
                            // Credits to the Nomen developper:
                            // if the length is greater than the offset, then the copy pointer must go back to the beginning of the source when it reaches the full length
                            // so that is why the offset has a factor here
                            if offset > 0 {
                                for current_pixel_index in 0..copy_length {
                                    use std::cmp;
                                    let offset_from_beginning = cmp::min(
                                        offset * (1 + current_pixel_index / offset),
                                        pixel_index
                                    );
                                    surface.pixels_mut()[pixel_index] = surface.pixels()[pixel_index - offset_from_beginning];
                                    pixel_index += 1;
                                }
                            }
                            else {
                                // the offset should never be null though
                                // tentatively: just copy the same pixel over and over
                                for current_pixel_index in 0..copy_length {
                                    surface.pixels_mut()[pixel_index] = surface.pixels()[pixel_index - current_pixel_index];
                                    pixel_index += 1;
                                }
                            }
                        }
                        else {
                            // RLE packet, short or long
                            let (color, run_length) = {
                                let rle_byte = sprite_data[data_index];
                                let color = rle_byte & 0x1F;
                                let mut run_length = (rle_byte as u32) & 0xE0;
                                if run_length > 0 {
                                    // short RLE byte
                                    run_length = run_length >> 5;
                                }
                                else {
                                    // long RLE byte
                                    data_index += 1;
                                    run_length = 8 + (sprite_data[data_index] as u32);
                                }
                                if pixel_index + (run_length as usize) > surface.pixels().len() {
                                    run_length = (surface.pixels().len() - pixel_index) as u32;
                                }
                                (color, run_length)
                            };
                            for _ in 0..run_length {
                                output_colored_pixel(color, pixel_index, &mut surface);
                                pixel_index += 1;
                            }
                        }
                    }
                    data_index += 1;
                }
            },
            ImageFormat::Invalid => return None,
        }
        Some(surface)
    }
}

impl SffData for Data {
    fn palette_count(&self) -> usize {
        self.palettes.len()
    }
    fn render_sprite(&self, group_index: u16, image_index: u16, palette_index: usize) -> Option<BitmapSurface> {
        if let Some(group) = self.groups.get(&group_index) {
            if let Some(sprite_index) = group.0.get(&image_index) {
                self.sprite_index_surface(*sprite_index, palette_index)
            }
            else {
                None
            }
        }
        else {
            None
        }
    }
}
