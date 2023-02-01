use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TextureHeader {
    pub flags: u16,
    pub mip_level_count: u8,
    pub format: u8,
    pub depth_or_array_layers: u8,
    pub u8_0: u8,
    pub payload_count: u8,
    pub u8_1: u8,
    pub width: u16,
    pub height: u16,
    pub size: u32,
    pub u16_0: u16,
    pub u16_1: u16,
    pub u32_0: u32,
    pub u32_1: u32,
    pub u32_2: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TexturePayloadHeader {
    pub mip_level: u32,
    pub mip_level_count: u32,
    pub size: u32,
    pub u32_0: u32,
}

bitflags! {
    #[derive(Debug)]
    pub struct TextureFlags: u16 {
        const D1 = 1 << 0;
        const D2 = 1 << 1;
        const D3 = 1 << 2;
        const Cubemap = 1 << 3;
        const Unknown4 = 1 << 4;
        const Color = 1 << 5;
        const Array = 1 << 6;
        const Unknown7 = 1 << 7;
    }
}
