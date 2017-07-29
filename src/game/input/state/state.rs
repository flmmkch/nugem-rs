use super::{AcceptInputDirectional, AcceptInputState, ButtonState, Directional, PartialState};

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

macro_rules! take_input {
    ($input: ident, $dst_state: expr, $src_state: expr) => (
        if let Some(actual_state) = $src_state.$input {
            $dst_state.$input = actual_state
        }
    );
}

impl AcceptInputState for State {
    fn accept(&mut self, partial_state: PartialState) {
        if let Some(partial_directional) = partial_state.directional {
            self.directional.accept(partial_directional);
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
