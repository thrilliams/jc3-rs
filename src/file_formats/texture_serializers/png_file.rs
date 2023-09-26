use std::io::{Error, ErrorKind, Seek, Write};

use png::{BitDepth, ColorType, Encoder};
use texpresso::Format;

use crate::file_formats::texture_file::TextureFile;

use super::texture_serializer::TextureSerializerExt;

pub struct PNGFile {}

impl TextureSerializerExt for PNGFile {
    fn serialize<R: Seek + Write>(output: &mut R, texture: &TextureFile) -> std::io::Result<()> {
        let format = match texture.format {
            // DXGI_FORMAT_BC1_UNORM
            71 => Ok(Format::Bc1),
            // DXGI_FORMAT_BC2_UNORM
            74 => Ok(Format::Bc2),
            // DXGI_FORMAT_BC3_UNORM
            77 => Ok(Format::Bc3),
            // DXGI_FORMAT_BC4_UNORM
            80 => Ok(Format::Bc4),
            // DXGI_FORMAT_BC5_UNORM
            83 => Ok(Format::Bc5),

            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "unsupported texture format!",
            )),
        }?;

        let width = texture.width as usize;
        let height = texture.height as usize;
        let mut decompressed = vec![0u8; 4 * width * height];
        format.decompress(
            &texture.elements[0].contents,
            texture.width.into(),
            texture.height.into(),
            &mut decompressed,
        );

        let mut encoder = Encoder::new(output, texture.width.into(), texture.height.into());
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header()?;

        writer.write_image_data(&decompressed)?;

        Ok(())
    }
}
