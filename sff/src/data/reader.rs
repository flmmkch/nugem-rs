use std::io::Read;

use crate::LoadingError;

type SignatureBytes = [u8; 12];

pub const SIGNATURE_BYTES: &'static [u8; 12] = b"ElecbyteSpr\0";

pub type VersionBytes = [u8; 4];

pub fn check_signature<T: Read>(mut reader: T) -> Result<(), LoadingError> {
    let mut sig_buffer: SignatureBytes = [0; SIGNATURE_BYTES.len()];
    reader.read_exact(&mut sig_buffer).map_err(|io_error| LoadingError::IoError(io_error))?;
    if &sig_buffer != SIGNATURE_BYTES {
        Err(LoadingError::NoSignature)?;
    }
    Ok(())
}
