use std::io::{Read, Seek};

use crate::bitmap::BitmapRenderer;
use crate::{LoadingError, RenderingError};

pub mod v1;
pub mod v2;

mod reader;
use self::reader::*;

#[derive(Debug)]
pub enum SpriteFile {
    V1(v1::Data),
    V2(v2::Data),
}

impl SpriteFile {
    pub fn palette_count(&self) -> usize {
        use SpriteFile::*;
        match self {
            V1(data) => data.palette_count(),
            V2(data) => data.palette_count(),
        }
    }

    pub fn render_sprite<R: BitmapRenderer>(
        &self,
        bitmap_renderer: &mut R,
        group_index: u16,
        image_index: u16,
        palette_index: usize,
    ) -> Result<(), RenderingError<<R as BitmapRenderer>::Error>> {
        use SpriteFile::*;
        match self {
            V1(data) => {
                data.render_sprite(bitmap_renderer, group_index, image_index, palette_index)
            }
            V2(data) => {
                data.render_sprite(bitmap_renderer, group_index, image_index, palette_index)
            }
        }
    }

    pub fn read<T: Read + Seek, I: IntoIterator<Item = v1::Palette>>(
        mut reader: T,
        external_palettes: I,
    ) -> Result<Self, LoadingError> {
        // first: the signature at the start of the file
        check_signature(&mut reader)?;
        // then: the version bytes
        let mut v_buffer = [0; 4];
        reader.read_exact(&mut v_buffer)?;
        if &v_buffer == v1::Data::version_bytes() {
            let (sprites, groups, shared_palette) = v1::read_sff(reader)?;
            let data = v1::Data::new(sprites, groups, external_palettes, shared_palette);
            Ok(SpriteFile::V1(data))
        } else if &v_buffer == v2::Data::version_bytes() {
            let d = v2::read_sff(reader)?;
            Ok(SpriteFile::V2(d))
        }
        else {
            Err(LoadingError::UnknownVersion)
        }
    }
}

pub(crate) trait SffData {
    fn version_bytes() -> &'static VersionBytes;
    fn palette_count(&self) -> usize;
    fn render_sprite<T: BitmapRenderer>(
        &self,
        bitmap_renderer: &mut T,
        group_index: u16,
        image_index: u16,
        palette_index: usize,
    ) -> Result<(), RenderingError<T::Error>>;
}
