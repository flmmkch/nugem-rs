use log::error;

use crate::game::mugen::format::generic_def::{DefLine, Categories};
use std::io::Read;
use super::{Command, CommandInput};
use super::command_input_parser::parse_command_input;

pub fn read_cmd_file<R: Read>(read: R, character_name: &str) -> Vec<Command> {
    let mut default_time = 15;
    let mut default_buffer_time = 1;
    let mut commands = Vec::new();
    let mut _reading_statedef = false;
    for (_, category) in Categories::read_def(read) {
        let cat_name = category.name().to_lowercase();
        match cat_name.as_str() {
            "remap" => {
                log::info!("cmd section {cat_name} not handled for character {character_name}");
                // TODO
            },
            "defaults" => {
                for (_line_number, line) in category.lines() {
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
                let mut command_string : Option<String> = None;
                let mut time : Option<u16> = None;
                let mut buffer_time : Option<u16> = None;
                for (_line_number, line) in category.into_lines() {
                    match line {
                        DefLine::KeyValue(key, value) => {
                            let key_name = key.to_lowercase();
                            match key_name.as_str() {
                                "name" => name = Some(value),
                                "command" => command_string = Some(value),
                                "time" => time = value.parse().ok(),
                                "buffer.time" => buffer_time = value.parse().ok(),
                                other => log::debug!("Unknown command key {other} for character {character_name}"),
                            }
                        },
                        _ => (),
                    }
                }
                if let Some((name, command_string)) = name.zip(command_string) {
                    let command_input = match parse_command_input(&command_string) {
                        Ok(input_states) => {
                            CommandInput::new(input_states)
                        }
                        Err(e) => {
                            error!("Error reading command \"{name}\" for character {character_name}: {e}");
                            CommandInput::new(Default::default())
                        }
                    };
                    let time = time.unwrap_or(default_time);
                    let buffer_time = buffer_time.unwrap_or(default_buffer_time);
                    let command = Command::new(name, command_input, time, buffer_time);
                    commands.push(command);
                }
            },
            "statedef -1" => _reading_statedef = true,
            _state_trigger => {
                // State triggers
                // TODO
            },
        }
    }
    commands
}
