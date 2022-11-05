use super::scene::Scene;

pub enum Event {
    Quit,
    NextScene(Box<dyn Scene>),
}

pub type EventLoopSender = winit::event_loop::EventLoopProxy<Event>;

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Event::*;

        match &self {
            Quit => f.write_str(stringify!(Event::Quit)),
            NextScene(_) => f.write_str(stringify!(Event::NextScene)),
        }
    }
}
