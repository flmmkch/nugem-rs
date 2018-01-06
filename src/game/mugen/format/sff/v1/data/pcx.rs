use super::{Palette, Color};
use ::game::graphics::surface::BitmapSurface;
use std::io::{Read, Seek, SeekFrom};
use super::super::Error;
use byteorder::{LittleEndian, ReadBytesExt};

pub fn read_pcx_surface<T: Read + Seek>(mut reader: T, palette: &Palette) -> Result<BitmapSurface, Error> {
    // PCX file format:
    // reading the PCX header
    // byte 0: manufacturer, must be 0x0A
    {
        let manufacturer = reader.read_u8()?;
        if manufacturer != 0x0A {
            return Err(Error::InvalidPcxData(format!("Bad manufacturer byte: 0x{:0X}", manufacturer)));
        }
    }
    // byte 1: PCX Paintbrush version
    {
        let paintbrush = reader.read_u8()?;
        match paintbrush {
            0 |         // version 2.5
            2 |         // version 2.8 with palette
            3 |         // version 2.8 without palette
            5 => (),    // version 3.0
            _ => return Err(Error::InvalidPcxData(format!("Invalid paintbrush: {}", paintbrush))),
        }
    }
    // byte 2: encoding
    let using_rle = {
        let encoding_byte = reader.read_u8()?;
        match encoding_byte {
            0 => false, // uncompressed image
            1 => true, // PCX format run-length encoding
            _ => return Err(Error::InvalidPcxData(format!("Bad encoding byte: {}", encoding_byte))),
        }
    };
    // byte 3: bits per pixel
    // number of bits per pixel in each color plane: 1, 2, 4, 8, 24
    let bits_per_pixel = {
        let bpp_byte = reader.read_u8()?;
        match bpp_byte {
            1 | 2 | 4 | 8 | 24 => bpp_byte as u16,
            _ => return Err(Error::InvalidPcxData(format!("Invalid bits per plane: {}", bpp_byte))),
        }
    };
    // bytes 4 - 10: image size
    // width
    let (width, height) = {
        let x_min = reader.read_u16::<LittleEndian>()?;
        let y_min = reader.read_u16::<LittleEndian>()?;
        let x_max = reader.read_u16::<LittleEndian>()?;
        let y_max = reader.read_u16::<LittleEndian>()?;
        (x_max - x_min + 1, y_max - y_min + 1)
    };
    // skipped data
    // 11 - 12: vertical DPI
    reader.seek(SeekFrom::Current(2))?;
    // 13 - 14: horizontal DPI
    reader.seek(SeekFrom::Current(2))?;
    // 15 - 63: PCX palette (skip the next 48 bytes)
    reader.seek(SeekFrom::Current(48))?;
    // 64: reserved: should be set to 0
    {
        let reserved_byte = reader.read_u8()?;
        match reserved_byte {
            0 => (),
            _ => return Err(Error::InvalidPcxData(format!("Bad reserved byte: 0x{:0X}", reserved_byte))),
        }
    }
    // 65: number of color planes
    let bit_planes = (reader.read_u8()?) as u16;
    // 66 - 67: bytes per plane for a line
    let bytes_per_plane = reader.read_u16::<LittleEndian>()?;
    // 68 - 69: palette type
    reader.seek(SeekFrom::Current(2))?;
    // 70 - 71: horizontal scrolling info
    reader.seek(SeekFrom::Current(2))?;
    // 72 - 73: vertical scrolling info
    reader.seek(SeekFrom::Current(2))?;
    // skip to byte 128
    reader.seek(SeekFrom::Start(128))?;
    let scanline_length = bit_planes * bytes_per_plane;
    let line_padding = scanline_length * 8 / bits_per_pixel - width;
    let mut surface = BitmapSurface::new(width as u32, height as u32);
    // finished reading the header, now reading the pixel data
    {
        let max_pixel = (surface.width() * surface.height()) as usize;
        let mut pixel_index = 0;
        let mut scanline_position = 0;
        // if the image uses PCX run-length encoding
        if using_rle {
            while pixel_index < max_pixel {
                let (mut run_length, color_index, rle_run, padding) = {
                    if scanline_position < scanline_length {
                        let first_byte = reader.read_u8()?;
                        // if it's a RLE byte
                        if (first_byte & 0xC0) == 0xC0 {
                            let run_length = first_byte & 0x3F;
                            let second_byte = reader.read_u8()?;
                            (run_length as usize, second_byte as usize, true, false)
                        }
                        else {
                            (1, first_byte as usize, false, false)
                        }
                    }
                    else {
                        scanline_position = 0;
                        (line_padding as usize, 0, false, true)
                    }
                };
                if padding || scanline_position + (run_length as u16) >= scanline_length {
                    run_length = match (padding, rle_run, bytes_per_plane == width) {
                        (true, _, _) => run_length,
                        (false, true, _) => (scanline_length - scanline_position - line_padding) as usize,
                        (false, false, true) => 1,
                        (false, false, false) => 0,
                    };
                    // reset scanline
                    scanline_position = 0;
                }
                else {
                    scanline_position += run_length as u16;
                }
                if pixel_index + (run_length as usize) > max_pixel {
                    run_length = (max_pixel - pixel_index) as usize;
                }
                let color = {
                    if color_index > 0 {
                        palette.colors[color_index].clone()
                    }
                    else {
                        Color::Transparent
                    }
                };
                for _ in 0..run_length {
                    surface.pixels_mut()[pixel_index] = color.into();
                    pixel_index += 1;
                }
            }
        }
        else {
            // if it's raw pixels
            for pixel_index in 0..max_pixel {
                let color_byte = reader.read_u8()?;
                surface.pixels_mut()[pixel_index] = palette.colors[color_byte as usize].into();
            }
        }
    }
    Ok(surface)
}
