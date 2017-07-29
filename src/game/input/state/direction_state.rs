/// State of a given direction: horizontal or vertical.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum DirectionState {
    Minus,
    Neutral,
    Plus,
}

impl DirectionState {
    pub fn number(&self) -> u8 {
        match *self {
            DirectionState::Minus => 0,
            DirectionState::Neutral => 1,
            DirectionState::Plus => 2,
        }
    }
}
