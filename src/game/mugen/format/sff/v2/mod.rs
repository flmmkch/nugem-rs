use std::io;

mod data;
pub use self::data::{Data, GroupInfo, ImageFormat, PaletteInfo, SpriteInfo};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

mod sff;
pub use self::sff::read_sff;
