mod button;
pub use self::button::Button;

mod button_state;
pub use self::button_state::ButtonState;

mod direction_state;
pub use self::direction_state::DirectionState;

mod directional;
pub use self::directional::Directional;

mod partial_directional;
pub use self::partial_directional::PartialDirectional;

mod state;
pub use self::state::State;

mod partial_state;
pub use self::partial_state::PartialState;

/// Trait for accepting a partial direction input.
pub trait AcceptInputDirectional {
    /// Accepting a partial direction input.
    fn accept(&mut self, partial_direction: PartialDirectional);
}

/// Trait for accepting a partial state input.
pub trait AcceptInputState {
    /// Accepting a partial state input.
    fn accept(&mut self, partial_state: PartialState);
}
