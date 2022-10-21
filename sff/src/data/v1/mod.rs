use thiserror::Error;
use std::io;

mod data;
pub use self::data::{Data, Group, Sprite, Color, Palette};

mod sff;
pub use self::sff::read_sff;

#[derive(Debug, Error)]
pub enum RenderingError<T> {
    #[error("Renderer error")]
    RendererError(T),
    #[error("Invalid SFFv1 sprite data")]
    IoError(#[from] io::Error),
    #[error("Palette {0} not found")]
    PaletteNotFound(usize),
    #[error("Bad manufacturer byte: 0x{0:0X}")]
    BadManufacturerByte(u8),
    #[error("Invalid paintbrush: {0}")]
    InvalidPaintbrush(u8),
    #[error("Bad encoding byte: 0x{0:0X}")]
    BadEncodingByte(u8),
    #[error("Invalid bits per plane: {0}")]
    InvalidBitsPerPlane(u8),
    #[error("Bad reserved byte: 0x{0:0X}")]
    BadReservedByte(u8),
    #[error("Invalid linked sprite number {invalid_index} ({sprite_count} sprites available)")]
    InvalidLinkedSpriteNumber {
        invalid_index: u16,
        sprite_count: usize,
    },
    #[error("Invalid sprite group number {invalid_index} ({sprite_group_count} sprite groups available)")]
    InvalidSpriteGroupNumber {
        invalid_index: u16,
        sprite_group_count: usize,
    },
    #[error("Invalid image number {invalid_index} ({image_count} sprites available in the group)")]
    InvalidImageNumber {
        invalid_index: u16,
        image_count: usize,
    },
}

impl<T> RenderingError<T> {
    fn renderer_error(err: T) -> Self {
        Self::RendererError(err)
    }
}
