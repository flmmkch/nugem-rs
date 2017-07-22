mod data;
pub use self::data::{Data, Group, Sprite, Color, Palette};

mod sff;
pub use self::sff::read_sff;

pub mod pal;

use std::io;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidPcxData(String),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}
