use super::CommandInput;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Command {
    pub name: String,
    pub input: CommandInput,
    pub time: Option<u16>,
    pub buffer_time: Option<u16>,
}
