mod file_formats;
mod helpers;
mod util;

use std::fs::create_dir_all;
use std::path::Path;

use file_formats::texture::Texture;
use file_formats::texture_serializers::texture_serializer::TextureSerializerExt;
use helpers::serializable::SerializableExt;

use crate::file_formats::cool_archive::CoolArchive;
use crate::file_formats::file_lists::load_with_filter;
use crate::file_formats::packed_archive::PackedArchive;
use crate::file_formats::texture_serializers::webp_file::WEBPFile;

fn extract_map() -> std::io::Result<()> {
    let file_list_entries = load_with_filter(|name| {
        name.starts_with("textures/ui/zoom")
            || name.starts_with("textures/ui/world_map")
            || name.starts_with("textures/ui/dev_map_grid")
    })?;

    let packed_archive_entries =
        PackedArchive::deserialize_from_file_lists(file_list_entries, &"game_dir")?;

    for entry in packed_archive_entries {
        let path = Path::new(&entry.name);
        let dir_path = path.with_extension("");
        create_dir_all(dir_path.parent().unwrap())?;
        println!("{}", entry.name);
        let texture = Texture::deserialize_from_bytes(&entry.contents)?;
        WEBPFile::serialize_to_path(&path.with_extension("webp"), &texture)?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    // let file_list_entries = load_with_filter(|name| name.ends_with("l"))?;

    // let packed_archive_entries =
    //     PackedArchive::deserialize_from_file_lists(file_list_entries, &"game_dir")?;

    // for entry in &packed_archive_entries[0..10] {
    //     println!("{}", entry.name);
    //     let cool_archive = CoolArchive::deserialize_from_bytes(&entry.contents)?;
    //     for chunk in cool_archive.chunks {
    //         println!("{:?}", &chunk.contents[0..4]);
    //     }
    // }

    extract_map()?;

    Ok(())
}
