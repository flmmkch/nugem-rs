
use thiserror::Error;
use std::io;

mod data;
pub use self::data::{Data, GroupInfo, ImageFormat, PaletteInfo, SpriteInfo};

mod sff;
pub use self::sff::read_sff;

#[derive(Debug, Error)]
pub enum RenderingError<R> {
    #[error("Renderer error")]
    RendererError(R),
    #[error("Invalid SFFv2 sprite data")]
    IoError(#[from] io::Error),
    #[error("Null copy length")]
    NullCopyLength,
    #[error("Invalid image format {0}")]
    InvalidImageFormat(u8),
    #[error("Invalid sprite group number {invalid_index} ({sprite_group_count} sprite groups available)")]
    InvalidSpriteGroupNumber {
        invalid_index: u16,
        sprite_group_count: usize,
    },
    #[error("Invalid sprite number {invalid_index} ({sprite_count} sprites available in the group)")]
    InvalidSpriteNumber {
        invalid_index: u16,
        sprite_count: usize,
    },
}

impl<T> RenderingError<T> {
    fn renderer_error(err: T) -> Self {
        Self::RendererError(err)
    }
}

