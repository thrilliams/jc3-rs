use std::io::{Error, ErrorKind, Seek, Write};
use texpresso::Format;
use webp::{Encoder, PixelLayout};

use crate::file_formats::texture::Texture;

use super::texture_serializer::TextureSerializerExt;

pub struct WEBPFile {}

impl TextureSerializerExt for WEBPFile {
    fn serialize<R: Seek + Write>(output: &mut R, texture: &Texture) -> std::io::Result<()> {
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

        let encoded = Encoder::new(
            &decompressed,
            PixelLayout::Rgba,
            texture.width.into(),
            texture.height.into(),
        )
        .encode_lossless();

        output.write(&encoded)?;

        Ok(())
    }
}
