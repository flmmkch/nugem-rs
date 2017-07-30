use super::*;
use std::collections::HashMap;
use sdl2::{self, GameControllerSubsystem, JoystickSubsystem};
use std::io::{BufRead, BufReader, Cursor};

#[derive(Debug)]
pub struct Manager {
    sdl_gc: GameControllerSubsystem,
    sdl_joystick: JoystickSubsystem,
    devices: Vec<Device>,
    device_sdl_map: HashMap<DeviceKey, usize>,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct DeviceKey {
    device_type: DeviceType,
    sdl_id: usize,
}

fn controller_button(sdl_button: sdl2::controller::Button) -> Option<Button> {
    match sdl_button {
        sdl2::controller::Button::A => Some(Button::A),
        sdl2::controller::Button::B => Some(Button::B),
        sdl2::controller::Button::LeftShoulder => Some(Button::C),
        sdl2::controller::Button::X => Some(Button::X),
        sdl2::controller::Button::Y => Some(Button::Y),
        sdl2::controller::Button::RightShoulder => Some(Button::Z),
        sdl2::controller::Button::Start => Some(Button::Start),
        sdl2::controller::Button::Back => Some(Button::Back),
        _ => None,
    }
}

pub const SDL_AXIS_THRESHOLD : i16 = 32767 / 3;

impl Manager {
    pub fn new(sdl: &sdl2::Sdl) -> Manager {
        let mut manager = Manager {
            sdl_gc: sdl.game_controller().unwrap(),
            sdl_joystick: sdl.joystick().unwrap(),
            devices: Vec::new(),
            device_sdl_map: HashMap::new(),
        };
        manager.initialize();
        manager
    }
    fn initialize(&mut self) {
        self.sdl_joystick.set_event_state(true);
        self.sdl_gc.set_event_state(true);
        // load controller mappings
        // for now the mappings are determined at compile time
        {
            let buf_read = BufReader::new(Cursor::new(include_str!("gamecontrollerdb.txt")));
            for line_res in buf_read.lines() {
                if let Ok(line) = line_res {
                    if let Ok(_) = self.sdl_gc.add_mapping(line.as_str()) {
                        ()
                    }
                }
            }
        }
        let num_joysticks = self.sdl_gc.num_joysticks().unwrap_or(0);
        for i in 0..num_joysticks {
            if self.sdl_gc.is_game_controller(i) {
                match self.sdl_gc.open(i) {
                    Ok(game_controller) => self.add_game_controller(game_controller, i as usize),
                    Err(e) => error!("Unable to load game controller {}: {}", i, e),
                }
            }
        }
    }
    fn add_game_controller(&mut self, sdl_game_controller: sdl2::controller::GameController, sdl_id: usize) {
        let name = sdl_game_controller.name();
        // TODO: read initial state better
        let initial_state = State::new();
        let device = Device::new(DeviceType::GameController, name, initial_state, DeviceInternal::GameController(sdl_game_controller));
        self.add_device(device, sdl_id);
    }
    fn add_device(&mut self, device: Device, sdl_id: usize) {
        let device_id = self.devices.len();
        self.device_sdl_map.insert(DeviceKey::new(device.device_type(), sdl_id), device_id);
        self.devices.push(device);
    }
    fn sdl_device_id(&self, sdl_which: i32) -> Option<usize> {
        if let Some(device_id) = self.device_sdl_map.get(&DeviceKey::new(DeviceType::GameController, sdl_which as usize)) {
            Some(*device_id)
        }
        else {
            None
        }
    }
    pub fn process_sdl_event(&mut self, sdl_event: sdl2::event::Event) -> Option<event::Event> {
        macro_rules! process_input_event {
            ($input_option: expr, $input_ident: ident, $which: expr) => (
                if let Some(input) = $input_option {
                    let device_id = self.sdl_device_id($which).unwrap();
                    let processed_partial_state_opt = {
                        let mut partial_state = PartialState::new();
                        partial_state.$input_ident = Some(input);
                        self.devices[device_id].process_state(partial_state)
                    };
                    if let Some(partial_state) = processed_partial_state_opt {
                        let event = event::Event {
                            device_id,
                            partial_state,
                        };
                        Some(event)
                    }
                    else {
                        None
                    }
                }
                else {
                    None
                }
            )
        }
        macro_rules! process_input_button {
            ($input_state_option: expr, $input_button: expr, $which: expr) => (
                match $input_button {
                    Button::A => process_input_event!($input_state_option, a, $which),
                    Button::B => process_input_event!($input_state_option, b, $which),
                    Button::C => process_input_event!($input_state_option, c, $which),
                    Button::X => process_input_event!($input_state_option, x, $which),
                    Button::Y => process_input_event!($input_state_option, y, $which),
                    Button::Z => process_input_event!($input_state_option, z, $which),
                    Button::Start => process_input_event!($input_state_option, start, $which),
                    Button::Back => process_input_event!($input_state_option, back, $which),
                }
            )
        }
        match sdl_event {
            sdl2::event::Event::ControllerAxisMotion {
                timestamp: _,
                which,
                axis,
                value,
            } => {
                let direction_state_option = {
                    let positive = value > SDL_AXIS_THRESHOLD;
                    let negative = value < -SDL_AXIS_THRESHOLD;
                    match axis {
                        sdl2::controller::Axis::LeftX => {
                            match (positive, negative) {
                                (true, false) => Some(PartialDirectional::Horizontal(DirectionState::Plus)),
                                (false, true) => Some(PartialDirectional::Horizontal(DirectionState::Minus)),
                                (false, false) => Some(PartialDirectional::Horizontal(DirectionState::Neutral)),
                                _ => None
                            }
                        },
                        sdl2::controller::Axis::LeftY => {
                            match (positive, negative) {
                                (true, false) => Some(PartialDirectional::Vertical(DirectionState::Minus)),
                                (false, true) => Some(PartialDirectional::Vertical(DirectionState::Plus)),
                                (false, false) => Some(PartialDirectional::Vertical(DirectionState::Neutral)),
                                _ => None
                            }
                        },
                        _ => None,
                    }
                };
                process_input_event!(direction_state_option, directional, which)
            },
            sdl2::event::Event::ControllerButtonDown {
                timestamp: _,
                which,
                button,
            } => {
                if let Some(input_button) = controller_button(button) {
                    process_input_button!(Some(ButtonState::Down), input_button, which)
                }
                else {
                    None
                }
            },
            sdl2::event::Event::ControllerButtonUp {
                timestamp: _,
                which,
                button,
            } => {
                if let Some(input_button) = controller_button(button) {
                    process_input_button!(Some(ButtonState::Up), input_button, which)
                }
                else {
                    None
                }
            },
            _ => None,
        }
    }
}

impl DeviceKey {
    fn new(device_type: DeviceType, sdl_id: usize) -> DeviceKey {
        DeviceKey {
            device_type,
            sdl_id,
        }
    }
}
