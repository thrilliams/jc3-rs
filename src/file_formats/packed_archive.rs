use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::helpers::error::to_io_error;
use crate::helpers::serializable::SerializableExt;

use super::archive_table::ArchiveTable;
use super::file_lists::{FileListEntry, FileLists};

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
        file_list_entries: &mut Vec<FileListEntry>,
    ) -> std::io::Result<Vec<PackedArchiveEntry>> {
        let mut entries = Vec::new();

        file_list_entries.sort_unstable_by_key(|entry| entry.name_hash);

        for entry in &archive_table.entries {
            let name = match file_list_entries
                .binary_search_by_key(&entry.name_hash, |vec_entry| vec_entry.name_hash)
            {
                Ok(index) => file_list_entries[index].name.to_owned(),
                Err(_err) => continue,
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
        file_list_entries: &mut Vec<FileListEntry>,
    ) -> std::io::Result<Vec<PackedArchiveEntry>> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        PackedArchive::deserialize(&mut buf_reader, archive_table, file_list_entries)
    }

    pub fn deserialize_from_file_lists<P: AsRef<Path>>(
        file_lists: FileLists,
        game_dir: &P,
    ) -> std::io::Result<Vec<PackedArchiveEntry>> {
        let mut entries = Vec::new();

        for (path, mut file_list_entries) in file_lists {
            let mut base_path = PathBuf::new();
            base_path.push(game_dir);
            base_path.push(path);

            let archive_table_path = base_path.with_extension("tab");
            let archive_table = ArchiveTable::deserialize_from_path(&archive_table_path)?;

            let packed_archive_path = base_path.with_extension("arc");
            let packed_archive_entries = PackedArchive::deserialize_from_path(
                &packed_archive_path,
                &archive_table,
                &mut file_list_entries,
            )?;

            for entry in packed_archive_entries {
                entries.push(entry);
            }
        }

        Ok(entries)
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
