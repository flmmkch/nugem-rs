use log::error;

use crate::game::mugen::format::generic_def::{DefLine, GenericDef};
use std::fs::File;
use std::io::BufReader;
use super::{Command, CommandInput};
use super::command_input_parser::parse_command_input;

pub fn read_cmd_file(cmd_file: File) -> Vec<Command> {
    let mut default_time = 15;
    let mut default_buffer_time = 1;
    let mut commands = Vec::new();
    let mut reading_statedef = false;
    for category in GenericDef::read(BufReader::new(cmd_file)) {
        let cat_name = category.name().to_lowercase();
        match cat_name.as_str() {
            "remap" => {
                log::info!("cmd section not handled: {cat_name}");
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
                let mut command_string : Option<String> = None;
                let mut time : Option<u16> = None;
                let mut buffer_time : Option<u16> = None;
                for line in category.lines() {
                    match line {
                        DefLine::KeyValue(key, value) => {
                            let key_name = key.to_lowercase();
                            match key_name.as_str() {
                                "name" => name = Some(value),
                                "command" => command_string = Some(value),
                                "time" => time = value.parse().ok(),
                                "buffer.time" => buffer_time = value.parse().ok(),
                                other => log::debug!("Unknown command key {other}"),
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
                            error!("Error reading command \"{1}\": {0}", e, name);
                            CommandInput::new(Default::default())
                        }
                    };
                    let time = time.unwrap_or(default_time);
                    let buffer_time = buffer_time.unwrap_or(default_buffer_time);
                    let command = Command::new(name, command_input, time, buffer_time);
                    commands.push(command);
                }
            },
            "statedef -1" => reading_statedef = true,
            state_trigger => {
                // State triggers
                log::info!("state trigger not handled: {state_trigger}");
            },
        }
    }
    commands
}
