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
    North,
    NorthEast,
    East, 
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
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
            Directional::North => 7,
            Directional::NorthEast => 8,
            Directional::East => 5,
            Directional::SouthEast => 2,
            Directional::South => 1,
            Directional::SouthWest => 0,
            Directional::West => 3,
            Directional::NorthWest => 6,
            Directional::Neutral => 4,
        }
    }
    pub fn from_number(number: u8) -> Option<Directional> {
        match number {
            7 => Some(Directional::North),
            8 => Some(Directional::NorthEast),
            5 => Some(Directional::East),
            2 => Some(Directional::SouthEast),
            1 => Some(Directional::South),
            0 => Some(Directional::SouthWest),
            3 => Some(Directional::West),
            6 => Some(Directional::NorthWest),
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
