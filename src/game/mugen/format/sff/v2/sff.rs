use std::io::{Read, Seek, SeekFrom};
use std::collections::BTreeMap;
use super::Error;
use super::data::*;
use byteorder::{LittleEndian, ReadBytesExt};

struct RawSpriteInfo {
    group: u16,
    item: u16,
    sprite_info: SpriteInfo,
}

struct RawPaletteInfo {
    #[allow(dead_code)]
    group: u16,
    #[allow(dead_code)]
    item: u16,
    palette_info: PaletteInfo,
}

pub fn read_sff<T: Read + Seek>(mut reader: T) -> Result<Data, Error> {
    // 8 reserved bytes
    reader.seek(SeekFrom::Current(8))?;
    // 4 compatibility version bytes
    reader.seek(SeekFrom::Current(4))?;
    // 8 reserved bytes
    reader.seek(SeekFrom::Current(8))?;
    let first_sprite_offset = reader.read_u32::<LittleEndian>()?;
    let sprite_number = reader.read_u32::<LittleEndian>()?;
    let first_palette_offset = reader.read_u32::<LittleEndian>()?;
    let palette_number = reader.read_u32::<LittleEndian>()?;
    // ldata: literal data block
    // data that is copied straight into memory
    let ldata = {
        let ldata_offset = reader.read_u32::<LittleEndian>()?;
        let ldata_length = reader.read_u32::<LittleEndian>()?;
        let current_position = reader.seek(SeekFrom::Current(0))?;
        let mut ldata_bytes = vec![0; ldata_length as usize];
        reader.seek(SeekFrom::Start(ldata_offset as u64))?;
        reader.read_exact(&mut ldata_bytes[..])?;
        reader.seek(SeekFrom::Start(current_position))?;
        ldata_bytes
    };
    // tdata: translated data block
    // data that is supposed to be translated on load
    let tdata = {
        let tdata_offset = reader.read_u32::<LittleEndian>()?;
        let tdata_length = reader.read_u32::<LittleEndian>()?;
        let current_position = reader.seek(SeekFrom::Current(0))?;
        let mut tdata_bytes = vec![0; tdata_length as usize];
        reader.seek(SeekFrom::Start(tdata_offset as u64))?;
        reader.read_exact(&mut tdata_bytes[..])?;
        reader.seek(SeekFrom::Start(current_position))?;
        tdata_bytes
    };
    // reading the sprites data
    let raw_sprites = {
        let mut sprites = Vec::new();
        reader.seek(SeekFrom::Start(first_sprite_offset as u64))?;
        for _ in 0..(sprite_number as usize) {
            // read the sprite
            let group = reader.read_u16::<LittleEndian>()?;
            let item = reader.read_u16::<LittleEndian>()?;
            let width = reader.read_u16::<LittleEndian>()?;
            let height = reader.read_u16::<LittleEndian>()?;
            let axis_x = reader.read_u16::<LittleEndian>()?;
            let axis_y = reader.read_u16::<LittleEndian>()?;
            let linked_index = reader.read_u16::<LittleEndian>()?;
            // format: 0 -> raw, 1 -> invalid, 2 -> RLE8, 3 -> RLE5, 4 -> LZ5
            let format = {
                let format_byte = reader.read_u8()?;
                match format_byte {
                    0 => ImageFormat::Raw,
                    2 => ImageFormat::RLE8,
                    3 => ImageFormat::RLE5,
                    4 => ImageFormat::LZ5,
                    _ => ImageFormat::Invalid,
                }
            };
            let color_depth = reader.read_u8()?;
            let data_offset = reader.read_u32::<LittleEndian>()?;
            let data_length = reader.read_u32::<LittleEndian>()?;
            let palette_index = reader.read_u16::<LittleEndian>()?;
            let flags = reader.read_u16::<LittleEndian>()?;
            let uses_tdata = (flags & 0x01) != 0;
            let sprite_info = SpriteInfo {
                size: (width, height),
                axis: (axis_x, axis_y),
                linked_index,
                format,
                color_depth,
                data_offset,
                data_length,
                palette_index,
                uses_tdata,
            };
            sprites.push(RawSpriteInfo {
                group,
                item,
                sprite_info
            });
        }
        sprites
    };
    // reading the palettes data
    let raw_palettes = {
        let mut palettes = Vec::new();
        reader.seek(SeekFrom::Start(first_palette_offset as u64))?;
        for _ in 0..(palette_number as usize) {
            // read the palette
            let group = reader.read_u16::<LittleEndian>()?;
            let item = reader.read_u16::<LittleEndian>()?;
            let colors = reader.read_u16::<LittleEndian>()?;
            let linked_index = reader.read_u16::<LittleEndian>()?;
            let ldata_offset = reader.read_u32::<LittleEndian>()?;
            let ldata_length = reader.read_u32::<LittleEndian>()?;
            let palette_info = PaletteInfo {
                colors,
                linked_index,
                ldata_offset,
                ldata_length,
            };
            palettes.push(RawPaletteInfo {
                group,
                item,
                palette_info
            });
        }
        palettes
    };
    // assembling all of this into a data object
    let (groups, sprites) = {
        let mut groups_map = BTreeMap::new();
        let mut sprites = Vec::new();
        for raw_sprite in raw_sprites {
            groups_map.entry(raw_sprite.group).or_insert(GroupInfo(BTreeMap::new())).0.insert(raw_sprite.item, sprites.len());
            sprites.push(raw_sprite.sprite_info);
        }
        (groups_map, sprites)
    };
    let palettes = {
        let mut palettes = Vec::new();
        for raw_palette in raw_palettes {
            palettes.push(raw_palette.palette_info);
        }
        palettes
    };
    Ok(Data::new(sprites, groups, palettes, ldata, tdata))
}
