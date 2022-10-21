use super::{State, PartialState, AcceptInputState};
use sdl2;
use std::fmt;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum DeviceType {
    GameController,
}

#[derive(Debug)]
pub struct Device {
    device_type: DeviceType,
    name: String,
    current_state: State,
    #[allow(dead_code)]
    device_internal: DeviceInternal,
}

pub enum DeviceInternal {
    GameController(sdl2::controller::GameController),
}

impl Device {
    pub fn new(device_type: DeviceType, name: String, initial_state: State, device_internal: DeviceInternal) -> Device {
        Device {
            device_type,
            name,
            current_state: initial_state,
            device_internal,
        }
    }
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }
    #[allow(dead_code)]
    pub fn state(&self) -> &State {
        &self.current_state
    }
    pub fn process_state(&mut self, partial_state: PartialState) -> Option<PartialState> {
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

impl fmt::Debug for DeviceInternal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let self_string = {
            match *self {
                DeviceInternal::GameController(_) => "GameController(..)"
            }
        };
        write!(f, "{}", self_string)
    }
}