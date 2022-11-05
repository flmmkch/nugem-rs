use super::*;
use winit::event::{DeviceId, ElementState, KeyboardInput};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Manager {
    devices_map: HashMap<DeviceKey, Device>,
    gilrs: Option<gilrs::Gilrs>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum DeviceKey {
    Keyboard { device_id: DeviceId, keyboard_id: u32 },
    Gamepad { gamepad_id: gilrs::GamepadId },
}

impl From<ElementState> for ButtonState {
    fn from(value: ElementState) -> Self {
        match value {
            ElementState::Pressed => ButtonState::Down,
            ElementState::Released => ButtonState::Up,
        }
    }
}

fn controller_button(button: gilrs::Button) -> Option<Button> {
    match button {
        gilrs::Button::West => Some(Button::A),
        gilrs::Button::North => Some(Button::B),
        gilrs::Button::South => Some(Button::X),
        gilrs::Button::East => Some(Button::Y),
        gilrs::Button::C => Some(Button::C),
        gilrs::Button::Z => Some(Button::Z),
        gilrs::Button::Select => Some(Button::Back),
        gilrs::Button::Start => Some(Button::Start),
        _ => None,
    }
}

pub const MOTION_AXIS_THRESHOLD : f32 = 0.3;

fn initialize_gamepad(gamepad_id: gilrs::GamepadId, gilrs: &gilrs::Gilrs) -> super::Device {
    super::Device::new_gamepad(gilrs.gamepad(gamepad_id))
}

macro_rules! process_controller_event {
    ($input: expr, $input_ident: ident, $gamepad_id: expr, $manager:ident) => {{
        let device_key = DeviceKey::Gamepad { gamepad_id: $gamepad_id };
        let device = $manager.devices_map.entry(device_key).or_insert_with(|| initialize_gamepad($gamepad_id, $manager.gilrs.as_ref().unwrap()));
        let processed_partial_state_opt = {
            let mut partial_state = PartialState::new();
            partial_state.$input_ident = Some($input);
            device.process_state(partial_state)
        };
        if let Some(partial_state) = processed_partial_state_opt {
            let event = super::event::Event {
                device,
                partial_state,
            };
            Some(event)
        }
        else {
            None
        }
    }}
}

impl Manager {
    pub fn initialize() -> Self {
        let gilrs = match gilrs::Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(e) => { log::error!("Error initializing controller handling: {e}"); None },
        };
        let manager = Self {
            devices_map: Default::default(),
            gilrs,
        };
        manager
    }
    pub fn process_keyboard_input_event(&mut self, device_id: DeviceId, keyboard_input: KeyboardInput) -> Option<event::Event> {
        use winit::event::VirtualKeyCode;
        use DirectionalMotion::*;
        use DirectionState::*;
        let KeyboardInput { scancode: _, virtual_keycode, state, .. } = keyboard_input;

        macro_rules! process_keyboard_event {
            ($input: expr, $input_ident: ident, $device_id: expr, $keyboard_id: expr) => {{
                let device_key = DeviceKey::Keyboard { keyboard_id: $keyboard_id, device_id };
                let device = self.devices_map.entry(device_key).or_insert_with(|| Device::new_keyboard($keyboard_id));
                let processed_partial_state_opt = {
                    let mut partial_state = PartialState::new();
                    partial_state.$input_ident = Some($input);
                    device.process_state(partial_state)
                };
                if let Some(partial_state) = processed_partial_state_opt {
                    let event = super::event::Event {
                        device,
                        partial_state,
                    };
                    Some(event)
                }
                else {
                    None
                }
            }}
        }

        let button_state: ButtonState = state.into();
        // TODO handle non-qwerty keyboards
        match (virtual_keycode, button_state) {
            // alphanumerical keys mapping: WASD for direction, U/I/O for A/B/C buttons, J/K/L for X/Y/Z buttons, return (enter) for start, backspace for back
            (Some(VirtualKeyCode::A), ButtonState::Down) => process_keyboard_event!(Horizontal(Minus), directional, device_id, 0),
            (Some(VirtualKeyCode::D), ButtonState::Down) => process_keyboard_event!(Horizontal(Plus), directional, device_id, 0),
            (Some(VirtualKeyCode::W), ButtonState::Down) => process_keyboard_event!(Vertical(Minus), directional, device_id, 0),
            (Some(VirtualKeyCode::S), ButtonState::Down) => process_keyboard_event!(Vertical(Plus), directional, device_id, 0),
            (Some(VirtualKeyCode::A), ButtonState::Up) => process_keyboard_event!(Horizontal(Neutral), directional, device_id, 0),
            (Some(VirtualKeyCode::D), ButtonState::Up) => process_keyboard_event!(Horizontal(Neutral), directional, device_id, 0),
            (Some(VirtualKeyCode::W), ButtonState::Up) => process_keyboard_event!(Vertical(Neutral), directional, device_id, 0),
            (Some(VirtualKeyCode::S), ButtonState::Up) => process_keyboard_event!(Vertical(Neutral), directional, device_id, 0),
            (Some(VirtualKeyCode::U), _) => process_keyboard_event!(button_state, a, device_id, 0),
            (Some(VirtualKeyCode::I), _) => process_keyboard_event!(button_state, b, device_id, 0),
            (Some(VirtualKeyCode::O), _) => process_keyboard_event!(button_state, c, device_id, 0),
            (Some(VirtualKeyCode::J), _) => process_keyboard_event!(button_state, x, device_id, 0),
            (Some(VirtualKeyCode::K), _) => process_keyboard_event!(button_state, y, device_id, 0),
            (Some(VirtualKeyCode::L), _) => process_keyboard_event!(button_state, z, device_id, 0),
            (Some(VirtualKeyCode::Back), _) => process_keyboard_event!(button_state, back, device_id, 0),
            (Some(VirtualKeyCode::Return), _) => process_keyboard_event!(button_state, start, device_id, 0),
            // numpad keys mapping: directional arrows for direction, numpad 7/8/9 for A/B/C buttons, numpad 4/5/6 for X/Y/Z buttons, numpad enter for start, numpad comma for back
            (Some(VirtualKeyCode::Left), ButtonState::Down) => process_keyboard_event!(Horizontal(Minus), directional, device_id, 1),
            (Some(VirtualKeyCode::Right), ButtonState::Down) => process_keyboard_event!(Horizontal(Plus), directional, device_id, 1),
            (Some(VirtualKeyCode::Up), ButtonState::Down) => process_keyboard_event!(Vertical(Minus), directional, device_id, 1),
            (Some(VirtualKeyCode::Down), ButtonState::Down) => process_keyboard_event!(Vertical(Plus), directional, device_id, 1),
            (Some(VirtualKeyCode::Left), ButtonState::Up) => process_keyboard_event!(Horizontal(Neutral), directional, device_id, 1),
            (Some(VirtualKeyCode::Right), ButtonState::Up) => process_keyboard_event!(Horizontal(Neutral), directional, device_id, 1),
            (Some(VirtualKeyCode::Up), ButtonState::Up) => process_keyboard_event!(Vertical(Neutral), directional, device_id, 1),
            (Some(VirtualKeyCode::Down), ButtonState::Up) => process_keyboard_event!(Vertical(Neutral), directional, device_id, 1),
            (Some(VirtualKeyCode::Numpad7), _) => process_keyboard_event!(button_state, a, device_id, 1),
            (Some(VirtualKeyCode::Numpad8), _) => process_keyboard_event!(button_state, b, device_id, 1),
            (Some(VirtualKeyCode::Numpad9), _) => process_keyboard_event!(button_state, c, device_id, 1),
            (Some(VirtualKeyCode::Numpad4), _) => process_keyboard_event!(button_state, x, device_id, 1),
            (Some(VirtualKeyCode::Numpad5), _) => process_keyboard_event!(button_state, y, device_id, 1),
            (Some(VirtualKeyCode::Numpad6), _) => process_keyboard_event!(button_state, z, device_id, 1),
            (Some(VirtualKeyCode::NumpadComma), _) => process_keyboard_event!(button_state, back, device_id, 1),
            (Some(VirtualKeyCode::NumpadEnter), _) => process_keyboard_event!(button_state, start, device_id, 1),
            _ => None,
        }
    }
    pub fn process_controller_axis_event(&mut self, gamepad_id: gilrs::GamepadId, axis: gilrs::Axis, value: f32) -> Option<event::Event> {
        let direction_motion_opt = {
            use DirectionalMotion::*;
            use DirectionState::*;
            let gilrs = self.gilrs.as_ref().unwrap();
            let gamepad = gilrs.gamepad(gamepad_id);
            let deadzone = gamepad.axis_code(axis).and_then(|code| gamepad.deadzone(code)).unwrap_or(MOTION_AXIS_THRESHOLD);
            let positive = value > deadzone;
            let negative = value < -deadzone;
            match axis {
                gilrs::Axis::LeftStickX | gilrs::Axis::DPadX => {
                    // horizontal
                    match (positive, negative) {
                        (true, false) => Some(Horizontal(Plus)),
                        (false, true) => Some(Horizontal(Minus)),
                        (false, false) => Some(Horizontal(Neutral)),
                        _ => None
                    }
                },
                gilrs::Axis::LeftStickY | gilrs::Axis::DPadY => {
                    // vertical
                    match (positive, negative) {
                        (true, false) => Some(Vertical(Minus)),
                        (false, true) => Some(Vertical(Plus)),
                        (false, false) => Some(Vertical(Neutral)),
                        _ => None
                    }
                },
                _ => None,
            }
        };
        if let Some(direction_motion) = direction_motion_opt {
            process_controller_event!(direction_motion, directional, gamepad_id, self)
        }
        else {
            None
        }
    }
    pub fn process_next_gamepad_event(&mut self) -> Option<event::Event> {
        macro_rules! process_controller_button {
            ($input_state_option: expr, $input_button: expr, $which: expr) => (
                match $input_button {
                    Button::A => process_controller_event!($input_state_option, a, $which, self),
                    Button::B => process_controller_event!($input_state_option, b, $which, self),
                    Button::C => process_controller_event!($input_state_option, c, $which, self),
                    Button::X => process_controller_event!($input_state_option, x, $which, self),
                    Button::Y => process_controller_event!($input_state_option, y, $which, self),
                    Button::Z => process_controller_event!($input_state_option, z, $which, self),
                    Button::Start => process_controller_event!($input_state_option, start, $which, self),
                    Button::Back => process_controller_event!($input_state_option, back, $which, self),
                }
            )
        }

        if let Some((gilrs, event)) = self.gilrs.as_mut().and_then(|g| g.next_event().map(|e| (g, e))) {
            match event {
                gilrs::Event { id, event: gilrs::EventType::ButtonPressed(gilrs_button, _), .. } => {
                    if let Some(input_button) = controller_button(gilrs_button) {
                        process_controller_button!(ButtonState::Down, input_button, id)
                    }
                    else {
                        None
                    }
                },
                gilrs::Event { id, event: gilrs::EventType::ButtonReleased(gilrs_button, _), .. } => {
                    if let Some(input_button) = controller_button(gilrs_button) {
                        process_controller_button!(ButtonState::Up, input_button, id)
                    }
                    else {
                        None
                    }
                },
                gilrs::Event { id, event: gilrs::EventType::AxisChanged(axis, value, _), .. } => {
                    self.process_controller_axis_event(id, axis, value)
                },
                gilrs::Event { id: gamepad_id, event: gilrs::EventType::Connected, .. } => {
                    log::info!("Controller connected : {0}.", gilrs.gamepad(gamepad_id).name());
                    self.devices_map.insert(DeviceKey::Gamepad { gamepad_id }, initialize_gamepad(gamepad_id, gilrs));
                    None
                }
                gilrs::Event { id: gamepad_id, event: gilrs::EventType::Disconnected, .. } => {
                    log::info!("Controller disconnected : {0}.", gilrs.gamepad(gamepad_id).name());
                    self.devices_map.remove(&DeviceKey::Gamepad { gamepad_id });
                    None
                },
                _ => None,
            }
        }
        else {
            None
        }
    }
}
