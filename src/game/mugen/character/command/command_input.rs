use ::game::input;

/// Input entry with a potential modified, such as held down or released
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum ModifiedInput<T> {
    Normal(T),
    HoldDown(T),
    Release(T, Option<u16>),
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct CommandInputState {
    pub directional: Option<ModifiedInput<input::PartialDirectional>>,
    pub button_presses: Vec<ModifiedInput<input::Button>>,
    pub strict: bool,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct CommandInput {
    successive_inputs: Vec<CommandInputState>,
}

impl CommandInputState {
    pub fn new() -> CommandInputState {
        CommandInputState {
            directional: None,
            button_presses: Vec::new(),
            strict: false,
        }
    }
}

impl CommandInput {
    pub fn new(successive_inputs: Vec<CommandInputState>) -> CommandInput {
        CommandInput {
            successive_inputs,
        }
    }
    pub fn inputs(&self) -> &[CommandInputState] {
        &self.successive_inputs[..]
    }
}
