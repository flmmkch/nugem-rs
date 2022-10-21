use super::*;
use std::collections::VecDeque;
use ::game::input;
use sdl2;

pub struct EventQueue {
    queue: VecDeque<Event>,
    event_pump: sdl2::EventPump,
}

impl EventQueue {
    pub fn new(event_pump: sdl2::EventPump) -> EventQueue {
        EventQueue {
            queue: VecDeque::new(),
            event_pump,
        }
    }
    pub fn process(&mut self, input_manager: &mut input::Manager) {
        let mut new_events = Vec::new();
        for sdl_event in self.event_pump.poll_iter() {
            match sdl_event {
                sdl2::event::Event::Quit {..} => new_events.push(Event::Quit),
                sdl2::event::Event::ControllerAxisMotion {..}
                | sdl2::event::Event::ControllerButtonDown {..}
                | sdl2::event::Event::ControllerButtonUp {..} => {
                    match input_manager.process_sdl_event(sdl_event) {
                        Some(input_event) => new_events.push(Event::Input(input_event)),
                        None => (),
                    }
                },
                _ => (),
            }
        }
        for new_event in new_events {
            self.push(new_event);
        }
    }
    pub fn push(&mut self, e: Event) {
        self.queue.push_back(e);
    }
    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<Event> {
        self.queue.pop_front()
    }
    #[allow(dead_code)]
    pub fn peek(&self) -> Option<&Event> {
        self.queue.front()
    }
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}