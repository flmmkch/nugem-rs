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
        use Directional::*;
        match *self {
            Up => 7,
            UpForward => 8,
            Forward => 5,
            DownForward => 2,
            Down => 1,
            DownBackward => 0,
            Backward => 3,
            UpBackward => 6,
            Neutral => 4,
        }
    }
    pub fn from_number(number: u8) -> Option<Directional> {
        use Directional::*;
        match number {
            7 => Some(Up),
            8 => Some(UpForward),
            5 => Some(Forward),
            2 => Some(DownForward),
            1 => Some(Down),
            0 => Some(DownBackward),
            3 => Some(Backward),
            6 => Some(UpBackward),
            4 => Some(Neutral),
            _ => None,
        }
    }
    pub fn test_input(&self, tested_input: Directional, strict: bool) -> bool {
        use Directional::*;
        if strict {
            *self == tested_input
        }
        else {
            match *self {
                Up => match tested_input { Up | UpBackward | UpForward => true, _ => false },
                Forward => match tested_input { Forward | UpForward | UpBackward => true, _ => false },
                Down => match tested_input { Down | DownBackward | DownForward => true, _ => false },
                Backward => match tested_input { Backward | UpBackward | DownBackward => true, _ => false },
                _ => *self == tested_input,
            }
        }
    }
    pub fn motion(&self, motion: DirectionalMotion) -> Self {
        use DirectionalMotion::*;
        use Directional::*;
        match motion {
            FullDirection(d) => d,
            Vertical(DirectionState::Neutral) => match self { Up | Down => Neutral, UpForward | DownForward => Forward, UpBackward | DownBackward => Backward, _ => *self },
            Vertical(DirectionState::Minus) => match self { Neutral | Up => Down, Forward | UpForward => DownForward, Backward | UpBackward => DownBackward, _ => *self },
            Vertical(DirectionState::Plus) => match self { Neutral | Down => Up, Forward | DownForward => UpForward, Backward | DownBackward => UpBackward, _ => *self },
            Horizontal(DirectionState::Neutral) => match self { Forward | Backward => Neutral, UpForward | UpBackward => Up, DownForward | DownBackward => Down, _ => *self },
            Horizontal(DirectionState::Minus) => match self { Neutral | Forward => Backward, Up | UpForward => UpBackward, Down | DownForward => DownBackward, _ => *self },
            Horizontal(DirectionState::Plus) => match self { Neutral | Backward => Forward, Up | UpBackward => UpForward, Down | DownBackward => DownForward, _ => *self },
        }
    }
}

impl Default for Directional {
    fn default() -> Self {
        Directional::Neutral
    }
}

/// Partial directional input.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum DirectionalMotion {
    FullDirection(Directional),
    Vertical(DirectionState),
    Horizontal(DirectionState),
}

impl DirectionalMotion {
    pub fn accept_motion(self, motion: DirectionalMotion) -> DirectionalMotion {
        fn motion_combination(v: DirectionState, h: DirectionState) -> Directional {
            match (v, h) {
                (DirectionState::Plus, DirectionState::Plus) => Directional::UpForward,
                (DirectionState::Plus, DirectionState::Neutral) => Directional::Up,
                (DirectionState::Plus, DirectionState::Minus) => Directional::UpBackward,
                (DirectionState::Neutral, DirectionState::Plus) => Directional::Forward,
                (DirectionState::Neutral, DirectionState::Neutral) => Directional::Neutral,
                (DirectionState::Neutral, DirectionState::Minus) => Directional::Backward,
                (DirectionState::Minus, DirectionState::Plus) => Directional::DownForward,
                (DirectionState::Minus, DirectionState::Neutral) => Directional::Down,
                (DirectionState::Minus, DirectionState::Minus) => Directional::DownBackward,
            }
        }
        match motion {
            DirectionalMotion::FullDirection(d) => DirectionalMotion::FullDirection(d),
            DirectionalMotion::Vertical(v) => match self {
                DirectionalMotion::FullDirection(d) => DirectionalMotion::FullDirection(d.motion(motion)),
                DirectionalMotion::Horizontal(h) => DirectionalMotion::FullDirection(motion_combination(v, h)),
                DirectionalMotion::Vertical(_) => DirectionalMotion::Vertical(v),
            },
            DirectionalMotion::Horizontal(h) => match self {
                DirectionalMotion::FullDirection(d) => DirectionalMotion::FullDirection(d.motion(motion)),
                DirectionalMotion::Horizontal(_) => DirectionalMotion::Horizontal(h),
                DirectionalMotion::Vertical(v) => DirectionalMotion::FullDirection(motion_combination(v, h)),
            },
        }
    }
}

/// State of a given direction: horizontal or vertical.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum DirectionState {
    Minus,
    Neutral,
    Plus,
}

impl DirectionState {
    pub fn number(&self) -> u8 {
        match *self {
            DirectionState::Minus => 0,
            DirectionState::Neutral => 1,
            DirectionState::Plus => 2,
        }
    }
}

