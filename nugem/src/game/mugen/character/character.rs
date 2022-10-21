use std::path::{Path, PathBuf};
use std::fs;
use std::io::BufReader;
use ::game::mugen::format::generic_def::GenericDef;
use super::character_info::{self, CharacterInfo};
use ::game::mugen::character::air::{read_air_file, Animation};
use std::collections::HashMap;
use nugem_sff::v1::Palette;

#[derive(Debug, Clone)]
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
    pub fn read_data(&self) -> Result<nugem_sff::SpriteFile, nugem_sff::LoadingError> {
        let external_palettes_files: Vec<_> = self.read_external_palette_files().collect();
        let sprite_path = self.path.join(Path::new(&self.info["files"]["sprite"]));
        let file = fs::File::open(&sprite_path)?;
        let buf_reader = BufReader::new(file);
        nugem_sff::SpriteFile::read(buf_reader, external_palettes_files)
    }
    pub fn read_animations(&self) -> HashMap<u32, Animation> {
        let sprite_path = self.path.join(Path::new(&self.info["files"]["anim"]));
        let file = fs::File::open(&sprite_path).unwrap(); // TODO don't use unwrap
        read_air_file(file)
    }

    fn read_character_palette_file(palette_file: &Path) -> Result<Palette, std::io::Error>
    {
        let file = fs::File::open(palette_file)?;
        let reader = BufReader::new(file);
        Palette::read(reader)
    }

    fn read_external_palette_files<'a>(&'a self) -> impl Iterator<Item = Palette> +'a
    {
        let character_files = &self.info["files"];
        // iterate on palette indices
        core::iter::successors(Some(1u16), |n| n.checked_add(1))
            // palette key from index
            .map_while(move |palette_index| {
                let palette_file_key = format!("pal{}", palette_index);
                character_files.get(&palette_file_key)
            })
            // read the palette file from file name
            .filter_map(move |palette_file_name| {
                // read the palette file
                let palette_file_path = self.path.join(&palette_file_name);
                match Self::read_character_palette_file(&palette_file_path) {
                    Ok(palette) => Some(palette),
                    Err(e) => {
                        error!("Error reading palette file {}: {}", palette_file_path.display(), e);
                        None
                    },
                }
            })
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