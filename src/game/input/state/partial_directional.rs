use super::{AcceptInputDirectional, Directional, DirectionState};

/// Partial directional input.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum PartialDirectional {
    FullDirection(Directional),
    Vertical(DirectionState),
    Horizontal(DirectionState),
}

impl AcceptInputDirectional for PartialDirectional {
    fn accept(&mut self, partial_directional: PartialDirectional) {
        match partial_directional {
            PartialDirectional::FullDirection(_) => *self = partial_directional,
            PartialDirectional::Horizontal(input_h) => {
                match *self {
                    PartialDirectional::FullDirection(ref mut directional) => directional.accept(partial_directional),
                    PartialDirectional::Horizontal(h) => *self = PartialDirectional::Horizontal(h),
                    PartialDirectional::Vertical(v) => *self = PartialDirectional::FullDirection(Directional::new(input_h, v)),
                }
            },
            PartialDirectional::Vertical(input_v) => {
                match *self {
                    PartialDirectional::FullDirection(ref mut directional) => directional.accept(partial_directional),
                    PartialDirectional::Horizontal(h) => *self = PartialDirectional::FullDirection(Directional::new(h, input_v)),
                    PartialDirectional::Vertical(v) => *self = PartialDirectional::Vertical(v),
                }
            }
        }
    }
}
