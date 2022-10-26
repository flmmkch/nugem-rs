use super::{FileReader, ReadSeek};
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FileReaderFs {
    path_root: PathBuf,
}

impl FileReaderFs {
    pub fn new(path_root: PathBuf) -> Self {
        Self {
            path_root,
        }
    }
}

impl FileReader for FileReaderFs {
    fn read_file<'a>(&'a mut self, path: &Path) -> std::io::Result<Box<dyn ReadSeek + 'a>> {
        let full_file_path = self.path_root.join(path);
        let file = File::open(&full_file_path)?;
        Ok(Box::new(file))
    }

    fn file_names<'a>(&'a mut self) -> std::io::Result<Box<dyn Iterator<Item = PathBuf>>> {
        let walk_dir_entries = WalkDir::new(&self.path_root).into_iter()
            .filter_map(|entry_res| entry_res.ok())
            .map(|entry| PathBuf::from(entry.file_name()));
        Ok(Box::new(walk_dir_entries))
    }
}