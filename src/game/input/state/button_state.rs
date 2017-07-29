/// State of a single button.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum ButtonState {
    Up,
    Down,
}