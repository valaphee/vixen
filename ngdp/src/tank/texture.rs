use anyhow::Result;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

#[derive(Debug)]
pub struct Texture {
    pub flags: TextureFlags,
    pub mip_level_count: u8,
    pub format: u8,
    pub surface_count: u8,
    pub usage_category: u8,
    pub payload_count: u8,
    pub width: u16,
    pub height: u16,
    pub payload: Option<TexturePayload>,
}

impl Texture {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let flags = TextureFlags::from_bits(input.read_u8()?).unwrap();
        input.read_u8()?;
        let mip_level_count = input.read_u8()?;
        let format = input.read_u8()?;
        let surface_count = input.read_u8()?;
        let usage_category = input.read_u8()?;
        let payload_count = input.read_u8()?;
        input.read_u8()?;
        let width = input.read_u16::<LittleEndian>()?;
        let height = input.read_u16::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        let first_payload = if payload_count != 0 {
            Some(TexturePayload::read_from(input)?)
        } else {
            None
        };

        Ok(Self {
            flags,
            mip_level_count,
            format,
            surface_count,
            usage_category,
            payload_count,
            width,
            height,
            payload: first_payload,
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
