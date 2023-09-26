use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::helpers::error::to_io_error;
use crate::helpers::serializable::SerializableExt;
use crate::util::jenkins::hash_string;

use super::archive_table::ArchiveTable;
use super::file_lists::FileListEntry;

const SIMPLE_4_LOOKUP: [(u32, &str); 5] = [
    (0x20534444, "dds"),
    (0x41444620, "adf"),
    (0x43505452, "rtpc"),
    (0x57E0E057, "ban"),
    (0x35425346, "fsb5"),
];

const SIMPLE_8_LOOKUP: [(u64, &str); 3] = [
    (0x000000300000000E, "btc"),
    (0x444E425200000005, "rbn"),
    (0x4453425200000005, "rbs"),
];

pub struct PackedArchive {}

pub struct PackedArchiveEntry {
    pub name: String,
    pub contents: Vec<u8>,
}

#[allow(dead_code)]
impl PackedArchive {
    pub fn deserialize<R: Seek + Read>(
        input: &mut R,
        archive_table: &ArchiveTable,
        file_list_entries: &HashMap<u32, FileListEntry>,
    ) -> std::io::Result<Vec<PackedArchiveEntry>> {
        let mut entries = Vec::new();

        for entry in &archive_table.entries {
            let name = match file_list_entries.get(&entry.name_hash) {
                Some(entry) => entry.name.to_owned(),
                None => {
                    // reads between entry.size and 32 bytes into a vector then transforms it to an array
                    let mut guess = vec![0u8; min(32, entry.size)];
                    input.seek(SeekFrom::Start(entry.offset.try_into().unwrap()))?;
                    let read = input.read(&mut guess)?;
                    let guess = guess.as_slice();

                    let extension = PackedArchive::detect_file_extension(guess, read)?;
                    let name = format!("{}", entry.name_hash) + "." + extension;
                    let path: PathBuf = ["__UNKNOWN", extension, &name].iter().collect();
                    path.to_string_lossy().to_string()
                }
            };

            input.seek(SeekFrom::Start(
                entry.offset.try_into().map_err(to_io_error)?,
            ))?;
            let mut contents = vec![0u8; entry.size];
            input.read(&mut contents)?;

            entries.push(PackedArchiveEntry {
                name: name.to_owned(),
                contents,
            })
        }

        Ok(entries)
    }
    pub fn deserialize_from_path<P: AsRef<Path>>(
        path: &P,
        archive_table: &ArchiveTable,
        file_list_entries: &HashMap<u32, FileListEntry>,
    ) -> std::io::Result<Vec<PackedArchiveEntry>> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        PackedArchive::deserialize(&mut buf_reader, archive_table, file_list_entries)
    }
    pub fn deserialize_from_entry(
        file_list_entry: &FileListEntry,
        game_dir: &str,
    ) -> std::io::Result<PackedArchiveEntry> {
        let game_dir_path = Path::new(game_dir);
        let base_path = Path::new(&file_list_entry.arc_name);
        let base_path = game_dir_path.join(base_path);

        let archive_table_path = base_path.with_extension("tab");
        let mut archive_table = ArchiveTable::deserialize_from_path(&archive_table_path)?;

        // binary search for provided name
        let name_hash = hash_string(file_list_entry.name.as_str());
        archive_table.entries.sort_by_key(|e| e.name_hash);
        let archive_entry = archive_table
            .entries
            .binary_search_by_key(&name_hash, |e| e.name_hash)
            .map_err(|_e| Error::new(ErrorKind::InvalidData, "archive table entry not found!"))?;
        let archive_entry = &archive_table.entries[archive_entry];

        let packed_archive_path = base_path.with_extension("arc");
        let mut packed_archive_file = File::open(packed_archive_path)?;

        packed_archive_file.seek(SeekFrom::Start(
            archive_entry.offset.try_into().map_err(to_io_error)?,
        ))?;
        let mut contents = vec![0u8; archive_entry.size];
        packed_archive_file.read(&mut contents)?;

        Ok(PackedArchiveEntry {
            name: file_list_entry.name.to_owned(),
            contents,
        })
    }

    fn detect_file_extension(guess: &[u8], read: usize) -> std::io::Result<&str> {
        if read == 0 {
            return Ok("null");
        }

        if read >= 4 {
            let magic: [u8; 4] = guess[0..4].try_into().map_err(to_io_error)?;
            let magic = u32::from_be_bytes(magic);
            for (key, extension) in SIMPLE_4_LOOKUP {
                if magic == key {
                    return Ok(extension);
                }
            }
        }

        if read >= 8 {
            let magic: [u8; 8] = guess[0..8].try_into().map_err(to_io_error)?;
            let magic = u64::from_be_bytes(magic);
            for (key, extension) in SIMPLE_8_LOOKUP {
                if magic == key {
                    return Ok(extension);
                }
            }
        }

        if read >= 3 {
            if guess[0] == 1 && guess[1] == 4 && guess[2] == 0 {
                return Ok("bin");
            }
        }

        Ok("unknown")
    }
}
