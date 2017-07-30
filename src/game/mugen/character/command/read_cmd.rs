use std::collections::HashMap;
use ::game::mugen::format::generic_def::{Categories, DefLine, GenericDef};
use std::fs::File;
use std::io::{BufRead, BufReader};
use super::{CommandInput, CommandInputState};

pub fn read_cmd_file(cmd_file: File) {
    let mut default_time = 15;
    let mut default_buffer_time = 15;
    for category in GenericDef::read(BufReader::new(cmd_file)) {
        let cat_name = category.name().to_lowercase();
        match cat_name.as_str() {
            "remap" => {
                // TODO
            },
            "defaults" => {
                for line in category.lines() {
                    match line {
                        DefLine::KeyValue(key, value) => {
                            let key_name = key.to_lowercase();
                            match key_name.as_str() {
                                "command.time" => {
                                    if let Ok(value) = value.parse() {
                                        // Minimum: 1
                                        if value >= 1 {
                                            default_time = value;
                                        }
                                    }
                                },
                                "command.buffer.time" => {
                                    if let Ok(value) = value.parse() {
                                        // Minimum: 1
                                        if value >= 1 {
                                            default_buffer_time = value;
                                        }
                                    }
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }
            },
            "command" => {
                // commands
                let mut name = None;
                let mut command : Option<CommandInput> = None;
                let mut time : Option<u16> = None;
                let mut buffer_time : Option<u16> = None;
                for line in category.lines() {
                    match line {
                        DefLine::KeyValue(key, value) => {
                            let key_name = key.to_lowercase();
                            match key_name.as_str() {
                                "name" => {
                                    name = Some(value);
                                },
                                "command" => {
                                    
                                }
                                "time" => {
                                    if let Ok(value) = value.parse() {
                                        time = Some(value);
                                    }
                                },
                                "buffer.time" => {
                                    if let Ok(value) = value.parse() {
                                        buffer_time = Some(value);
                                    }
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }
            },
            _ => {
                // State triggers
            },
        }
    }
}
