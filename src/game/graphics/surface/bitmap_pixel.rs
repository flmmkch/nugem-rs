#[derive(Clone)]
#[derive(Debug)]
#[derive(Copy)]
pub struct BitmapPixel(u8, u8, u8, u8);

impl BitmapPixel {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> BitmapPixel {
        BitmapPixel(
            r,
            g,
            b,
            a,
        )
    }
    pub fn r(&self) -> u8 {
        self.0
    }
    pub fn g(&self) -> u8 {
        self.1
    }
    pub fn b(&self) -> u8 {
        self.2
    }
    pub fn a(&self) -> u8 {
        self.3
    }
}
