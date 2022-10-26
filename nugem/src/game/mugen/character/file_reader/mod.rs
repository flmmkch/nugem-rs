use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

pub mod rar;

pub mod fs;

pub mod zip;

pub trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}

pub trait FileReader {
    fn read_file<'a>(&'a mut self, path: &Path) -> std::io::Result<Box<dyn ReadSeek + 'a>>;

    fn file_names<'a>(&'a mut self) -> std::io::Result<Box<dyn Iterator<Item = PathBuf> + 'a>>;
}

fn file_not_found_error(path: &Path) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, format!("File not found: {}", path.display()).to_owned())
}
fn invalid_path_unicode_error(path: &Path) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, format!("Invalid Unicode in path: {}", path.display()))
}

fn canonicalize_chars<I: Iterator<Item = char>>(chars: I) -> impl Iterator<Item = char> {
    chars.map(|c| match c { '\\' => '/', _ => c.to_ascii_lowercase()})
}

fn file_name_match(expected: &str, tested: &str) -> bool {
    canonicalize_chars(expected.chars()).eq(canonicalize_chars(tested.chars()))
}