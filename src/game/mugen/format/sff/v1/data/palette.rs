use ::game::graphics::surface::BitmapPixel;
use std::fmt;

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Rgb(u8, u8,u8),
    Transparent,
}

pub const PALETTE_COLOR_COUNT : usize = 256;

pub struct Palette {
    pub colors: [Color; PALETTE_COLOR_COUNT],
}

impl Into<BitmapPixel> for Color {
    fn into(self) -> BitmapPixel {
        match self {
            Color::Transparent => BitmapPixel::new(0, 0, 0, 0),
            Color::Rgb(r,g,b) => BitmapPixel::new(r, g, b, u8::max_value()),
        }
    }
}

impl Clone for Palette {
    fn clone(&self) -> Palette {
        Palette {
            colors: self.colors,
        }
    }
}

impl fmt::Debug for Palette {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Palette {{...}}")
    }
}
