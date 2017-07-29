use std::path::{Path, PathBuf};
use std::fs;
use std::io::BufReader;
use ::game::mugen::format::generic_def::GenericDef;
use super::character_info::{self, CharacterInfo};
use ::game::mugen::format::sff;

#[derive(Debug)]
pub struct Character {
    info: CharacterInfo,
    path: PathBuf,
}

pub struct CharactersDir {
    read_dir: fs::ReadDir,
}

impl Character {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        self.info["info"]["name"].as_str()
    }
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        self.info["info"]["displayname"].as_str()
    }
    fn read_directory(chara_dir_path: &Path) -> Option<Character> {
        if chara_dir_path.is_dir() {
            if let Some(chara_dir_name) = chara_dir_path.file_name() {
                let def_path = chara_dir_path.join(Path::new(&chara_dir_name).with_extension("def"));
                if def_path.is_file() {
                    // read the def file
                    if let Ok(file) = fs::File::open(def_path) {
                        let reader = BufReader::new(file);
                        let def_info = GenericDef::read(reader);
                        if let Some(character_info) = character_info::read_def(def_info) {
                            let character = Character {
                                info: character_info,
                                path: chara_dir_path.to_path_buf(),
                            };
                            return Some(character);
                        }
                        else {
                            return None
                        }
                    }
                }
            }
        }
        None
    }
    pub fn read_data(&self) -> Result<sff::Data, sff::Error> {
        let sprite_path = self.path.join(Path::new(&self.info["files"]["sprite"]));
        let file_res = fs::File::open(&sprite_path);
        if let Ok(file) = file_res {
            let buf_reader = BufReader::new(file);
            sff::read(buf_reader, &self.info, &self.path)
        }
        else {
            Err(sff::Error::FileOpeningError(sprite_path))
        }
    }
}

impl Iterator for CharactersDir {
    type Item = Character;

    fn next(&mut self) -> Option<Character> {
        match self.read_dir.next() {
            Some(entry_result) => {
                match entry_result {
                    Ok(entry) => {
                        let path_buf = entry.path();
                        if let Some(character) = Character::read_directory(path_buf.as_path()) {
                            Some(character)
                        }
                        else {
                            self.next()
                        }
                    },
                    _ => self.next(),
                }
            }
            None => None,
        }
    }
}

pub fn find_characters(directory_path: &Path) -> Option<CharactersDir> {
    match fs::read_dir(directory_path) {
        Ok(main_directory) => {
            Some(CharactersDir {
                read_dir: main_directory
            })
        },
        _ => None,
    }
}
