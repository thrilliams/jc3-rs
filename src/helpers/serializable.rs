use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;

// i'm pretty sure this is stupid
pub trait SerializableExt<T: SerializableExt<T>> {
    fn deserialize<R: Seek + Read>(input: &mut R) -> std::io::Result<T>;
    fn serialize<R: Seek + Write>(&self, output: &mut R) -> std::io::Result<()>;

    fn deserialize_from_path<P: AsRef<Path>>(path: &P) -> std::io::Result<T> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        T::deserialize(&mut buf_reader)
    }
    fn serialize_to_path<P: AsRef<Path>>(&self, path: &P) -> std::io::Result<()> {
        let file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        let mut buf_writer = BufWriter::new(file);
        self.serialize(&mut buf_writer)
    }

    fn deserialize_from_bytes(bytes: &[u8]) -> std::io::Result<T> {
        let mut reader = Cursor::new(bytes);
        T::deserialize(&mut reader)
    }
    fn serialize_to_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut writer = Cursor::new(&mut bytes);
        self.serialize(&mut writer)?;
        Ok(bytes)
    }
}

pub trait SerializablePartExt<T> {
    fn read<R: Seek + Read>(input: &mut R, le: bool) -> std::io::Result<T>;
    fn write<R: Seek + Write>(&self, output: &mut R, le: bool) -> std::io::Result<()>;
}
