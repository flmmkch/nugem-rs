use std::io::BufRead;
use ::game::mugen::format::generic_def::Categories;
use super::{CharacterInfo, ValidInfo};
use std::collections::HashMap;

pub fn read_def<T: BufRead>(def_categories: Categories<T>) -> Option<CharacterInfo> {
    let mut character_info = CharacterInfo::new();
    for category in def_categories {
        let cat_name = category.name().to_lowercase();
        let cat_map = character_info.entry(cat_name).or_insert(HashMap::new());
        for key_value in category.key_values() {
            let key_name = key_value.key().to_lowercase();
            cat_map.insert(key_name, key_value.value().to_owned());
        }
    }
    if character_info.valid() {
        Some(character_info)
    }
    else {
        None
    }
}
