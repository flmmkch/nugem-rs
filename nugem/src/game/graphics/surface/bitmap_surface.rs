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

#[derive(Default, Debug)]
pub struct BitmapSurfaceRenderer {
    position: usize,
    surface: Option<BitmapSurface>,
}

impl BitmapSurfaceRenderer {
    pub fn take(&mut self) -> Option<BitmapSurface> {
        self.position = 0;
        self.surface.take()
    }
}

impl io::Seek for BitmapSurfaceRenderer {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        use std::io::SeekFrom;
        self.position = match pos {
            SeekFrom::Start(offset) => offset as usize,
            SeekFrom::End(offset) => self.surface.as_ref().unwrap().pixels().len() + offset as usize,
            SeekFrom::Current(offset) => self.position + offset as usize,
        };
        Ok(self.position as u64)
    }
}

impl BitmapRenderer for BitmapSurfaceRenderer {
    type Error = RendererError;
    fn initialize_surface(&mut self, width: u64, height: u64) -> Result<(), Self::Error> {
        self.surface.replace(BitmapSurface::new(width as u32, height as u32));
        self.position = 0;
        Ok(())
    }
    fn render_pixels(&mut self, pixel: BitmapPixel, count: u64) -> Result<(), Self::Error> {
        let pixels_mut = self.surface.as_mut().unwrap().pixels_mut();
        pixels_mut[self.position..(self.position + count as usize)].fill(pixel);
        self.position += count as usize;
        Ok(())
    }
    fn surface_pixel_count(&self) -> Result<u64, Self::Error> {
        let pixel_count = self.surface.as_ref().unwrap().pixels().len();
        Ok(pixel_count as u64)
    }
    fn render_single_pixel(&mut self, pixel: BitmapPixel) -> Result<(), Self::Error> {
        let pixels_mut = self.surface.as_mut().unwrap().pixels_mut();
        pixels_mut[self.position] = pixel;
        self.position += 1;
        Ok(())
    }
    fn get_pixel(&self, pixel_index: u64) -> Result<Option<BitmapPixel>, Self::Error> {
        let pixel_opt = self.surface.as_ref().and_then(|s| s.pixels().get(pixel_index as usize)).cloned();
        Ok(pixel_opt)
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
