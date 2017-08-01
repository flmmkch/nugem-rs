use ::game::mugen::character::CharacterInfo;
use super::{Color, Palette};
use super::data::PALETTE_COLOR_COUNT;
use std::io::Read;
use std::path::Path;
use std::fs;
use std::io::BufReader;

fn read_palette<T: Read>(mut reader: T) -> Option<Palette> {
    let mut colors_buf = [0; PALETTE_COLOR_COUNT * 3];
    if let Ok(_) = reader.read(&mut colors_buf) {
        let mut palette = Palette {
            colors: [Color::Transparent; 256]
        };
        for i in 0..PALETTE_COLOR_COUNT {
            let red = colors_buf[i * 3];
            let green = colors_buf[i * 3 + 1];
            let blue = colors_buf[i * 3 + 2];
            palette.colors[PALETTE_COLOR_COUNT - 1 - i] = Color::Rgb(red, green, blue);
        }
        Some(palette)
    }
    else {
        None
    }
}

pub struct PaletteFilesReader<'a> {
    character_info: &'a CharacterInfo,
    character_dir: &'a Path,
    current_index: usize,
}

impl<'a> Iterator for PaletteFilesReader<'a> {
    type Item = Palette;

    fn next(&mut self) -> Option<Palette> {
        self.current_index += 1;
        let palette_file_key = format!("pal{}", self.current_index);
        if let Some(palette_file_name) = self.character_info["files"].get(&palette_file_key) {
            let mut palette = Palette {
                colors: [Color::Transparent; 256]
            };
            // read the palette file
            let path = self.character_dir.join(&palette_file_name);
            if path.is_file() {
                let file_res = fs::File::open(&path);
                if let Ok(file) = file_res {
                    let reader = BufReader::new(file);
                    if let Some(palette_read) = read_palette(reader) {
                        palette = palette_read;
                    }
                    else {
                        error!("Error reading palette file {}", path.to_string_lossy());
                    }
                }
                else {
                    error!("Error opening palette file {}", path.to_string_lossy());
                }
            }
            else {
                error!("Unable to open palette file {}", path.to_string_lossy());
            }
            Some(palette)
        }
        else {
            None
        }
    }
}

impl<'a> PaletteFilesReader<'a> {
    pub fn new(character_info: &'a CharacterInfo, character_dir: &'a Path) -> PaletteFilesReader<'a> {
        let current_index : usize = 0;
        PaletteFilesReader {
            character_info,
            character_dir,
            current_index,
        }
    }
}
