use std::{
    fs::File,
    io::{BufWriter, Cursor, Seek, Write},
    path::Path,
};

use crate::file_formats::texture_file::TextureFile;

// of the three currently supported formats, DDS is the fastest since data isn't processed
// webp is the slowest, but yields by far the smallest files
// png is average, but well-supported and acceptably slow to run as-needed for e.g. a webserver

pub trait TextureSerializerExt {
    fn serialize<R: Seek + Write>(output: &mut R, texture: &TextureFile) -> std::io::Result<()>;

    fn serialize_to_path<P: AsRef<Path>>(path: &P, texture: &TextureFile) -> std::io::Result<()> {
        let file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        let mut buf_writer = BufWriter::new(file);
        Self::serialize(&mut buf_writer, texture)
    }
    fn serialize_to_bytes(texture: &TextureFile) -> std::io::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut writer = Cursor::new(&mut bytes);
        Self::serialize(&mut writer, texture)?;
        Ok(bytes)
    }
}
