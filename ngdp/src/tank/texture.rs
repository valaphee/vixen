use anyhow::Result;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

#[derive(Debug)]
pub struct Texture {
    pub flags: TextureFlags,
    pub mip_level_count: u8,
    pub format: u8,
    pub depth_or_array_layers: u8,
    pub usage: u8,
    pub payload_count: u8,
    pub stencil: bool,
    pub width: u16,
    pub height: u16,
    pub payload: Option<TexturePayload>,
    pub unknown10: u8, // uv map mode?
    pub unknown11: u8, // type?
}

impl Texture {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let flags = TextureFlags::from_bits(input.read_u8()?).unwrap();
        input.read_u8()?; // always 0
        let mip_level_count = input.read_u8()?;
        let format = input.read_u8()?;
        let depth_or_array_layers = input.read_u8()?;
        let usage = input.read_u8()?;
        let payload_count = input.read_u8()?;
        let stencil = input.read_u8()? != 0;
        let width = input.read_u16::<LittleEndian>()?;
        let height = input.read_u16::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?; // size
        let unknown10 = input.read_u8()?;
        let unknown11 = input.read_u8()?;
        input.read_u32::<LittleEndian>()?; // padding
        input.read_u32::<LittleEndian>()?; // padding
        input.read_u32::<LittleEndian>()?; // padding
        let payload = if payload_count != 0 {
            Some(TexturePayload::read_from(input)?)
        } else {
            None
        };

        Ok(Self {
            flags,
            mip_level_count,
            format,
            depth_or_array_layers,
            usage,
            payload_count,
            stencil,
            width,
            height,
            payload,
            unknown10,
            unknown11,
        })
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct TextureFlags: u8 {
        const Texture1D = 1 << 0;
        const Texture2D = 1 << 1;
        const Texture3D = 1 << 2;
        const TextureCube = 1 << 3;
        const Unknown4 = 1 << 4;
        const Unknown5 = 1 << 5;
        const TextureArray = 1 << 6;
        const Unknown7 = 1 << 7;
    }
}

#[derive(Debug)]
pub struct TexturePayload {
    pub mip_level: u32,
    pub mip_level_count: u32,
    pub data: Vec<u8>,
}

impl TexturePayload {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mip_level = input.read_u32::<LittleEndian>()?;
        let mip_level_count = input.read_u32::<LittleEndian>()?;
        let size = input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        let mut data = vec![0; size as usize];
        input.read_exact(&mut data)?;

        Ok(Self {
            mip_level,
            mip_level_count,
            data,
        })
    }
}
