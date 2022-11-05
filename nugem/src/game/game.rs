use super::Config;
use crate::game::{ events, graphics, input, scene };
use winit::window::{Window, WindowBuilder};
use std::sync::RwLock;

use winit::{
    event::WindowEvent,
    event_loop::{ EventLoop, EventLoopBuilder },
};

const DEFAULT_WINDOW_TITLE: &'static str = "Nugem";

pub struct Game {
    config: Config,
    input_manager: input::Manager,
    event_loop: EventLoop<events::Event>,
    #[allow(dead_code)]
    window: Window,
    graphics_state: graphics::State,
    current_scene: RwLock<Box<dyn scene::Scene>>,
}

impl Game {
    pub async fn new(config: Config) -> Game {
        use super::scene::Scene;

        let input_manager = input::Manager::initialize();

        // initialize event loop, gpu and window
        let event_loop: EventLoop<events::Event> = EventLoopBuilder::with_user_event().build();
        let fullscreen = if config.fullscreen() { Some(winit::window::Fullscreen::Borderless(None)) } else { None };
        let window = WindowBuilder::new()
            .with_title(DEFAULT_WINDOW_TITLE)
            .with_fullscreen(fullscreen)
            .build(&event_loop).expect("Failed to open window");
        let graphics_state = graphics::State::new(&window).await.expect("Failed to initialize GPU");

        // initialize scenes
        let next_scene = Box::new(scene::fight::Fight::new(&config));
        let mut loading_scene = Box::new(scene::Loading::new(next_scene));
        loading_scene.load(&graphics_state, &config).unwrap();
        let current_scene: RwLock<Box<dyn scene::Scene>> = RwLock::new(loading_scene);

        Game {
            config,
            input_manager,
            event_loop,
            window,
            graphics_state,
            current_scene,
        }
    }
    
    pub async fn run(mut self) {
        let event_loop_proxy = self.event_loop.create_proxy();

        self.event_loop.run(move |event, _event_loop_window_target, control_flow| {
            use winit::event::Event as WinitEv;
            use events::Event as NugemEvent;

            // ControlFlow::Wait pauses the event loop if no events are available to process.
            // This is ideal for non-game applications that only update in response to user
            // input, and uses significantly less power/CPU time than ControlFlow::Poll.
            // control_flow.set_wait();

            // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
            // dispatched any events. This is ideal for games and similar applications.
            control_flow.set_poll();

            fn process_input_event(input_manager_result: Option<input::event::Event>, current_scene: &mut Box<dyn scene::Scene>, event_loop_sender: events::EventLoopSender, control_flow: &mut winit::event_loop::ControlFlow) {
                if let Some(input_event) = input_manager_result {
                    if let Some(global_event) = current_scene.input_event(input_event) {
                        if let Err(event_loop_closed) = event_loop_sender.send_event(global_event) {
                            log::error!("Event loop closed before input, quitting... {event_loop_closed}");
                            control_flow.set_exit();
                        }
                    }
                }
            }


            match event {
                WinitEv::UserEvent(NugemEvent::Quit) | WinitEv::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    control_flow.set_exit();
                },
                WinitEv::UserEvent(NugemEvent::NextScene(next_scene)) => {
                    let current_scene_mut = self.current_scene.get_mut().unwrap();
                    *current_scene_mut = next_scene;
                },
                WinitEv::WindowEvent { event: winit::event::WindowEvent::Resized(new_size), .. } => self.graphics_state.resize(new_size.clone()),
                WinitEv::WindowEvent { event: winit::event::WindowEvent::ScaleFactorChanged{ new_inner_size, scale_factor: _scale_factor }, .. } => self.graphics_state.resize(new_inner_size.clone()),
                // input handling
                // keyboard input on window
                WinitEv::WindowEvent { event: winit::event::WindowEvent::KeyboardInput { device_id, input, .. }, .. } => process_input_event(self.input_manager.process_keyboard_input_event(device_id, input), self.current_scene.get_mut().unwrap(), event_loop_proxy.clone(), control_flow),
                // controller axis motion on window
                WinitEv::WindowEvent { event: winit::event::WindowEvent::AxisMotion { device_id, axis, value, .. }, .. } => process_input_event(self.input_manager.process_controller_axis_event(device_id, axis, value), self.current_scene.get_mut().unwrap(), event_loop_proxy.clone(), control_flow),
                // device event
                WinitEv::DeviceEvent { device_id, event: device_event } => process_input_event(self.input_manager.process_device_event(device_id, device_event), self.current_scene.get_mut().unwrap(), event_loop_proxy.clone(), control_flow),
                WinitEv::MainEventsCleared => {
                    // Application update code.

                    // Queue a RedrawRequested event.
                    //
                    // You only need to call this if you've determined that you need to redraw, in
                    // applications which do not always need to. Applications that redraw continuously
                    // can just render here instead.
                    let current_scene_mut = self.current_scene.get_mut().unwrap();

                    let continue_main = current_scene_mut.update(&self.graphics_state, &self.config, event_loop_proxy.clone());
                    if !continue_main {
                        control_flow.set_exit()
                    }
                    else {
                        current_scene_mut.display(&self.graphics_state);
                    }
                },
                WinitEv::RedrawRequested(_) => {
                    // Redraw the application.
                    //
                    // It's preferable for applications that do not render continuously to render in
                    // this event rather than in MainEventsCleared, since rendering in here allows
                    // the program to gracefully handle redraws requested by the OS.
                },
                _ => ()
            }
        })
    }
}
