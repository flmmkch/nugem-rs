use std::ffi::OsStr;
use std::path::PathBuf;
use super::character_info::{self, CharacterInfo};
use super::{command, file_reader};
use crate::game::mugen::character::air::{read_air_file, Animation};
use crate::game::mugen::format::generic_def::Categories;
use std::collections::HashMap;
use character_info::ValidInfo;
use nugem_sff::v1::Palette;
use log::error;

pub struct Character {
    info: CharacterInfo,
    file_reader: Box<dyn file_reader::FileReader>,
    character_files_path_root: PathBuf,
}

impl std::fmt::Debug for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(Character))
            .field(stringify!(info), &self.info)
            .finish()
    }
}

impl Character {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        Self::name_from_borrowed_info(&self.info)
    }

    fn name_from_borrowed_info(info: &CharacterInfo) -> &str {
        info["info"]["name"].as_str()
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        self.info["info"]["displayname"].as_str()
    }

    /// Opens a character with a given name and file reader
    pub fn open(name: &OsStr, mut file_reader: Box<dyn file_reader::FileReader>) -> Option<Character> {
        let file_names: Vec<PathBuf> = file_reader.file_names().into_iter().flatten().collect();
        let character_info_paths = file_names.iter().filter(|file_name| file_name.extension() == Some(OsStr::new("def")));
        // try the various .def files to find the character definition
        for character_info_path in character_info_paths {
            let character_files_path_root = character_info_path.parent().map(PathBuf::from).unwrap_or(PathBuf::new());
            log::debug!("Reading character info file {}", character_info_path.display());
            let character_info_file = match file_reader.read_file(&character_info_path) {
                Ok(character) => character,
                Err(e) => { log::error!("Failed to read character {0}: {e}", name.to_string_lossy()); return None; },
            };
            let def_info = Categories::read_def(character_info_file);
            let info = character_info::read_def(def_info);
            if info.valid() {
                return Some(Character {
                    info,
                    file_reader,
                    character_files_path_root,
                });
            } else {
                log::debug!("Ignoring file {0} for character info", character_info_path.display());
            }
        }
        None
    }

    pub fn read_data(&mut self) -> Result<nugem_sff::SpriteFile, nugem_sff::LoadingError> {
        let external_palettes_files: Vec<_> = self.read_external_palette_files().collect();
        let sprite_path = self.character_files_path_root.join(&self.info["files"]["sprite"]);
        let sprite_file = self.file_reader.read_file(&sprite_path)?;
        nugem_sff::SpriteFile::read(sprite_file, external_palettes_files)
    }

    fn read_animations_opt(&mut self) -> Option<HashMap<u32, Animation>> {
        let anim_file_name = self.info.get("files").and_then(|f| f.get("anim"))?;
        let anim_file_path = self.character_files_path_root.join(anim_file_name);
        let anim_file = self.file_reader.read_file(&anim_file_path).ok()?;
        let air_file = read_air_file(anim_file);
        Some(air_file)
    }

    pub fn read_animations(&mut self) -> HashMap<u32, Animation> {
        if let Some(air_file) = self.read_animations_opt() {
            air_file
        } else {
            log::error!("Failed to read animation file for character {0}", self.name());
            HashMap::default()
        }
    }

    fn read_external_palette_files<'a>(&'a mut self) -> impl Iterator<Item = Palette> +'a
    {
        let (character_files, file_reader, character_files_path_root) = (&self.info["files"], &mut self.file_reader, &self.character_files_path_root);
        // iterate on palette indices
        core::iter::successors(Some(1u16), |n| n.checked_add(1))
            // palette key from index
            .map_while(move |palette_index| {
                let palette_file_key = format!("pal{}", palette_index);
                character_files.get(&palette_file_key)
            })
            // read the palette file from file name
            .filter_map(move |palette_file_name| {
                let palette_file_path = character_files_path_root.join(palette_file_name);
                let palette_res = file_reader.read_file(&palette_file_path).and_then(Palette::read);
                // read the palette file
                match palette_res {
                    Ok(palette) => Some(palette),
                    Err(e) => {
                        error!("Error reading palette file {}: {}", palette_file_name, e);
                        None
                    },
                }
            })
    }

    fn read_commands_opt(&mut self) -> Option<std::io::Result<Vec<super::command::Command>>>
    {
        let (info, file_reader) = (&self.info, &mut self.file_reader);
        let character_name = Self::name_from_borrowed_info(&info);
        let cmd_file_name = info.get("files").and_then(|f| f.get("cmd"))?;
        let cmd_file_path = self.character_files_path_root.join(&cmd_file_name);
        let result = file_reader.read_file(&cmd_file_path)
            .map(|cmd_file| command::read_cmd_file(cmd_file, character_name));
        Some(result)
    }

    pub fn read_commands(&mut self) -> Result<Vec<super::command::Command>, std::io::Error> {
        if let Some(cmd_result) = self.read_commands_opt() {
            cmd_result
        } else {
            log::error!("Missing command file for character {0}", self.name());
            Ok(Default::default())
        }
    }
}
