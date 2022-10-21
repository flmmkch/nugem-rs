use ::game::input;

#[derive(Debug)]
pub enum Event {
    Quit,
    Input(input::event::Event),
}
