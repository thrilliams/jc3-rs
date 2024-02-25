use glob::glob;
use std::collections::HashMap;
use std::fs::read_to_string;

use crate::helpers::error::to_io_error;
use crate::util::jenkins::hash_string;

pub type FileLists = HashMap<String, Vec<FileListEntry>>;

pub struct FileListEntry {
    pub name: String,
    pub name_hash: u32,
}

pub fn load_with_filter(filter: fn(&str) -> bool) -> std::io::Result<FileLists> {
    let mut file_lists: FileLists = HashMap::new();

    for entry in glob("file_lists/**/*.filelist").map_err(to_io_error)? {
        let path = entry.map_err(to_io_error)?;
        let contents = read_to_string(&path)?;

        let mut entries = Vec::new();
        for line in contents.split('\n') {
            let name = line.trim().to_string();
            if name.starts_with(';') || name.len() == 0 || !filter(&name) {
                continue;
            }

            let name_hash = hash_string(&name);

            entries.push(FileListEntry { name, name_hash });
        }

        if entries.len() == 0 {
            continue;
        }

        let arc_name = path
            .strip_prefix("file_lists/")
            .map_err(to_io_error)?
            .with_extension("")
            .to_string_lossy()
            .to_string();

        file_lists.insert(arc_name, entries);
    }

    Ok(file_lists)
}

pub fn load() -> std::io::Result<FileLists> {
    load_with_filter(|_| true)
}
