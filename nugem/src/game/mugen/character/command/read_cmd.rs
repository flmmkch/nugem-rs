use std::collections::HashMap;
use ::game::mugen::format::generic_def::{Categories, DefLine, GenericDef};
use std::fs::File;
use std::io::{BufRead, BufReader};
use super::{Command, CommandInput, CommandInputState};
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
                                    match parse_command_input(value.as_str()) {
                                        Ok(input_states) => {
                                            command = Some(CommandInput::new(input_states));
                                        }
                                        Err(e) => {
                                            error!("Error reading command: {}", e);
                                        }
                                    }
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
                if let Some(name_value) = name {
                    if let Some(command_input_value) = command {
                        let command_time = {
                            if let Some(value) = time {
                                value
                            }
                            else {
                                default_time
                            }
                        };
                        let command_buffer_time = {
                            if let Some(value) = buffer_time {
                                value
                            }
                            else {
                                default_buffer_time
                            }
                        };
                        let command = Command::new(name_value, command_input_value, command_time, command_buffer_time);
                        commands.push(command);
                    }
                }
            },
            "statedef -1" => reading_statedef = true,
            _ => {
                // State triggers
            },
        }
    }
    commands
}
