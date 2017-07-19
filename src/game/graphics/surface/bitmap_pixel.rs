pub struct BitmapPixel {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl BitmapPixel {
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> BitmapPixel {
        BitmapPixel {
            a,
            r,
            g,
            b,
        }
    }
}
