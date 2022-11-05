use super::{State, PartialState, AcceptInputState};

#[derive(Clone, Debug)]
pub enum Device {
    Keyboard { name: String, keyboard_id: u32, current_state: State },
    Gamepad { name: String, gamepad_id: gilrs::GamepadId, current_state: State },
}

impl Device {
    pub fn new_keyboard(keyboard_id: u32) -> Device {
        let name = format!("Keyboard {keyboard_id}");
        Device::Keyboard {
            name,
            keyboard_id,
            current_state: State::new(),
        }
    }
    pub fn new_gamepad(gamepad: gilrs::Gamepad) -> Device {
        let name = gamepad.name().to_owned();
        Device::Gamepad {
            name,
            gamepad_id: gamepad.id(),
            current_state: State::new(),
        }
    }
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            Device::Keyboard { name, .. } => name.as_str(),
            Device::Gamepad { name, .. } => name.as_str(),
        }
    }
    #[allow(dead_code)]
    pub fn state(&self) -> &State {
        match self {
            Device::Keyboard { current_state, .. } => current_state,
            Device::Gamepad { current_state, .. } => current_state,
        }
    }
    #[allow(dead_code)]
    pub fn state_mut(&mut self) -> &mut State {
        match self {
            Device::Keyboard { current_state, .. } => current_state,
            Device::Gamepad { current_state, .. } => current_state,
        }
    }
    pub fn process_state(&mut self, partial_state: PartialState,) -> Option<PartialState> {
        let original_state = self.state().clone();
        self.state_mut().accept(partial_state.clone());
        if self.state() != &original_state {
            Some(partial_state)
        }
        else {
            None
        }
    }
}
