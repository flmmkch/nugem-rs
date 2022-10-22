mod button;
pub use self::button::Button;

mod button_state;
pub use self::button_state::ButtonState;

mod directional;
pub use self::directional::*;

mod state;
pub use self::state::State;

mod partial_state;
pub use self::partial_state::PartialState;

/// Trait for accepting a partial state input.
pub trait AcceptInputState {
    /// Accepting a partial state input.
    fn accept(&mut self, partial_state: PartialState);
}
