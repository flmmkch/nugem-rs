use super::{AcceptInputState, ButtonState, Directional, PartialState};

/// Input state.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct State {
    pub directional: Directional,
    pub a: ButtonState,
    pub b: ButtonState,
    pub c: ButtonState,
    pub x: ButtonState,
    pub y: ButtonState,
    pub z: ButtonState,
    pub start: ButtonState,
    pub back: ButtonState,
}

impl State {
    pub fn new() -> State {
        State {
            directional: Directional::Neutral,
            a: ButtonState::Up,
            b: ButtonState::Up,
            c: ButtonState::Up,
            x: ButtonState::Up,
            y: ButtonState::Up,
            z: ButtonState::Up,
            start: ButtonState::Up,
            back: ButtonState::Up,
        }
    }
}

macro_rules! take_input {
    ($input: ident, $dst_state: expr, $src_state: expr) => (
        if let Some(actual_state) = $src_state.$input {
            $dst_state.$input = actual_state
        }
    );
}

impl AcceptInputState for State {
    fn accept(&mut self, partial_state: PartialState) {
        if let Some(motion) = partial_state.directional {
            self.directional = self.directional.motion(motion);
        }
        take_input!(a, self, partial_state);
        take_input!(b, self, partial_state);
        take_input!(c, self, partial_state);
        take_input!(x, self, partial_state);
        take_input!(y, self, partial_state);
        take_input!(z, self, partial_state);
        take_input!(start, self, partial_state);
        take_input!(back, self, partial_state);
    }
}
