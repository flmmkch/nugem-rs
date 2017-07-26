pub mod v1;
pub mod v2;
use std::io::{self, Read, Seek};
use std::path::{Path, PathBuf};
use ::game::mugen::character::CharacterInfo;
use ::game::graphics::surface::BitmapSurface;

#[derive(Debug)]
pub enum Data {
    V1(v1::Data),
    V2(v2::Data),
}

#[derive(Debug)]
pub enum Error {
    NoSignature,
    ReadError(io::Error),
    UnknownVersion,
    SffV1Error(v1::Error),
    SffV2Error(v2::Error),
    FileOpeningError(PathBuf),
}

pub trait SffData {
    fn palette_count(&self) -> usize;
    fn render_sprite(&self, group_index: u16, image_index: u16, palette_index: usize) -> Option<BitmapSurface>;
}

impl SffData for Data {
    fn palette_count(&self) -> usize {
        match self {
            &Data::V1(ref d) => d.palette_count(),
            &Data::V2(ref d) => d.palette_count(),
        }
    }
    fn render_sprite(&self, group_index: u16, image_index: u16, palette_index: usize) -> Option<BitmapSurface> {
        match self {
            &Data::V1(ref d) => d.render_sprite(group_index, image_index, palette_index),
            &Data::V2(ref d) => d.render_sprite(group_index, image_index, palette_index),
        }
    }
}

pub fn read<T: Read + Seek>(mut reader: T, character_info: &CharacterInfo, character_dir: &Path) -> Result<Data, Error> {
    // first: the signature at the start of the file
    {
        let mut sig_buffer = [0; 12];
        match reader.read_exact(&mut sig_buffer) {
            Ok(()) => {
                if &sig_buffer != b"ElecbyteSpr\0" {
                    return Err(Error::NoSignature);
                }
            },
            Err(e) => return Err(Error::ReadError(e)),
        }
    }
    // then: the version bytes
    let mut v_buffer = [0; 4];
    match reader.read_exact(&mut v_buffer) {
        Ok(()) => {
            if &v_buffer == &[0, 1, 0, 1] {
                match v1::read_sff(reader) {
                    Ok((sprites, groups, shared_palette)) => {
                        let palettes : Vec<_> = v1::pal::PaletteFilesReader::new(character_info, character_dir).collect();
                        let data = v1::Data::new(sprites, groups, palettes, shared_palette);
                        Ok(Data::V1(data))
                    },
                    Err(e) => Err(Error::SffV1Error(e)),
                }
            }
            else {
                if &v_buffer == &[0, 1, 0, 2] {
                    match v2::read_sff(reader) {
                        Ok(d) => Ok(Data::V2(d)),
                        Err(e) => Err(Error::SffV2Error(e)),
                    }
                }
                else {
                    Err(Error::UnknownVersion)
                }
            }
        }
        Err(e) => Err(Error::ReadError(e)),
    }
}

/// Read a little-endian unsigned 32-bit integer
pub fn read_u32<T: Read>(reader: &mut T) -> Result<u32, io::Error> {
    let mut v_buffer = [0; 4];
    reader.read_exact(&mut v_buffer)?;
    let result = v_buffer
                    .iter()
                    .fold((0 as u32, 0), |(mut v, n), c| {
                        let a = ((*c) as u32) << n;
                        v += a;
                        (v, n+8)
                        })
                    .0;
    Ok(result)
}

/// Read a little-endian unsigned 16-bit integer
pub fn read_u16<T: Read>(reader: &mut T) -> Result<u16, io::Error> {
    let mut v_buffer = [0; 2];
    reader.read_exact(&mut v_buffer)?;
    let result = v_buffer
                    .iter()
                    .fold((0 as u16, 0), |(mut v, n), c| {
                        let a = ((*c) as u16) << n;
                        v += a;
                        (v, n+8)
                        })
                    .0;
    Ok(result)
}

/// Read an unsigned 8-bit integer
pub fn read_u8<T: Read>(reader: &mut T) -> Result<u8, io::Error> {
    let mut v_buffer = [0; 1];
    reader.read_exact(&mut v_buffer)?;
    Ok(v_buffer[0])
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_read_u32() {
        use std::io::Cursor;
        {
            let original_buffer = [2, 0, 0, 0, 0, 0, 0, 0];
            let mut reader = Cursor::new(&original_buffer);
            let v = read_u32(&mut reader).unwrap();
            assert_eq!(v, 2);
        }
        {
            let original_buffer = [0, 1, 0, 0, 0, 0, 0, 0];
            let mut reader = Cursor::new(&original_buffer);
            let v = read_u32(&mut reader).unwrap();
            assert_eq!(v, 256);
        }
        {
            let original_buffer = [0, 0, 1, 0, 0, 0, 0, 0];
            let mut reader = Cursor::new(&original_buffer);
            let v = read_u32(&mut reader).unwrap();
            assert_eq!(v, 65536);
        }
        {
            let original_buffer = [0, 1, 1, 0];
            let mut reader = Cursor::new(&original_buffer);
            let v = read_u32(&mut reader).unwrap();
            assert_eq!(v, 65792);
        }
    }
}
