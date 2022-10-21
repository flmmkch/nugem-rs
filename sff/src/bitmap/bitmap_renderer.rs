use std::io::Seek;
use std::io;
use std::num::IntErrorKind;

use super::BitmapPixel;

pub trait BitmapRenderer: Seek {
    type Error: From<io::Error> + From<IntErrorKind>;
    /// Initializes a drawing surface with a given `width` and `height`
    fn initialize_surface(&mut self, width: u64, height: u64) -> Result<(), Self::Error>;
    /// Renders pixels
    fn render_pixels(&mut self, pixel: BitmapPixel, count: u64) -> Result<(), Self::Error>;
    /// Gets the total number of pixels in the surface
    fn surface_pixel_count(&self) -> Result<u64, Self::Error>;
    /// Renders one pixel
    fn render_single_pixel(&mut self, pixel: BitmapPixel) -> Result<(), Self::Error> {
        self.render_pixels(pixel, 1)
    }
    /// Get the given pixel for an index
    fn get_pixel(&self, pixel_index: u64) -> Result<Option<BitmapPixel>, Self::Error>;
    /// Copy already rendered pixels with a negative offset
    fn copy_pixels_offset(&mut self, count: u64, offset: u64) -> Result<(), Self::Error> {
        let start_pixel_index = self.stream_position()?;
        let copy_source_start_index = start_pixel_index.checked_sub(offset).ok_or(IntErrorKind::NegOverflow)?;
        let copy_source_end_index = copy_source_start_index.checked_add(count).ok_or(IntErrorKind::PosOverflow)?;
        for pixel_to_copy_index in copy_source_start_index..copy_source_end_index {
            if let Some(pixel) = self.get_pixel(pixel_to_copy_index)? {
                self.render_single_pixel(pixel)?;
            }
        }
        Ok(())
    }
}
