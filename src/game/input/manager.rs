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
        let initial_state = State {
            directional: Directional::Neutral,
            a: ButtonState::Up,
            b: ButtonState::Up,
            c: ButtonState::Up,
            x: ButtonState::Up,
            y: ButtonState::Up,
            z: ButtonState::Up,
            start: ButtonState::Up,
            back: ButtonState::Up,
        };
        let device = Device::new(DeviceType::GameController, name, initial_state, DeviceInternal::GameController(sdl_game_controller));
        self.add_device(device, sdl_id);
    }
    fn add_device(&mut self, device: Device, sdl_id: usize) {
        let device_id = self.devices.len();
        self.device_sdl_map.insert(DeviceKey::new(device.device_type(), sdl_id), device_id);
        self.devices.push(device);
    }
    pub fn process_sdl_event(&self, sdl_event: sdl2::event::Event) -> Option<event::Event> {
        match sdl_event {
            sdl2::event::Event::ControllerAxisMotion {
                timestamp,
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
                                _ => None
                            }
                        },
                        sdl2::controller::Axis::LeftY => {
                            match (positive, negative) {
                                (true, false) => Some(PartialDirectional::Vertical(DirectionState::Minus)),
                                (false, true) => Some(PartialDirectional::Vertical(DirectionState::Plus)),
                                _ => None
                            }
                        },
                        _ => None,
                    }
                };
                if let Some(direction_state) = direction_state_option {
                    let mut partial_state = PartialState::new();
                    partial_state.directional = Some(direction_state);
                    let event = event::Event {
                        device_id: *self.device_sdl_map.get(&DeviceKey::new(DeviceType::GameController, which as usize)).unwrap(),
                        timestamp,
                        partial_state: partial_state,
                    };
                    Some(event)
                }
                else {
                    None
                }
            },
            sdl2::event::Event::ControllerButtonDown {
                ..
            } => None, // TODO
            sdl2::event::Event::ControllerButtonUp {
                ..
            } => None, // TODO
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
