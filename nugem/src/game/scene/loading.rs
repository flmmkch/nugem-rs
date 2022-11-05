use super::Scene;
use crate::game::events;

pub struct Loading {
    next_scene: Option<Box<dyn Scene>>,
}

impl Loading {
    pub fn new(next_scene: Box<dyn Scene>) -> Self {
        let next_scene = Some(next_scene);
        Self {
            next_scene,
        }
    }
}

impl Scene for Loading {
    fn load(&mut self, _: &crate::game::graphics::State, _: &crate::game::Config) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn display(&mut self, _graphics_state: &crate::game::graphics::State) {
        // TODO
    }

    fn input_event(&mut self, _input_event: crate::game::input::event::Event) -> Option<crate::game::events::Event> {
        None
    }

    fn update(&mut self, graphics_state: &crate::game::graphics::State, config: &crate::game::Config, event_loop_sender: events::EventLoopSender) -> bool {
        // go to the next scene when we have finished loading
        if let Some(mut next_scene) = self.next_scene.take() {
            match next_scene.load(graphics_state, config) {
                Ok(()) => event_loop_sender.send_event(events::Event::NextScene(next_scene)).ok().is_some(),
                Err(e) => {
                    log::error!("Error loading next scene: {e}");
                    false
                }
            }
        }
        else {
            false
        }
    }
}

impl Drop for Loading {
    fn drop(&mut self) {
        log::debug!("Finished loading");
    }
}
