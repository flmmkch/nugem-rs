use super::{AcceptInputDirectional, DirectionState, PartialDirectional};

/// Directional input.
///
/// Numerical map of directions:
///
/// 6 7 8
///
/// 3 4 5
///
/// 0 1 2
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Directional {
    Up,
    UpForward,
    Forward, 
    DownForward,
    Down,
    DownBackward,
    Backward,
    UpBackward,
    Neutral,
}

impl Directional {
    pub fn new(horizontal: DirectionState, vertical: DirectionState) -> Directional {
        let horizontal_number = 1 + horizontal.number();
        let vertical_number = 3 * (1 + vertical.number());
        Directional::from_number(horizontal_number + vertical_number).unwrap()
    }
    pub fn number(&self) -> u8 {
        match *self {
            Directional::Up => 7,
            Directional::UpForward => 8,
            Directional::Forward => 5,
            Directional::DownForward => 2,
            Directional::Down => 1,
            Directional::DownBackward => 0,
            Directional::Backward => 3,
            Directional::UpBackward => 6,
            Directional::Neutral => 4,
        }
    }
    pub fn from_number(number: u8) -> Option<Directional> {
        match number {
            7 => Some(Directional::Up),
            8 => Some(Directional::UpForward),
            5 => Some(Directional::Forward),
            2 => Some(Directional::DownForward),
            1 => Some(Directional::Down),
            0 => Some(Directional::DownBackward),
            3 => Some(Directional::Backward),
            6 => Some(Directional::UpBackward),
            4 => Some(Directional::Neutral),
            _ => None,
        }
    }
}

impl AcceptInputDirectional for Directional {
    fn accept(&mut self, partial_directional: PartialDirectional) {
        match partial_directional {
            PartialDirectional::FullDirection(direction) => *self = direction,
            PartialDirectional::Horizontal(direction_state) => {
                let vertical_number = self.number() - (self.number() % 3);
                let horizontal_number = direction_state.number();
                *self = Directional::from_number(vertical_number + horizontal_number).unwrap();
            },
            PartialDirectional::Vertical(direction_state) => {
                let vertical_number = 3 * direction_state.number();
                let horizontal_number = self.number() % 3;
                *self = Directional::from_number(vertical_number + horizontal_number).unwrap();
            },
        }
    }
}
