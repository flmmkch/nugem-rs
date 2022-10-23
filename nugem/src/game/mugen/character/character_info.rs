use std::collections::HashMap;
use std::io::BufRead;
use crate::game::mugen::format::generic_def::{Categories, DefLine};

pub type CharacterInfo = HashMap<String, HashMap<String, String>>;

pub trait ValidInfo {
    fn valid(&self) -> bool;
}

impl ValidInfo for CharacterInfo {
    fn valid(&self) -> bool {
        let mut validity = true;
        validity = validity && self.contains_key("info");
        validity = validity && self["info"].contains_key("name");
        validity = validity && self["info"].contains_key("displayname");
        validity = validity && self["files"].contains_key("sprite");
        validity
    }
}

pub fn read_def<T: BufRead>(def_categories: Categories<T>) -> Option<CharacterInfo> {
    let mut character_info = CharacterInfo::new();
    for (_, category) in def_categories {
        let cat_name = category.name().to_lowercase();
        let cat_map = character_info.entry(cat_name).or_insert(HashMap::new());
        for (_, line) in category.into_lines() {
            match line {
                DefLine::KeyValue(key, value) => {
                    let key_name = key.to_lowercase();
                    cat_map.insert(key_name, value);
                },
                _ => (),
            }
        }
    }
    if character_info.valid() {
        Some(character_info)
    }
    else {
        log::error!("Invalid character info");
        None
    }
}

