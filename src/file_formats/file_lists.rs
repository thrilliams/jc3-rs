use glob::glob;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind};

use crate::helpers::error::to_io_error;
use crate::util::jenkins::hash_string;

pub struct FileListEntry {
    pub name: String,
    pub arc_name: String,
}

#[allow(dead_code)]
pub fn load() -> std::io::Result<HashMap<u32, FileListEntry>> {
    let mut name_hashes: HashMap<u32, FileListEntry> = HashMap::new();

    for entry in glob("file_lists/**/*.filelist").map_err(to_io_error)? {
        let path = entry.map_err(to_io_error)?;

        let arc_name = path
            .strip_prefix("file_lists/")
            .map_err(to_io_error)?
            .with_extension("")
            .to_string_lossy()
            .to_string();

        let contents = read_to_string(&path)?;

        for line in contents.split('\n') {
            let name = line.trim();
            if name.starts_with(';') || name.len() == 0 {
                continue;
            }

            let name_hash = hash_string(name);
            if name_hashes.contains_key(&name_hash) {
                let extant_entry = name_hashes
                    .get(&name_hash)
                    .expect("nonsensical hash error!");
                // some names are included in multiple files which is fine i think
                if name == extant_entry.name && arc_name != extant_entry.arc_name {
                    continue;
                }
                println!(
                    "{name} == {0} in {1}",
                    extant_entry.name, extant_entry.arc_name
                );
                return Err(Error::new(ErrorKind::InvalidData, "hash collision!"));
            }

            name_hashes.insert(
                name_hash,
                FileListEntry {
                    name: name.to_string(),
                    arc_name: arc_name.clone(),
                },
            );
        }
    }

    Ok(name_hashes)
}
