use std::io::{Cursor, Error, ErrorKind, Read, SeekFrom};

use flate2::read::ZlibDecoder;

use crate::helpers::{byte::ByteReaderExt, error::to_io_error, serializable::SerializableExt};

const SIGNATURE: &[u8; 4] = b"AAF\0"; // 0x00464141
const COMMENT: &[u8; 28] = b"AVALANCHEARCHIVEFORMATISCOOL";
const CHUNK_SIGNATURE: &[u8; 4] = b"EWAM"; // 0x4D415745

pub struct CoolArchive {
    pub le: bool,
    pub total_uncompressed_size: u32,
    pub block_size: u32,
    pub chunks: Vec<CoolArchiveChunk>,
}

pub struct CoolArchiveChunk {
    pub data_offset: u64,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub contents: Vec<u8>,
}

impl SerializableExt<CoolArchive> for CoolArchive {
    fn deserialize<R: std::io::Seek + std::io::Read>(
        mut input: &mut R,
    ) -> std::io::Result<CoolArchive> {
        let le = input.validate_signature(SIGNATURE)?;

        let version = input.read_u32(le)?;
        if version != 1 {
            return Err(Error::new(ErrorKind::InvalidData, "version did not match!"));
        }

        let comment_bytes: [u8; 28] = input.read_bytes()?;
        if COMMENT != &comment_bytes {
            return Err(Error::new(ErrorKind::InvalidData, "comment did not match!"));
        }

        let total_uncompressed_size = input.read_u32(le)?;
        let block_size = input.read_u32(le)?;
        let block_count = input.read_u32(le)?;

        let mut chunks = Vec::new();
        for _i in 0..block_count {
            let data_offset = input.stream_position()?;

            let compressed_size = input.read_u32(le)?;
            let uncompressed_size = input.read_u32(le)?;
            let next_offset = input.read_u32(le)?;
            let block_magic: [u8; 4] = input.read_bytes()?;

            if CHUNK_SIGNATURE != &block_magic {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "chunk signature did not match!",
                ));
            }

            input.seek(SeekFrom::Start(data_offset))?;

            let mut decoder = ZlibDecoder::new(&mut input);
            let mut contents = vec![0u8; uncompressed_size.try_into().map_err(to_io_error)?];
            decoder.read_exact(&mut contents)?;

            chunks.push(CoolArchiveChunk {
                data_offset,
                compressed_size,
                uncompressed_size,
                contents,
            });

            input.seek(SeekFrom::Start(data_offset + next_offset as u64))?;
        }

        Ok(CoolArchive {
            le,
            total_uncompressed_size,
            block_size,
            chunks,
        })
    }

    fn serialize<R: std::io::Seek + std::io::Write>(&self, _output: &mut R) -> std::io::Result<()> {
        unimplemented!()
    }
}
