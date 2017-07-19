pub mod v1;
pub mod v2;
use std::io::{self, Read, Seek};

#[derive(Debug)]
pub enum Data {
    V1(v1::Data),
    V2(v2::Data),
}

#[derive(Debug)]
pub enum Error {
    NoSignature,
    ReadError(io::Error),
    UnknownVersion,
    SffV1Error(v1::Error),
    SffV2Error(v2::Error),
}

pub fn read<T: Read + Seek>(mut reader: T) -> Result<Data, Error> {
    // first: the signature at the start of the file
    {
        let mut sig_buffer = [0; 12];
        match reader.read_exact(&mut sig_buffer) {
            Ok(()) => {
                if &sig_buffer != b"ElecbyteSpr\0" {
                    return Err(Error::NoSignature);
                }
            },
            Err(e) => return Err(Error::ReadError(e)),
        }
    }
    // then: the version bytes
    let mut v_buffer = [0; 4];
    match reader.read_exact(&mut v_buffer) {
        Ok(()) => {
            if &v_buffer == &[0, 1, 0, 1] {
                match v1::read(reader) {
                    Ok(d) => Ok(Data::V1(d)),
                    Err(e) => Err(Error::SffV1Error(e)),
                }
            }
            else {
                if &v_buffer == &[0, 1, 0, 2] {
                    match v2::read(reader) {
                        Ok(d) => Ok(Data::V2(d)),
                        Err(e) => Err(Error::SffV2Error(e)),
                    }
                }
                else {
                    Err(Error::UnknownVersion)
                }
            }
        }
        Err(e) => Err(Error::ReadError(e)),
    }
}

pub fn read_u32<T: Read>(reader: &mut T) -> Result<u32, io::Error> {
    let mut v_buffer = [0; 4];
    reader.read_exact(&mut v_buffer)?;
    let result = v_buffer
                    .iter()
                    .fold((0 as u32, 0), |(mut v, n), c| {
                        let a = ((*c) as u32) << n;
                        v += a;
                        (v, n+8)
                        })
                    .0;
    Ok(result)
}

pub fn read_u16<T: Read>(reader: &mut T) -> Result<u16, io::Error> {
    let mut v_buffer = [0; 2];
    reader.read_exact(&mut v_buffer)?;
    let result = v_buffer
                    .iter()
                    .fold((0 as u16, 0), |(mut v, n), c| {
                        let a = ((*c) as u16) << n;
                        v += a;
                        (v, n+8)
                        })
                    .0;
    Ok(result)
}

pub fn read_u8<T: Read>(reader: &mut T) -> Result<u8, io::Error> {
    let mut v_buffer = [0; 1];
    reader.read_exact(&mut v_buffer)?;
    Ok(v_buffer[0])
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_read_u32() {
        use std::io::Cursor;
        {
            let original_buffer = [2, 0, 0, 0, 0, 0, 0, 0];
            let mut reader = Cursor::new(&original_buffer);
            let v = read_u32(&mut reader).unwrap();
            assert_eq!(v, 2);
        }
        {
            let original_buffer = [0, 1, 0, 0, 0, 0, 0, 0];
            let mut reader = Cursor::new(&original_buffer);
            let v = read_u32(&mut reader).unwrap();
            assert_eq!(v, 256);
        }
        {
            let original_buffer = [0, 0, 1, 0, 0, 0, 0, 0];
            let mut reader = Cursor::new(&original_buffer);
            let v = read_u32(&mut reader).unwrap();
            assert_eq!(v, 65536);
        }
        {
            let original_buffer = [0, 1, 1, 0];
            let mut reader = Cursor::new(&original_buffer);
            let v = read_u32(&mut reader).unwrap();
            assert_eq!(v, 65792);
        }
    }
}
