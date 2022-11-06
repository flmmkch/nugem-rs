mod command_input;

pub use self::command_input::*;

mod command;
pub use self::command::*;

mod command_input_parser;

mod read_cmd;
pub use self::read_cmd::*;

mod button_remap;
pub use self::button_remap::*;

#[derive(Debug)]
pub struct CommandConfiguration {
    pub commands: Vec<Command>,
    pub remap: ButtonRemap,
    /// Default value for the "time" parameter of a Command. Minimum 1.
    pub default_time: u16,
    /// Default value for the "buffer.time" parameter of a Command. Minimum 1, maximum 30.
    pub default_buffer_time: u16,

}
