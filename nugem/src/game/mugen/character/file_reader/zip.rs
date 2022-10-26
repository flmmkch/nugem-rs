use zip::ZipArchive;
use std::{fs::File, path::{PathBuf, Path}, io::{Cursor, Read}};

use super::{FileReader, ReadSeek};

pub struct FileReaderZip {
    archive_path: PathBuf,
}

impl FileReaderZip {
    pub fn new(archive_path: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            archive_path,
        })
    }
}

impl FileReader for FileReaderZip {
    fn read_file<'a>(&'a mut self, path: &Path) -> std::io::Result<Box<dyn ReadSeek + 'a>> {
        let zip_archive_file = File::open(&self.archive_path)?;
        let mut zip_archive = ZipArchive::new(zip_archive_file)?;
        let path_str = path.as_os_str().to_str().ok_or_else(|| super::invalid_path_unicode_error(path))?;
        let zip_file_index = zip_archive.file_names().position(|p| super::file_name_match(&path_str, &p)).ok_or_else(|| super::file_not_found_error(path))?;
        let mut zip_file = zip_archive.by_index(zip_file_index)?;
        let mut memory_file = Vec::new();
        zip_file.read_to_end(&mut memory_file)?;
        Ok(Box::new(Cursor::new(memory_file)))
    }

    fn file_names<'a>(&'a mut self) -> std::io::Result<Box<dyn Iterator<Item = PathBuf> + 'a>> {
        let zip_archive_file = File::open(&self.archive_path)?;
        let zip_archive = ZipArchive::new(zip_archive_file)?;
        let file_paths: Vec<PathBuf> = zip_archive.file_names().map(PathBuf::from).collect();
        Ok(Box::new(file_paths.into_iter()))
    }
}
