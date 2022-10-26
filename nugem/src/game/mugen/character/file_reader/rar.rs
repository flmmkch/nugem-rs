use std::{path::{Path, PathBuf}, io::Cursor, ffi::{CString, CStr, c_long, c_uint, c_int}};
use super::{FileReader, ReadSeek};

pub struct FileReaderRar {
    archive_path: PathBuf,
}

impl FileReaderRar {
    fn archive_path_cstring(archive_path: &Path) -> std::io::Result<CString> {
        let archive_path_str = archive_path.to_str().ok_or_else(|| super::invalid_path_unicode_error(archive_path))?;
        let cstring = std::ffi::CString::new(archive_path_str)?;
        Ok(cstring)
    }
    pub fn new(archive_path: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            archive_path,
        })
    }
    fn open_archive(&self, mode: c_uint) -> std::io::Result<OpenArchive> {
        let archive_path_cstring = Self::archive_path_cstring(&self.archive_path)?;
        OpenArchive::new(&archive_path_cstring, mode).map_err(|errno| std::io::Error::new(std::io::ErrorKind::Other, format!("Error {0} when trying to open RAR file {1}", errno, self.archive_path.display())))
    }
}

impl FileReader for FileReaderRar {
    fn read_file<'a>(&'a mut self, path: &Path) -> std::io::Result<Box<dyn ReadSeek + 'a>> {
        let mut rar_archive = self.open_archive(unrar_sys::RAR_OM_EXTRACT)?;
        let path_str = path.as_os_str().to_str().ok_or_else(|| super::invalid_path_unicode_error(path))?;
        let memory_file = rar_archive.find_file(path_str).ok_or_else(|| super::file_not_found_error(path))?;
        Ok(Box::new(Cursor::new(memory_file)))
    }

    fn file_names<'a>(&'a mut self) -> std::io::Result<Box<dyn Iterator<Item = PathBuf> + 'a>> {
        let rar_archive = self.open_archive(unrar_sys::RAR_OM_LIST)?;
        let file_paths: Vec<PathBuf> = rar_archive.file_names().map(PathBuf::from).collect();
        Ok(Box::new(file_paths.into_iter()))
    }
}

// from the unrar_sys code and https://github.com/muja/unrar.rs/pull/10

struct OpenArchive {
    handle: unrar_sys::Handle,
}

impl OpenArchive {
    fn new(name: &CStr, mode: c_uint) -> Result<Self, u32> {
        let mut archive_data = unrar_sys::OpenArchiveData::new(name.as_ptr(), mode);
        unsafe {
            let handle = unrar_sys::RAROpenArchive(&mut archive_data as *mut _);
            if !handle.is_null() && archive_data.open_result == unrar_sys::ERAR_SUCCESS as u32 {
                Ok(Self {
                    handle,
                })
            }
            else {
                Err(archive_data.open_result)
            }
        }
    }
    fn read_c_string(src: &[core::ffi::c_char]) -> std::borrow::Cow<str> {
        let slice = unsafe { &*(src as *const _  as *const [u8]) };
        let str_slice = &slice[0..slice.iter().copied().take_while(|c| *c > 0).count()];
        String::from_utf8_lossy(str_slice)
    }
    fn file_names<'a>(&'a self) -> impl Iterator<Item = String> + 'a {
        self.iter_internal()
            .map_while(|header| {
                let read_result = unsafe { unrar_sys::RARProcessFile(self.handle, unrar_sys::RAR_SKIP, std::ptr::null(), std::ptr::null()) };
                if read_result == unrar_sys::ERAR_SUCCESS {
                    let header_filename_string = Self::read_c_string(&header.filename).into_owned();
                    Some(header_filename_string)
                }
                else {
                    None
                }
            })
    }
    fn iter_internal<'a>(&'a self) -> impl Iterator<Item = unrar_sys::HeaderData> + 'a {
        std::iter::from_fn(move || {
            let mut header = unrar_sys::HeaderData::default();
            let read_result = unsafe { unrar_sys::RARReadHeader(self.handle, &mut header as *mut _) };
            if read_result == unrar_sys::ERAR_SUCCESS {
                Some(header)
            } else {
                None
            }
        })
    }
    extern "C" fn callback_bytes(msg: c_uint, user_data: c_long, p1: c_long, p2: c_long) -> c_int {
        // println!("msg: {}, user_data: {}, p1: {}, p2: {}", msg, user_data, p1, p2);
        match msg {
            unrar_sys::UCM_PROCESSDATA => {
                let vec = unsafe { &mut *(user_data as *mut Vec<u8>) };
                let bytes = unsafe { std::slice::from_raw_parts(p1 as *const _, p2 as usize) };
                vec.extend_from_slice(bytes);
                1
            }
            _ => 0,
        }
    }
    fn find_file(&mut self, file_name: &str) -> Option<Vec<u8>> {
        let normalized_file_name = file_name.replace("\\", "/");
        for header in self.iter_internal() {
            let rar_file_name = Self::read_c_string(&header.filename);
            if super::file_name_match(&normalized_file_name, &rar_file_name) {
                let mut bytes = Vec::new();
                unsafe { unrar_sys::RARSetCallback(self.handle, Self::callback_bytes, &mut bytes as *mut _ as c_long) };
                let process_result = unsafe { unrar_sys::RARProcessFile(self.handle, unrar_sys::RAR_TEST, std::ptr::null(), std::ptr::null()) };
                if process_result == unrar_sys::ERAR_SUCCESS {
                    return Some(bytes);
                }
                else {
                    break;
                }
            }
            else {
                // skip
                let skip_result = unsafe { unrar_sys::RARProcessFile(self.handle, unrar_sys::RAR_SKIP, std::ptr::null(), std::ptr::null()) };
                if skip_result != unrar_sys::ERAR_SUCCESS {
                    break;
                }
            }
        }
        None
    }
}

impl Drop for OpenArchive {
    fn drop(&mut self) {
        unsafe {
            unrar_sys::RARCloseArchive(self.handle);
        }
    }
}

struct OpenArchiveFile<'a> {
    owner: std::marker::PhantomData<&'a OpenArchive>,
}
