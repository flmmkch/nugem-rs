use super::{Command, CommandConfiguration, CommandInput};

use crate::game::mugen::format::generic_def::{DefLine, Categories, Category};
use std::io::Read;
use super::{ButtonRemap, command_input_parser::parse_command_input};

pub fn read_cmd_file<R: Read>(read: R, character_name: &str) -> CommandConfiguration {
    let mut default_buffer_time = 1;
    let mut default_time = 15;
    let mut commands = Vec::new();
    let mut reading_statedef = false;
    let mut remap = ButtonRemap::default();
    log::info!("Reading CMD file for character {character_name}");
    for (cat_line_number, category) in Categories::read_def(read) {
        let cat_name = category.name().to_lowercase();
        match (cat_name.as_str(), reading_statedef) {
            ("remap", false) => {
                remap.read_cmd_remap(category);
            },
            ("defaults", false) => {
                read_cmd_defaults(category, &mut default_time, &mut default_buffer_time);
            },
            ("command", false) => {
                match read_cmd_command(category) {
                    Ok(command) => commands.push(command),
                    Err((name, e)) => log::error!("Error reading command \"{name}\" at line {cat_line_number}: {e}"),
                }
            },
            ("statedef -1", false) => reading_statedef = true,
            (other_category, false) => log::error!("Unknown CMD category at line {cat_line_number}: {other_category}"),
            (_state_trigger, true) => {
                // State triggers
                // TODO
            },
        }
    }
    CommandConfiguration {
        commands,
        remap,
        default_buffer_time,
        default_time,
    }
}

fn read_cmd_defaults(category: Category, default_time: &mut u16, default_buffer_time: &mut u16) {
    for (line_number, line) in category.lines() {
        match line {
            DefLine::KeyValue(key, value) => {
                let key_name = key.to_lowercase();
                match key_name.as_str() {
                    "command.time" => {
                        if let Ok(value) = value.parse() {
                            // Minimum: 1
                            if value >= 1 {
                                *default_time = value;
                            }
                        }
                    },
                    "command.buffer.time" => {
                        if let Ok(value) = value.parse() {
                            // Minimum: 1
                            if value >= 1 {
                                *default_buffer_time = value;
                            }
                        }
                    },
                    _ => log::error!("Unknown command default key at line {line_number}: {key}={value}"),
                }
            },
            DefLine::Simple(value) => log::error!("Unknown command default at line {line_number}: {value}"),
        }
    }
}

fn read_cmd_command<'a>(category: Category) -> Result<Command, (String, String)> {
    // commands
    let mut name = None;
    let mut command_string : Option<String> = None;
    let mut time : Option<u16> = None;
    let mut buffer_time : Option<u16> = None;
    for (line_number, line) in category.into_lines() {
        match line {
            DefLine::KeyValue(key, value) => {
                let key_name = key.to_lowercase();
                match key_name.as_str() {
                    "name" => name = Some(value),
                    "command" => command_string = Some(value),
                    "time" => time = value.parse().ok(),
                    "buffer.time" => buffer_time = value.parse().ok(),
                    _ => log::error!("Unknown command key at line {line_number}: {key}={value}"),
                }
            },
            DefLine::Simple(value) => log::error!("Unknown command value at line {line_number}: {value}"),
        }
    }
    if let Some((name, command_string)) = name.zip(command_string) {
        let input_states = parse_command_input(&command_string).map_err(|e| (name.clone(), e.to_string()))?;
        let input = CommandInput::new(input_states);
        let command = Command {
            name,
            input,
            time,
            buffer_time
        };
        Ok(command)
    }
    else {
        Err((String::new(), "Missing command name or input".to_owned()))
    }
}
