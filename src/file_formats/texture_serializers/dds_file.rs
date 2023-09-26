use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::{
    file_formats::texture_file::{TextureFile, TextureSerializerExt},
    helpers::{byte::*, serializable::SerializablePartExt},
};

pub struct DDSFile {}

impl DDSFile {
    fn get_pixel_format(texture: &TextureFile) -> std::io::Result<PixelFormat> {
        // https://msdn.microsoft.com/en-us/library/windows/desktop/bb173059.aspx "DXGI_FORMAT enumeration"
        // https://msdn.microsoft.com/en-us/library/windows/desktop/cc308051.aspx "Legacy Formats: Map Direct3D 9 Formats to Direct3D 10"
        match texture.format {
            // DXGI_FORMAT_BC1_UNORM
            71 => Ok(PixelFormat::new(FileFormat::DXT1)?),
            // DXGI_FORMAT_BC2_UNORM
            74 => Ok(PixelFormat::new(FileFormat::DXT3)?),
            // DXGI_FORMAT_BC3_UNORM
            77 => Ok(PixelFormat::new(FileFormat::DXT5)?),
            // DXGI_FORMAT_B8G8R8A8_UNORM
            87 => Ok(PixelFormat::new(FileFormat::A8R8G8B8)?),

            // DXGI_FORMAT_R8_UNORM, DXGI_FORMAT_BC5_UNORM, DXGI_FORMAT_BC7_UNORM
            x if x == 61 || x == 83 || x == 98 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: 0,
                four_cc: 0x30315844, // 'DX10'
                rgb_bit_count: 0,
                red_bit_mask: 0,
                green_bit_mask: 0,
                blue_bit_mask: 0,
                alpha_bit_mask: 0,
            }),

            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "unrecognized texture format!",
            )),
        }

        // throw new NotSupportedException();
    }
}

impl TextureSerializerExt for DDSFile {
    fn serialize<R: Seek + Write>(output: &mut R, texture: &TextureFile) -> std::io::Result<()> {
        let le = true;

        let header = DDSHeader {
            size: DDSHeader::DEFAULT_SIZE,
            flags: HeaderFlags::Texture as u32 | HeaderFlags::Mipmap as u32,
            height: texture.height.into(),
            width: texture.width.into(),
            pitch_or_linear_size: 0,
            depth: 0,
            mip_map_count: texture.mip_count.into(),
            reserved_1: [0u8; 11 * 4],
            pixel_format: DDSFile::get_pixel_format(texture)?,
            surface_flags: 8,
            cubemap_flags: 0,
            reserved_2: [0u8; 3 * 4],
        };

        output.write_u32(0x20534444, le)?;
        header.write(output, le)?;

        if header.pixel_format.four_cc == 0x3031584 {
            output.write_u32(texture.format, le)?;
            output.write_u32(2, le)?;
            output.write_u32(0, le)?;
            output.write_u32(1, le)?;
            output.write_u32(0, le)?;
        }

        output.write(&texture.elements[0].contents)?;

        Ok(())
    }
}

pub struct DDSHeader {
    pub size: u32,
    pub flags: u32,
    pub height: i32,
    pub width: i32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,
    pub reserved_1: [u8; 11 * 4],
    pub pixel_format: PixelFormat,
    pub surface_flags: u32,
    pub cubemap_flags: u32,
    pub reserved_2: [u8; 3 * 4],
}

impl DDSHeader {
    pub const DEFAULT_SIZE: u32 = (18 * 4) + PixelFormat::DEFAULT_SIZE + (5 * 4);
}

impl SerializablePartExt<DDSHeader> for DDSHeader {
    fn read<R: Seek + Read>(input: &mut R, le: bool) -> std::io::Result<DDSHeader> {
        let size = input.read_u32(le)?;
        let flags = input.read_u32(le)?;
        let height = input.read_s32(le)?;
        let width = input.read_s32(le)?;
        let pitch_or_linear_size = input.read_u32(le)?;
        let depth = input.read_u32(le)?;
        let mip_map_count = input.read_u32(le)?;

        let mut reserved_1 = [0u8; 11 * 4];
        if input.read(&mut reserved_1)? != reserved_1.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "not enough reserved data read!",
            ));
        }

        let pixel_format = PixelFormat::read(input, le)?;
        let surface_flags = input.read_u32(le)?;
        let cubemap_flags = input.read_u32(le)?;

        let mut reserved_2 = [0u8; 3 * 4];
        if input.read(&mut reserved_2)? != reserved_2.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "not enough reserved data read!",
            ));
        }

        Ok(DDSHeader {
            size,
            flags,
            height,
            width,
            pitch_or_linear_size,
            depth,
            mip_map_count,
            reserved_1,
            pixel_format,
            surface_flags,
            cubemap_flags,
            reserved_2,
        })
    }

    fn write<R: Seek + Write>(&self, output: &mut R, le: bool) -> std::io::Result<()> {
        output.write_u32(self.size, le)?;
        output.write_u32(self.flags, le)?;
        output.write_s32(self.height, le)?;
        output.write_s32(self.width, le)?;
        output.write_u32(self.pitch_or_linear_size, le)?;
        output.write_u32(self.depth, le)?;
        output.write_u32(self.mip_map_count, le)?;
        output.write(&self.reserved_1)?;
        self.pixel_format.write(output, le)?;
        output.write_u32(self.surface_flags, le)?;
        output.write_u32(self.cubemap_flags, le)?;
        output.write(&self.reserved_2)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[repr(u32)]
enum HeaderFlags {
    Texture = 0x00001007, // DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT
    Mipmap = 0x00020000,  // DDSD_MIPMAPCOUNT
    Volume = 0x00800000,  // DDSD_DEPTH
    Pitch = 0x00000008,   // DDSD_PITCH
    LinerSize = 0x00080000, // DDSD_LINEARSIZE
}

pub struct PixelFormat {
    pub size: u32,
    pub flags: u32,
    pub four_cc: u32,
    pub rgb_bit_count: u32,
    pub red_bit_mask: u32,
    pub green_bit_mask: u32,
    pub blue_bit_mask: u32,
    pub alpha_bit_mask: u32,
}

impl PixelFormat {
    pub const DEFAULT_SIZE: u32 = 8 * 4;

    pub fn new(file_format: FileFormat) -> std::io::Result<PixelFormat> {
        match file_format {
            FileFormat::DXT1 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::FourCC as u32,
                four_cc: 0x31545844, // "DXT1"
                rgb_bit_count: 0,
                red_bit_mask: 0,
                green_bit_mask: 0,
                blue_bit_mask: 0,
                alpha_bit_mask: 0,
            }),
            FileFormat::DXT3 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::FourCC as u32,
                four_cc: 0x33545844, // "DXT3"
                rgb_bit_count: 0,
                red_bit_mask: 0,
                green_bit_mask: 0,
                blue_bit_mask: 0,
                alpha_bit_mask: 0,
            }),
            FileFormat::DXT5 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::FourCC as u32,
                four_cc: 0x35545844, // "DXT5"
                rgb_bit_count: 0,
                red_bit_mask: 0,
                green_bit_mask: 0,
                blue_bit_mask: 0,
                alpha_bit_mask: 0,
            }),
            FileFormat::A8R8G8B8 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::RGBA as u32,
                four_cc: 0,
                rgb_bit_count: 32,
                red_bit_mask: 0x00FF0000,
                green_bit_mask: 0x0000FF00,
                blue_bit_mask: 0x000000FF,
                alpha_bit_mask: 0xFF000000,
            }),
            FileFormat::X8R8G8B8 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::RGB as u32,
                four_cc: 0,
                rgb_bit_count: 32,
                red_bit_mask: 0x00FF0000,
                green_bit_mask: 0x0000FF00,
                blue_bit_mask: 0x000000FF,
                alpha_bit_mask: 0x00000000,
            }),
            FileFormat::A8B8G8R8 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::RGBA as u32,
                four_cc: 0,
                rgb_bit_count: 32,
                red_bit_mask: 0x000000FF,
                green_bit_mask: 0x0000FF00,
                blue_bit_mask: 0x00FF0000,
                alpha_bit_mask: 0xFF000000,
            }),
            FileFormat::X8B8G8R8 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::RGB as u32,
                four_cc: 0,
                rgb_bit_count: 32,
                red_bit_mask: 0x000000FF,
                green_bit_mask: 0x0000FF00,
                blue_bit_mask: 0x00FF0000,
                alpha_bit_mask: 0x00000000,
            }),
            FileFormat::A1R5G5B5 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::RGBA as u32,
                four_cc: 0,
                rgb_bit_count: 16,
                red_bit_mask: 0x00007C00,
                green_bit_mask: 0x000003E0,
                blue_bit_mask: 0x0000001F,
                alpha_bit_mask: 0x00008000,
            }),
            FileFormat::A4R4G4B4 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::RGBA as u32,
                four_cc: 0,
                rgb_bit_count: 16,
                red_bit_mask: 0x00000F00,
                green_bit_mask: 0x000000F0,
                blue_bit_mask: 0x0000000F,
                alpha_bit_mask: 0x0000F000,
            }),
            FileFormat::R8G8B8 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::RGB as u32,
                four_cc: 0,
                rgb_bit_count: 24,
                red_bit_mask: 0x00FF0000,
                green_bit_mask: 0x0000FF00,
                blue_bit_mask: 0x000000FF,
                alpha_bit_mask: 0x00000000,
            }),
            FileFormat::R5G6B5 => Ok(PixelFormat {
                size: PixelFormat::DEFAULT_SIZE,
                flags: PixelFormatFlags::RGB as u32,
                four_cc: 0,
                rgb_bit_count: 16,
                red_bit_mask: 0x0000F800,
                green_bit_mask: 0x000007E0,
                blue_bit_mask: 0x0000001F,
                alpha_bit_mask: 0x00000000,
            }),
            FileFormat::INVALID => Err(Error::new(
                ErrorKind::InvalidInput,
                "file format not valid!",
            )),
        }
    }
}

impl SerializablePartExt<PixelFormat> for PixelFormat {
    fn read<R: Seek + Read>(input: &mut R, le: bool) -> std::io::Result<PixelFormat> {
        let size = input.read_u32(le)?;
        let flags = input.read_u32(le)?;
        let four_cc = input.read_u32(le)?;
        let rgb_bit_count = input.read_u32(le)?;
        let red_bit_mask = input.read_u32(le)?;
        let green_bit_mask = input.read_u32(le)?;
        let blue_bit_mask = input.read_u32(le)?;
        let alpha_bit_mask = input.read_u32(le)?;

        Ok(PixelFormat {
            size,
            flags,
            four_cc,
            rgb_bit_count,
            red_bit_mask,
            green_bit_mask,
            blue_bit_mask,
            alpha_bit_mask,
        })
    }

    fn write<R: Seek + Write>(&self, output: &mut R, le: bool) -> std::io::Result<()> {
        output.write_u32(self.size, le)?;
        output.write_u32(self.flags as u32, le)?;
        output.write_u32(self.four_cc, le)?;
        output.write_u32(self.rgb_bit_count, le)?;
        output.write_u32(self.red_bit_mask, le)?;
        output.write_u32(self.green_bit_mask, le)?;
        output.write_u32(self.blue_bit_mask, le)?;
        output.write_u32(self.alpha_bit_mask, le)?;
        Ok(())
    }
}

#[allow(dead_code)]
enum PixelFormatFlags {
    FourCC = 0x00000004,
    RGB = 0x00000040,
    RGBA = 0x00000041,
    Luminance = 0x00020000,
}

#[allow(dead_code)]
pub enum FileFormat {
    DXT1,
    DXT3,
    DXT5,
    A8R8G8B8,
    X8R8G8B8,
    A8B8G8R8,
    X8B8G8R8,
    A1R5G5B5,
    A4R4G4B4,
    R8G8B8,
    R5G6B5,
    INVALID,
}
