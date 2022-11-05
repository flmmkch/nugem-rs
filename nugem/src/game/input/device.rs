use super::{State, PartialState, AcceptInputState};

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum DeviceType {
    Keyboard(u32),
    GameController,
}

#[derive(Debug)]
pub struct Device {
    device_type: DeviceType,
    name: String,
    current_state: State,
}

impl Device {
    pub fn new(device_type: DeviceType, name: String, initial_state: State) -> Device {
        Device {
            device_type,
            name,
            current_state: initial_state,
        }
    }
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    #[allow(dead_code)]
    pub fn state(&self) -> &State {
        &self.current_state
    }
    pub fn process_state(&mut self, partial_state: PartialState,) -> Option<PartialState> {
        let original_state = self.current_state.clone();
        self.current_state.accept(partial_state.clone());
        if &self.current_state != &original_state {
            Some(partial_state)
        }
        else {
            None
        }
    }
}
