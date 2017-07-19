use std::io::{self, Read, Seek, SeekFrom};
use super::{read_u32, read_u16, read_u8};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Sprite {
    pub axis: (u16, u16),
    pub linked_index: u16,
    pub uses_shared_palette: bool,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Group(BTreeMap<u16, Sprite>);

#[derive(Debug)]
pub struct Data {
    pub shared_palette: bool,
    pub groups: BTreeMap<u16, Group>,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

struct SffImage {
    pub group_index: u16,
    pub image_index: u16,
    pub sprite: Sprite,
}

pub fn read<T: Read + Seek>(mut reader: T) -> Result<Data, Error> {
    // get the total stream size
    let stream_size = {
        let stream_pos = reader.seek(SeekFrom::Current(0))?;
        let stream_size = reader.seek(SeekFrom::End(0))?;
        // go the the stream start again
        reader.seek(SeekFrom::Start(stream_pos))?;
        stream_size
    }; 
    // read the number of groups
    let group_count = read_u32(&mut reader)?;
    // read the number of images
    let image_count = read_u32(&mut reader)?;
    // read the next subfile offset in the stream
    let mut next_subfile_offset = read_u32(&mut reader)?;
    // read the sub-header size
    let subheader_size = read_u32(&mut reader)?;
    // read the shared palette byte
    let shared_palette = {
        let shared_palette_byte = read_u8(&mut reader)?;
        shared_palette_byte != 0
    };
    let groups = {
        let mut all_images = Vec::new();
        let mut stream_pos = reader.seek(SeekFrom::Current(0))?;
        // then read the subfiles
        while (all_images.len() < (image_count as usize)) && ((next_subfile_offset as u64) < stream_size) {
            // seek to the next subfile offset
            stream_pos = reader.seek(SeekFrom::Start(next_subfile_offset as u64))?;
            next_subfile_offset = read_u32(&mut reader)?;
            let data_size = read_u32(&mut reader)?;
            let axis_x = read_u16(&mut reader)?;
            let axis_y = read_u16(&mut reader)?;
            let group_index = read_u16(&mut reader)?;
            let image_index = read_u16(&mut reader)?;
            let linked_index = read_u16(&mut reader)?;
            let uses_shared_palette = {
                let shared_palette_byte = read_u8(&mut reader)?;
                shared_palette_byte != 0
            };
            // skipping the next 12 bytes: byte 19 to byte 31
            reader.seek(SeekFrom::Current(12))?;
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
        for sff_image in all_images {
            let group = all_groups.entry(sff_image.group_index).or_insert(Group(BTreeMap::new()));
            group.0.insert(sff_image.image_index, sff_image.sprite);
        }
        all_groups
    };
    Ok(Data{
        shared_palette,
        groups: groups,
    })
}
