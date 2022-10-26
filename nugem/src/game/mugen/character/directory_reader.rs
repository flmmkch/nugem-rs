use super::{Character, file_reader::{self, FileReader}};
use std::{path::Path, fs::read_dir};

pub fn read_directory_characters(directory_path: &Path) -> impl Iterator<Item = Character>
{
    read_dir(directory_path)
        .into_iter()
        .flatten()
        .flat_map(Result::into_iter)
        .filter_map(|entry| {
            match entry.file_type() {
                Ok(file) if file.is_dir() => {
                    let character_file_reader = file_reader::fs::FileReaderFs::new(entry.path().to_path_buf());
                    Character::open(&entry.file_name(), Box::new(character_file_reader))
                },
                Ok(_) => {
                    let file_name = entry.file_name();
                    if let Some(character_name) = Path::new(&file_name).file_stem() {
                        let file_name_string = file_name.to_string_lossy().to_lowercase();
                        let character_file_reader_opt: Option<Box<dyn file_reader::FileReader>> = if file_name_string.ends_with(".zip") {
                            file_reader::zip::FileReaderZip::new(entry.path().to_path_buf()).map(|s| Box::new(s) as Box<dyn FileReader>).ok()
                        } else if file_name_string.ends_with(".rar") {
                            file_reader::rar::FileReaderRar::new(entry.path().to_path_buf()).map(|s| Box::new(s) as Box<dyn FileReader>).ok()
                        } else {
                            None
                        };
                        character_file_reader_opt.and_then(|file_reader| Character::open(&character_name, file_reader))
                    }
                    else {
                        None
                    }
                },
                Err(_) => None,
            }
        })
}