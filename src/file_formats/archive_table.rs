use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;

use crate::helpers::byte::ByteReaderExt;
use crate::helpers::byte::StreamLengthExt;
use crate::helpers::error::to_io_error;
use crate::helpers::serializable::SerializableExt;

const SIGNATURE: &[u8; 4] = b"TAB\0";

pub struct ArchiveTable {
    pub le: bool,
    pub alignment: u32,
    pub entries: Vec<ArchiveTableEntry>,
}

pub struct ArchiveTableEntry {
    pub name_hash: u32,
    pub offset: usize,
    pub size: usize,
}

impl SerializableExt<ArchiveTable> for ArchiveTable {
    fn deserialize<R: Seek + Read>(input: &mut R) -> std::io::Result<ArchiveTable> {
        let le = input.validate_signature(SIGNATURE)?;

        let unk04 = input.read_u16(le)?;
        let unk06 = input.read_u16(le)?;
        if unk04 != 2 || unk06 != 1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "unknown bits did not match!",
            ));
        }

        let alignment = input.read_u32(le)?;
        if alignment != 0x800 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "alignment did not match!",
            ));
        }

        let mut entries = Vec::new();

        let length = input.stream_length()?;
        while input.stream_position()? + 12 <= length {
            let name_hash = input.read_u32(le)?;
            let offset = input.read_u32(le)?;
            let size = input.read_u32(le)?;

            entries.push(ArchiveTableEntry {
                name_hash,
                offset: offset.try_into().map_err(to_io_error)?,
                size: size.try_into().map_err(to_io_error)?,
            })
        }

        Ok(ArchiveTable {
            le,
            alignment,
            entries,
        })
    }

    fn serialize<R: Seek + Write>(&self, _output: &mut R) -> std::io::Result<()> {
        unimplemented!()
    }
}
