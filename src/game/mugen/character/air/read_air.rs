use std::collections::HashMap;
use ::game::mugen::format::generic_def::{Categories, DefLine, GenericDef};
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use super::*;

pub fn read_air_file(cmd_file: File) -> HashMap<u32, Animation>  {
    let mut result_map = HashMap::new();
    for category in GenericDef::read(BufReader::new(cmd_file)) {
        let cat_name = category.name().to_lowercase();
        lazy_static! {
            static ref REGEX_BEGIN_ACTION: Regex = Regex::new("^begin action ([0-9]+)$").unwrap();
        }
        if let Some(c) = REGEX_BEGIN_ACTION.captures(cat_name.as_str()) {
            if let Some(digits) = c.get(1) {
                if let Ok(number) = digits.as_str().parse::<u32>() {
                    let mut line_iterator = category.lines();
                    // let mut default_normal_collision = None;
                    // let mut default_attack_collision = None;
                    let mut current_animation_frames = Vec::new();
                    while let Some(line) = line_iterator.next() {
                        lazy_static! {
                            static ref REGEX_CLSN_DEFAULT: Regex = Regex::new(r"^Clsn(1|2)Default: ([0-9]+)$").unwrap();
                            static ref REGEX_CLSN: Regex = Regex::new(r"^Clsn(1|2): ([0-9]+)$").unwrap();
                            static ref REGEX_CLSN_KEY: Regex = Regex::new(r"^Clsn\[([0-9+])\]$").unwrap();
                        }
                        if let DefLine::Simple(line_string) = line {
                            if let Some(collision_definition) = REGEX_CLSN.captures(line_string.as_str()) {
                                if collision_definition.get(1).is_some() && collision_definition.get(2).is_some() {
                                }
                            }
                            else {
                                if let Some(collision_default) = REGEX_CLSN_DEFAULT.captures(&line_string.as_str()) {
                                }
                                else {
                                    // try for a frame line
                                    let strings: Vec<&str> = line_string.split(',').collect();
                                    if strings.len() >= 5 {
                                        let group_res = strings[0].trim().parse();
                                        let image_res = strings[1].trim().parse();
                                        let offset_x_res = strings[2].trim().parse();
                                        let offset_y_res = strings[3].trim().parse();
                                        let ticks_res = strings[4].trim().parse();
                                        if group_res.is_ok() && image_res.is_ok() && offset_x_res.is_ok() && offset_y_res.is_ok() && ticks_res.is_ok() {
                                            current_animation_frames.push(AnimationFrame {
                                                group: group_res.unwrap(),
                                                image: image_res.unwrap(),
                                                offset: (offset_x_res.unwrap(), offset_y_res.unwrap()),
                                                ticks: Some(ticks_res.unwrap()),
                                                flip: (false, false),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let animation = Animation::new(vec![AnimationSteps::new(Vec::new(), current_animation_frames)]);
                    result_map.insert(number, animation);
                }
            }
        }
    }
    result_map
}
