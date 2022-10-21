use std::{fmt, io};
use crate::bitmap::BitmapPixel;

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Rgb(u8, u8,u8),
    Transparent,
}

impl From<Color> for BitmapPixel {
    fn from(value: Color) -> BitmapPixel {
        match value {
            Color::Transparent => BitmapPixel::new(0, 0, 0, 0),
            Color::Rgb(r,g,b) => BitmapPixel::new(r, g, b, u8::max_value()),
        }
    }
}

pub const PALETTE_COLOR_COUNT : usize = 256;

pub struct Palette {
    pub colors: [Color; PALETTE_COLOR_COUNT],
}

impl Palette {
    pub fn read<T: io::Read>(mut reader: T) -> Result<Palette, io::Error> {
        let mut colors_buf = [0; PALETTE_COLOR_COUNT * 3];
        reader.read(&mut colors_buf)?;
        let mut palette = Palette {
            colors: [Color::Transparent; 256]
        };
        for i in 0..PALETTE_COLOR_COUNT {
            let red = colors_buf[i * 3];
            let green = colors_buf[i * 3 + 1];
            let blue = colors_buf[i * 3 + 2];
            palette.colors[PALETTE_COLOR_COUNT - 1 - i] = Color::Rgb(red, green, blue);
        }
        Ok(palette)
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
