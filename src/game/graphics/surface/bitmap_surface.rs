use super::BitmapPixel;

#[derive(Debug)]
pub struct BitmapSurface {
    w: usize,
    h: usize,
    pixels: Vec<BitmapPixel>,
}

impl BitmapSurface {
    pub fn new(width: u32, height: u32) -> BitmapSurface {
        let w = width as usize;
        let h = height as usize;
        let pixels: Vec<BitmapPixel> = Vec::with_capacity(w * h);
        let mut bitmap_surface = BitmapSurface {
            w,
            h,
            pixels,
        };
        bitmap_surface.zero_init();
        bitmap_surface
    }
    pub fn zero_init(&mut self) {
        // Clear is supposed to have no effect on the capacity for the vector: it stays at w * h
        self.pixels.clear();
        // Fill with transparent pixels
        for _ in 0..(self.w * self.h) {
            self.pixels.push(BitmapPixel::new(0, 0, 0, 0));
        }
    }
    pub fn width(&self) -> u32 {
        self.w as u32
    }
    pub fn height(&self) -> u32 {
        self.h as u32
    }
    pub fn pixels(&self) -> &[BitmapPixel] {
        &self.pixels
    }
    pub fn pixels_mut(&mut self) -> &mut [BitmapPixel] {
        &mut self.pixels
    }
}
