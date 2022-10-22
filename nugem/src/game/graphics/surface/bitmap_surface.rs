use std::io;
use std::num::IntErrorKind;
use nugem_sff::bitmap::{BitmapPixel, BitmapRenderer};

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct BitmapSurfaceRenderer {
    cursor: usize,
    surface: BitmapSurface,
}

impl BitmapSurfaceRenderer {
    pub fn take(self) -> BitmapSurface {
        self.surface
    }
}

impl BitmapRenderer for BitmapSurfaceRenderer {
    type Error = RendererError;
    type Initializer = ();
    fn initialize_surface(_initializer: Self::Initializer, width: u64, height: u64) -> Result<Self, Self::Error> {
        let surface = BitmapSurface::new(width as u32, height as u32);
        Ok(BitmapSurfaceRenderer { cursor: 0, surface })
    }
    fn render_pixels(&mut self, pixel: BitmapPixel, count: u64) -> Result<(), Self::Error> {
        let pixels_mut = self.surface.pixels_mut();
        pixels_mut[self.cursor..(self.cursor + count as usize)].fill(pixel);
        self.cursor += count as usize;
        Ok(())
    }
    fn surface_pixel_count(&mut self) -> Result<u64, Self::Error> {
        let pixel_count = self.surface.pixels().len();
        Ok(pixel_count as u64)
    }
    fn render_single_pixel(&mut self, pixel: BitmapPixel) -> Result<(), Self::Error> {
        let pixels_mut = self.surface.pixels_mut();
        pixels_mut[self.cursor] = pixel;
        self.cursor += 1;
        Ok(())
    }
    fn get_pixel(&mut self, pixel_index: u64) -> Result<Option<BitmapPixel>, Self::Error> {
        let pixel_opt = self.surface.pixels().get(pixel_index as usize).cloned();
        Ok(pixel_opt)
    }
    fn copy_pixels_offset(&mut self, count: u64, offset: u64) -> Result<(), Self::Error> {
        let copy_start_index = self.cursor.saturating_sub(offset as usize);
        self.surface.pixels_mut().copy_within(copy_start_index..(copy_start_index + count as usize), self.cursor);
        self.cursor += count as usize;
        Ok(())
    }
}

impl io::Seek for BitmapSurfaceRenderer {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let new_cursor = match pos {
            io::SeekFrom::Start(i) => i as usize,
            io::SeekFrom::End(i) => (self.surface.pixels.len() as i64 + i) as usize,
            io::SeekFrom::Current(i) => (self.cursor as i64 + i) as usize,
        };
        self.cursor = new_cursor;
        Ok(new_cursor as u64)
    }
    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.cursor as u64)
    }
}


#[derive(Debug)]
pub enum RendererError {
    UninitializedSurface,
    IoError(io::Error),
    NumKindError(IntErrorKind),
}

impl From<io::Error> for RendererError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<IntErrorKind> for RendererError {
    fn from(err: IntErrorKind) -> Self {
        Self::NumKindError(err)
    }
}
