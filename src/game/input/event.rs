use super::PartialState;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Event {
    pub device_id: usize,
    pub timestamp: u32,
    pub partial_state: PartialState,
}