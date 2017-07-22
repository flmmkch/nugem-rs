use std::io::{self, Read};

#[derive(Debug)]
pub struct Data {}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

pub fn read<T: Read>(_: T) -> Result<Data, Error> {
    Ok(Data{})
}
