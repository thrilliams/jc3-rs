use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};

use crate::helpers::byte::*;
use crate::helpers::error::to_io_error;
use crate::helpers::serializable::{SerializableExt, SerializablePartExt};

const SIGNATURE: &[u8; 4] = b"AVTX"; // 0x58545641
const ELEMENT_COUNT: usize = 8;

pub struct Texture {
    pub le: bool,
    pub unknown_06: u8,
    pub dimension: u8,
    pub format: u32,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub flags: u16,
    pub mip_count: u8,
    pub header_mip_count: u8,
    pub unknown_1c: u32,
    pub elements: Vec<TextureElement>,
}

pub struct TextureElement {
    pub offset: u32,
    pub size: u32,
    pub unknown_8: u16,
    pub unknown_a: u8,
    pub is_external: bool,
    pub contents: Vec<u8>,
}

impl SerializableExt<Texture> for Texture {
    fn deserialize<R: Seek + Read>(input: &mut R) -> std::io::Result<Texture> {
        let le = input.validate_signature(SIGNATURE)?;

        let version = input.read_u16(le)?;
        if version != 1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "file version did not match!",
            ));
        }

        let unknown_06 = input.read_u8()?;
        let dimension = input.read_u8()?;
        let format = input.read_u32(le)?;
        let width = input.read_u16(le)?;
        let height = input.read_u16(le)?;
        let depth = input.read_u16(le)?;
        let flags = input.read_u16(le)?;
        let mip_count = input.read_u8()?;
        let header_mip_count = input.read_u8()?;

        let unknown_16 = input.read_u8()?;
        let unknown_17 = input.read_u8()?;
        let unknown_18 = input.read_u8()?;
        let unknown_19 = input.read_u8()?;
        let unknown_1a = input.read_u8()?;
        let unknown_1b = input.read_u8()?;
        let unknown_1c = input.read_u32(le)?;

        let mut elements = Vec::new();
        for _i in 0..ELEMENT_COUNT {
            let element = TextureElement::read(input, le)?;
            elements.push(element);
        }

        if flags != 0 && (flags & !(1 | 8 | 0x40)) != 0 {
            return Err(Error::new(ErrorKind::InvalidData, "flags did not match!"));
        }

        if unknown_17 != 0 || unknown_19 != 0 || unknown_1a != 0 || unknown_1b != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "unknown bits did not match!",
            ));
        }

        if unknown_16 != 0 && unknown_16 != 1 && unknown_16 != 2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "unknown bits did not match!",
            ));
        }

        if unknown_18 != 0
            && unknown_18 != 2
            && unknown_18 != 1
            && unknown_18 != 3
            && unknown_18 != 4
        {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "unknown bits did not match!",
            ));
        }

        Ok(Texture {
            le,
            unknown_06,
            dimension,
            format,
            width,
            height,
            depth,
            flags,
            mip_count,
            header_mip_count,
            unknown_1c,
            elements,
        })
    }

    fn serialize<R: Seek + Write>(&self, _output: &mut R) -> std::io::Result<()> {
        unimplemented!()
    }
}

impl SerializablePartExt<TextureElement> for TextureElement {
    fn read<R: Seek + Read>(input: &mut R, le: bool) -> std::io::Result<TextureElement> {
        let offset = input.read_u32(le)?;
        let size = input.read_u32(le)?;
        let unknown_8 = input.read_u16(le)?;
        let unknown_a = input.read_u8()?;
        let is_external = input.read_b8()?;

        let mut contents = Vec::new();
        if size > 0 {
            let safe_size: usize = size.try_into().map_err(to_io_error)?;
            contents.resize(safe_size, 0);
            let starting_position = input.stream_position()?;
            input.seek(SeekFrom::Start(offset.into()))?;
            let read = input.read(&mut contents)?;
            if read < safe_size {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "could not read texture contents!",
                ));
            }
            input.seek(SeekFrom::Start(starting_position))?;
        }

        Ok(TextureElement {
            offset,
            size,
            unknown_8,
            unknown_a,
            is_external,
            contents,
        })
    }

    fn write<R: Seek + Write>(&self, output: &mut R, le: bool) -> std::io::Result<()> {
        output.write_u32(self.offset, le)?;
        output.write_u32(self.size, le)?;
        output.write_u16(self.unknown_8, le)?;
        output.write_u8(self.unknown_a)?;
        output.write_b8(self.is_external)?;

        if self.size > 0 {
            let starting_position = output.stream_position()?;
            output.seek(SeekFrom::Start(self.offset.into()))?;
            let written = output.write(&self.contents)?;
            if written < self.size.try_into().map_err(to_io_error)? {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "could not write texture contents!",
                ));
            }
            output.seek(SeekFrom::Start(starting_position))?;
        }

        Ok(())
    }
}
