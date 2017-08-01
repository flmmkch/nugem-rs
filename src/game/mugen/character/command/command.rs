use super::CommandInput;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Command {
    name: String,
    command: CommandInput,
    time: u16,
    buffer_time: u16,
}

impl Command {
    pub fn new(name: String, command: CommandInput, time: u16, buffer_time: u16) -> Command {
        Command {
            name,
            command,
            time,
            buffer_time,
        }
    }
}