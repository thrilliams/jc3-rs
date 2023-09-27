use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::SeekFrom;

use super::error::to_io_error;

// Seek.stream_length is expiremental so we're rolling our own
pub trait StreamLengthExt {
    fn stream_length(&mut self) -> std::io::Result<u64>;
}

impl<T: Seek> StreamLengthExt for T {
    fn stream_length(&mut self) -> std::io::Result<u64> {
        let current_position = self.stream_position()?;
        let end_position = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(current_position))?;
        return Ok(end_position);
    }
}

pub trait ByteReaderExt: std::io::Read {
    fn read_bytes<const N: usize>(&mut self) -> std::io::Result<[u8; N]>;

    fn read_u8(&mut self) -> std::io::Result<u8> {
        let bytes = self.read_bytes::<1>()?;
        Ok(bytes[0])
    }
    fn read_u16(&mut self, le: bool) -> std::io::Result<u16> {
        let bytes = self.read_bytes::<2>()?;
        let value = if le {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_be_bytes(bytes)
        };
        Ok(value)
    }
    fn read_u32(&mut self, le: bool) -> std::io::Result<u32> {
        let bytes = self.read_bytes::<4>()?;
        let value = if le {
            u32::from_le_bytes(bytes)
        } else {
            u32::from_be_bytes(bytes)
        };
        Ok(value)
    }

    fn read_b8(&mut self) -> std::io::Result<bool> {
        Ok(self.read_u8()? > 0)
    }

    fn read_s8(&mut self) -> std::io::Result<i8> {
        let bytes = self.read_bytes::<1>()?;
        let value = i8::from_ne_bytes(bytes);
        Ok(value)
    }
    fn read_s16(&mut self, le: bool) -> std::io::Result<i16> {
        let bytes = self.read_bytes::<2>()?;
        let value = if le {
            i16::from_le_bytes(bytes)
        } else {
            i16::from_be_bytes(bytes)
        };
        Ok(value)
    }
    fn read_s32(&mut self, le: bool) -> std::io::Result<i32> {
        let bytes = self.read_bytes::<4>()?;
        let value = if le {
            i32::from_le_bytes(bytes)
        } else {
            i32::from_be_bytes(bytes)
        };
        Ok(value)
    }

    fn read_string(&mut self, length: u32) -> std::io::Result<String> {
        let mut bytes = vec![0u8; length.try_into().map_err(to_io_error)?];
        self.read(&mut bytes)?;
        Ok(String::from_utf8(bytes).map_err(to_io_error)?)
    }

    fn validate_signature<const N: usize>(
        &mut self,
        le_signature: &[u8; N],
    ) -> std::io::Result<bool> {
        let mut magic = self.read_bytes::<N>()?;

        if &magic == le_signature {
            return Ok(true);
        }

        magic.reverse();

        if &magic == le_signature {
            return Ok(false);
        }

        Err(Error::new(
            ErrorKind::InvalidData,
            "file signature did not match!",
        ))
    }
}

pub trait ByteWriterExt {
    fn write_bytes<const N: usize>(&mut self, bytes: [u8; N]) -> std::io::Result<()>;

    fn write_u8(&mut self, value: u8) -> std::io::Result<()> {
        let bytes = value.to_ne_bytes();
        self.write_bytes(bytes)?;
        Ok(())
    }
    fn write_u16(&mut self, value: u16, le: bool) -> std::io::Result<()> {
        let bytes = if le {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        self.write_bytes(bytes)?;
        Ok(())
    }
    fn write_u32(&mut self, value: u32, le: bool) -> std::io::Result<()> {
        let bytes = if le {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        self.write_bytes(bytes)?;
        Ok(())
    }

    fn write_b8(&mut self, value: bool) -> std::io::Result<()> {
        self.write_u8(if value { 1 } else { 0 })
    }

    fn write_s8(&mut self, value: i8) -> std::io::Result<()> {
        let bytes = value.to_ne_bytes();
        self.write_bytes(bytes)?;
        Ok(())
    }
    fn write_s16(&mut self, value: i16, le: bool) -> std::io::Result<()> {
        let bytes = if le {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        self.write_bytes(bytes)?;
        Ok(())
    }
    fn write_s32(&mut self, value: i32, le: bool) -> std::io::Result<()> {
        let bytes = if le {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        self.write_bytes(bytes)?;
        Ok(())
    }
}

impl<T: Read> ByteReaderExt for T {
    fn read_bytes<const N: usize>(&mut self) -> std::io::Result<[u8; N]> {
        let mut bytes = [0u8; N];
        let read = self.read(&mut bytes)?;
        if read < N {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "not enough bytes were read!",
            ));
        }
        Ok(bytes)
    }
}

impl<T: Write> ByteWriterExt for T {
    fn write_bytes<const N: usize>(&mut self, bytes: [u8; N]) -> std::io::Result<()> {
        let written = self.write(&bytes)?;
        if written < N {
            return Err(Error::new(
                ErrorKind::Other,
                "not enough bytes were written!",
            ));
        }
        Ok(())
    }
}
