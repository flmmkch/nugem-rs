use crate::game::input;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum InputModifier {
    Normal,
    HoldDown,
    Release{ time: Option<u16> },
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum InputSymbol {
    Direction(input::Directional),
    Buttons(Vec<input::Button>),
}

impl From<input::Directional> for InputSymbol {
    fn from(direction: input::Directional) -> Self {
        InputSymbol::Direction(direction)
    }
}

impl FromIterator<input::Button> for InputSymbol {
    fn from_iter<I: IntoIterator<Item = input::Button>>(iter: I) -> Self {
        let buttons = iter.into_iter().collect();
        InputSymbol::Buttons(buttons)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct CommandInputState {
    pub modifier: InputModifier,
    pub partial: bool,
    pub symbol: InputSymbol,
    /** '>' character in command input expression: if true, there must be no other key pressed or released between the previous and the current symbol */
    pub strict: bool,
}

impl From<InputSymbol> for CommandInputState {
    fn from(symbol: InputSymbol) -> Self {
        CommandInputState {
            modifier: InputModifier::Normal,
            partial: false,
            symbol,
            strict: false,
        }
    }
}


#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct CommandInput {
    states: Vec<CommandInputState>,
}

impl CommandInput {
    pub fn new(successive_inputs: Vec<CommandInputState>) -> CommandInput {
        CommandInput {
            states: successive_inputs,
        }
    }
    pub fn inputs(&self) -> &[CommandInputState] {
        &self.states[..]
    }
}
