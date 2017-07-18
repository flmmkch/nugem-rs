use super::BitmapPixel;

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
    fn zero_init(&mut self) {
        // Clear is supposed to have no effect on the capacity for the vector
        self.pixels.clear();
        // Fill with black pixels
        for _ in 0..(self.w * self.h) {
            self.pixels.push(BitmapPixel::new(255, 0, 0, 0));
        }
    }
    fn width(&self) -> u32 {
        self.w as u32
    }
    fn height(&self) -> u32 {
        self.h as u32
    }
    fn pixels(&self) -> &[BitmapPixel] {
        &self.pixels
    }
    fn pixels_mut(&mut self) -> &mut [BitmapPixel] {
        &mut self.pixels
    }
    //pub fn redimension(&mut self, width: u32, height: u32) {}
}
