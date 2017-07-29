use super::{AcceptInputDirectional, AcceptInputState, ButtonState, PartialDirectional};

/// Partial input state.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct PartialState {
    pub directional: Option<PartialDirectional>,
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
        if let Some(partial_directional) = partial_state.directional {
            if self.directional.is_some() {
                self.directional.as_mut().unwrap().accept(partial_directional);
            }
            else {
                self.directional = Some(partial_directional);
            }
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
