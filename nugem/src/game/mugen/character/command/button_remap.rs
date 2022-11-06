use std::collections::HashMap;

use crate::game::input::Button;
use crate::game::mugen::format::generic_def::{Category, DefLine};
use super::command_input_parser::parse_button_symbol;


#[derive(Debug, Default)]
pub struct ButtonRemap(HashMap<Button, Option<Button>>);

impl ButtonRemap {
    pub fn read_cmd_remap(&mut self, category: Category) {
        let remaps = &mut self.0;
        for (line_number, line) in category.lines() {
            match line {
                DefLine::KeyValue(key, value) => {
                    match parse_button_symbol(key) {
                        Ok((_, input)) => {
                            if !value.is_empty() {
                                match parse_button_symbol(value) {
                                    Ok((_, remap)) => { remaps.insert(input, Some(remap)); },
                                    Err(e) => log::error!("Unknown remap button at line {line_number}: {e}"),
                                }
                            }
                            else {
                                remaps.insert(input, None);
                            }
                        },
                        Err(e) => log::error!("Unknown remapped input button at line {line_number}: {e}"),
                    }
                },
                DefLine::Simple(value) => log::error!("Unknown command remap at line {line_number}: {value}"),
            }
        }
    }
    /// Get the remapped button for the given input. If the button was remapped to nothing, then the result will be None and the input must be ignored. If there was no remaped defined, then the input will be returned.
    pub fn remapped(&self, input: Button) -> Option<Button> {
        self.0.get(&input)
            .cloned()
            .unwrap_or(Some(input))
    }
}
