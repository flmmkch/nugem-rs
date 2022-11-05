use super::{PartialState, Device};

#[derive(Clone, Debug)]
pub struct Event<'a> {
    pub device: &'a Device,
    pub partial_state: PartialState,
}