use std::io::{Error, Read, Seek, SeekFrom};
use super::{Group, Sprite};
use std::collections::BTreeMap;
use byteorder::{LittleEndian, ReadBytesExt};

struct SffImage {
    pub group_index: u16,
    pub image_index: u16,
    pub sprite: Sprite,
}

pub fn read_sff<T: Read + Seek>(mut reader: T) -> Result<(Vec<Sprite>, BTreeMap<u16, Group>, bool), Error> {
    // get the total stream size
    let stream_size = {
        let stream_pos = reader.seek(SeekFrom::Current(0))?;
        let stream_size = reader.seek(SeekFrom::End(0))?;
        // go the the stream start again
        reader.seek(SeekFrom::Start(stream_pos))?;
        stream_size
    }; 
    // read the number of groups
    reader.read_u32::<LittleEndian>()?;
    // read the number of images
    let image_count = reader.read_u32::<LittleEndian>()?;
    // read the next subfile offset in the stream
    let mut next_subfile_offset = reader.read_u32::<LittleEndian>()?;
    // read the sub-header size
    reader.read_u32::<LittleEndian>()?;
    // read the shared palette byte
    let shared_palette = {
        let shared_palette_byte = reader.read_u8()?;
        shared_palette_byte != 0
    };
    let mut all_images = Vec::new();
    // then read the subfiles
    while (all_images.len() < (image_count as usize)) && ((next_subfile_offset as u64) < stream_size) {
        // seek to the next subfile offset
        reader.seek(SeekFrom::Start(next_subfile_offset as u64))?;
        next_subfile_offset = reader.read_u32::<LittleEndian>()?;
        let data_size = reader.read_u32::<LittleEndian>()?;
        let axis_x = reader.read_u16::<LittleEndian>()?;
        let axis_y = reader.read_u16::<LittleEndian>()?;
        let group_index = reader.read_u16::<LittleEndian>()?;
        let image_index = reader.read_u16::<LittleEndian>()?;
        let linked_index = reader.read_u16::<LittleEndian>()?;
        let uses_shared_palette = {
            let shared_palette_byte = reader.read_u8()?;
            shared_palette_byte != 0
        };
        // next 13 bytes: blank (can be used for comments according to formats.txt)
        reader.seek(SeekFrom::Current(13))?;
        // according to the documentation in formats.txt, this is:
        // "PCX graphic data. If palette data is available, it is the last 768 bytes."
        let mut data = vec![0; data_size as usize];
        reader.read_exact(&mut data[..])?;
        // add the sprite to all_images
        let sprite = Sprite {
            axis: (axis_x, axis_y),
            linked_index,
            uses_shared_palette,
            data,
        };
        let sff_image = SffImage {
            group_index,
            image_index,
            sprite,
        };
        all_images.push(sff_image);
    }
    let mut all_groups = BTreeMap::new();
    let mut sprites = Vec::new();
    for sff_image in all_images {
        let sprite_index = sprites.len();
        let group = all_groups.entry(sff_image.group_index).or_insert(Group(BTreeMap::new()));
        group.0.insert(sff_image.image_index, sprite_index);
        sprites.push(sff_image.sprite);
    }
    Ok((
        sprites,
        all_groups,
        shared_palette,
    ))
}
