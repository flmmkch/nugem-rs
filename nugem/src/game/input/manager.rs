use super::*;
use winit::event::{DeviceId, DeviceEvent, ElementState, KeyboardInput};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Manager {
    devices_map: HashMap<DeviceKey, Device>,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct DeviceKey {
    device_type: DeviceType,
    device_id: DeviceId,
}

impl From<ElementState> for ButtonState {
    fn from(value: ElementState) -> Self {
        match value {
            ElementState::Pressed => ButtonState::Down,
            ElementState::Released => ButtonState::Up,
        }
    }
}

fn controller_button(winit_button: u32) -> Option<Button> {
    // xbox 360 mappings
    // TODO use loaded controller mappings or a library for that
    match winit_button {
        0 => Some(Button::A),
        1 => Some(Button::B),
        2 => Some(Button::X),
        3 => Some(Button::Y),
        4 => Some(Button::C), // left shoulder
        5 => Some(Button::Z), // right shoulder
        6 => Some(Button::Back),
        7 => Some(Button::Start),
        _ => None,
    }
}

pub const MOTION_AXIS_THRESHOLD : f64 = <f64>::MAX / 3f64;

macro_rules! process_controller_event {
    ($input: expr, $input_ident: ident, $device_id: expr, $devices_map:expr) => {{
        let device_key = DeviceKey { device_type: DeviceType::GameController, device_id: $device_id };
        let device = $devices_map.entry(device_key).or_insert_with(|| super::Device::new(DeviceType::GameController, "".into(), State::new()));
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
        let manager = Self {
            devices_map: Default::default(),
        };
        // load controller mappings
        // TODO
        manager
    }
    pub fn process_keyboard_input_event(&mut self, device_id: DeviceId, keyboard_input: KeyboardInput) -> Option<event::Event> {
        use winit::event::VirtualKeyCode;
        use DirectionalMotion::*;
        use DirectionState::*;
        let KeyboardInput { scancode: _, virtual_keycode, state, .. } = keyboard_input;

        macro_rules! process_keyboard_event {
            ($input: expr, $input_ident: ident, $device_id: expr, $keyboard_id: expr) => {{
                let device_key = DeviceKey { device_type: DeviceType::Keyboard($keyboard_id), device_id };
                let device = self.devices_map.entry(device_key).or_insert_with(|| super::Device::new(DeviceType::Keyboard($keyboard_id), "".into(), State::new()));
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
    pub fn process_controller_axis_event(&mut self, device_id: DeviceId, axis: u32, value: f64) -> Option<event::Event> {
        let direction_motion_opt = {
            use DirectionalMotion::*;
            use DirectionState::*;
            let positive = value > MOTION_AXIS_THRESHOLD;
            let negative = value < -MOTION_AXIS_THRESHOLD;
            match axis {
                0 => {
                    // horizontal
                    match (positive, negative) {
                        (true, false) => Some(Horizontal(Plus)),
                        (false, true) => Some(Horizontal(Minus)),
                        (false, false) => Some(Horizontal(Neutral)),
                        _ => None
                    }
                },
                1 => {
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
            process_controller_event!(direction_motion, directional, device_id, &mut self.devices_map)
        }
        else {
            None
        }
    }
    pub fn process_device_event(&mut self, device_id: DeviceId, device_event: DeviceEvent) -> Option<event::Event> {
        macro_rules! process_controller_button {
            ($input_state_option: expr, $input_button: expr, $which: expr) => (
                match $input_button {
                    Button::A => process_controller_event!($input_state_option, a, $which, &mut self.devices_map),
                    Button::B => process_controller_event!($input_state_option, b, $which, &mut self.devices_map),
                    Button::C => process_controller_event!($input_state_option, c, $which, &mut self.devices_map),
                    Button::X => process_controller_event!($input_state_option, x, $which, &mut self.devices_map),
                    Button::Y => process_controller_event!($input_state_option, y, $which, &mut self.devices_map),
                    Button::Z => process_controller_event!($input_state_option, z, $which, &mut self.devices_map),
                    Button::Start => process_controller_event!($input_state_option, start, $which, &mut self.devices_map),
                    Button::Back => process_controller_event!($input_state_option, back, $which, &mut self.devices_map),
                }
            )
        }
        match device_event {
            DeviceEvent::Added => {
                // TODO
                None
            },
            DeviceEvent::Removed => {
                // TODO
                None
            },
            DeviceEvent::Button { button, state } => {
                let button_state: ButtonState = state.into();

                if let Some(input_button) = controller_button(button) {
                    process_controller_button!(button_state, input_button, device_id)
                }
                else {
                    None
                }
            },
            DeviceEvent::Motion { axis, value } => self.process_controller_axis_event(device_id, axis, value),
            DeviceEvent::Text { codepoint } => {
                log::debug!("Input event of type text with codepoint {codepoint}");
                None
            },
            DeviceEvent::Key(keyboard_input) => self.process_keyboard_input_event(device_id, keyboard_input),
            _ => None,
        }
    }
}
