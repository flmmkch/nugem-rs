use super::{AcceptInputState, ButtonState, DirectionalMotion};

/// Partial input state.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct PartialState {
    pub directional: Option<DirectionalMotion>,
    pub a: Option<ButtonState>,
    pub b: Option<ButtonState>,
    pub c: Option<ButtonState>,
    pub x: Option<ButtonState>,
    pub y: Option<ButtonState>,
    pub z: Option<ButtonState>,
    pub start: Option<ButtonState>,
    pub back: Option<ButtonState>,
}

impl PartialState {
    pub fn new() -> PartialState {
        PartialState {
            directional: None,
            a: None,
            b: None,
            c: None,
            x: None,
            y: None,
            z: None,
            start: None,
            back: None,
        }
    }
}

macro_rules! take_optional_input {
    ($input: ident, $dst_state: expr, $src_state: expr) => (
        if let Some(actual_state) = $src_state.$input {
            $dst_state.$input = Some(actual_state)
        }
    );
}

impl AcceptInputState for PartialState {
    fn accept(&mut self, partial_state: PartialState) {
        if let Some(motion) = partial_state.directional {
            let direction = self.directional.map(|d| d.accept_motion(motion)).unwrap_or(motion);
            self.directional = Some(direction);
        }
        take_optional_input!(a, self, partial_state);
        take_optional_input!(b, self, partial_state);
        take_optional_input!(c, self, partial_state);
        take_optional_input!(x, self, partial_state);
        take_optional_input!(y, self, partial_state);
        take_optional_input!(z, self, partial_state);
        take_optional_input!(start, self, partial_state);
        take_optional_input!(back, self, partial_state);
    }
}
