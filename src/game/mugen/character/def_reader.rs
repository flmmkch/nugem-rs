use std::io::BufRead;
use ::game::mugen::format::generic_def::Categories;

pub struct CharacterInfo {
    pub name: String,
    pub display_name: String,
    pub mugen_version: String,
}

pub fn read_def<T: BufRead>(def_categories: Categories<T>) -> Option<CharacterInfo> {
    let mut character_info = CharacterInfo::new();
    for category in def_categories {
        let cat_name = category.name().to_lowercase();
        match cat_name.as_str() {
            "info" => {
                for key_value in category.key_values() {
                    let key_name = key_value.key().to_lowercase();
                    match key_name.as_str() {
                        "name" => character_info.name = key_value.value().to_owned(),
                        "displayname" => character_info.display_name = key_value.value().to_owned(),
                        "mugenversion" => character_info.mugen_version = key_value.value().to_owned(),
                        _ => (),
                    }
                }
            },
            _ => (),
        }
    }
    if character_info.valid() {
        Some(character_info)
    }
    else {
        None
    }
}

impl CharacterInfo {
    pub fn new() -> CharacterInfo {
        CharacterInfo {
            name: String::new(),
            display_name: String::new(),
            mugen_version: String::new(),
        }
    }
    pub fn valid(&self) -> bool {
        let mut validity = true;
        validity = validity && !self.name.is_empty();
        validity = validity && !self.display_name.is_empty();
        validity
    }
}
