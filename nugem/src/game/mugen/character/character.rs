use std::path::{Path, PathBuf};
use std::fs;
use std::io::BufReader;
use super::character_info::{self, CharacterInfo};
use super::command;
use crate::game::mugen::character::air::{read_air_file, Animation};
use crate::game::mugen::format::generic_def::Categories;
use std::collections::HashMap;
use nugem_sff::v1::Palette;
use log::error;

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
            let chara_dir_name = chara_dir_path.file_name()?;
            let def_path = chara_dir_path.join(Path::new(&chara_dir_name).with_extension("def"));
            if def_path.is_file() {
                // read the def file
                let file = fs::File::open(def_path).ok()?;
                let def_info = Categories::read_def(file);
                let character_info = character_info::read_def::<BufReader<fs::File>>(def_info)?;
                let character = Character {
                    info: character_info,
                    path: chara_dir_path.to_path_buf(),
                };
                return Some(character);
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
        if let Some(anim_file_name) = self.info.get("files").and_then(|f| f.get("anim")) {
            let anim_file_path = self.path.join(&anim_file_name);
            if let Ok(anim_file) = fs::File::open(&anim_file_path) {
                return read_air_file(anim_file);
            } else {
                log::error!("Failed to read animation file for character {0}", self.name())
            }
        } else {
            log::error!("Missing animation file for character {0}", self.name())
        }
        HashMap::default()
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

    pub fn read_commands(&self) -> Result<Vec<super::command::Command>, std::io::Error> {
        if let Some(cmd_file_name) = self.info.get("files").and_then(|f| f.get("cmd")) {
            let cmd_file_path = self.path.join(&cmd_file_name);
            let cmd_file = fs::File::open(&cmd_file_path)?;
            Ok(command::read_cmd_file(cmd_file, self.name()))
        } else {
            log::error!("Missing command file for character {0}", self.name());
            Ok(Default::default())
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
