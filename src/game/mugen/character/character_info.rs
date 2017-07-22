use std::collections::HashMap;

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
