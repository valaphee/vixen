use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom};
use thiserror::Error;

#[derive(Default)]
pub struct PaaLoader;

impl AssetLoader for PaaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { load_paa(bytes, load_context).await })
    }

    fn extensions(&self) -> &[&str] {
        &["paa"]
    }
}

#[derive(Error, Debug)]
enum PaaError {
    #[error("unknown tag: {0}")]
    UnknownTag(String),
}

async fn load_paa<'a, 'b>(bytes: &'a [u8], load_context: &'a mut LoadContext<'b>) -> Result<()> {
    let file = Paa::read_from(&mut Cursor::new(bytes.to_vec()))?;

    let mut image = Image::default();
    image.texture_descriptor.format = match file.kind {
        PaaKind::Dxt1 => TextureFormat::Bc1RgbaUnorm,
        PaaKind::Dxt5 => TextureFormat::Bc3RgbaUnorm,
    };
    image.texture_descriptor.mip_level_count = file.mipmaps.len() as u32;
    {
        let PaaMipmap {
            width,
            height,
            data: _,
        } = file.mipmaps.first().unwrap();
        image.texture_descriptor.size = Extent3d {
            width: *width as u32,
            height: *height as u32,
            depth_or_array_layers: 1,
        };
    }
    let mut data: Vec<u8> = Vec::new();
    for mipmap in file.mipmaps {
        data.extend(mipmap.data);
    }
    image.data = data;
    load_context.set_default_asset(LoadedAsset::new(image));

    Ok(())
}

#[derive(Debug)]
struct Paa {
    kind: PaaKind,
    tags: Vec<PaaTag>,
    palette: Vec<u32>,
    mipmaps: Vec<PaaMipmap>,
}

impl Paa {
    fn read_from<R: Read + Seek>(input: &mut R) -> Result<Paa> {
        let kind = PaaKind::read_from(input)?;

        let mut tags = Vec::new();
        let mut position = input.stream_position()?;
        while let Ok(tag) = PaaTag::read_from(input) {
            tags.push(tag);
            position = input.stream_position()?;
        }
        input.seek(SeekFrom::Start(position))?;

        let palette_size = input.read_u16::<LittleEndian>()?;
        let mut palette = Vec::new();
        for _ in 0..palette_size {
            palette.push(input.read_u24::<LittleEndian>()?);
        }

        let mut mipmaps = Vec::new();
        while let Ok(mipmap) = PaaMipmap::read_from(input, &kind) {
            mipmaps.push(mipmap);
        }

        Ok(Self {
            kind,
            tags,
            palette,
            mipmaps,
        })
    }
}

#[derive(Debug)]
enum PaaKind {
    Dxt1,
    Dxt5,
}

impl PaaKind {
    fn read_from<R: Read>(input: &mut R) -> Result<PaaKind> {
        Ok(match input.read_u16::<LittleEndian>()? {
            0xFF01 => Self::Dxt1,
            0xFF05 => Self::Dxt5,
            _ => todo!(),
        })
    }

    fn size(&self, width: usize, height: usize) -> usize {
        match self {
            Self::Dxt1 => ((width + 3) & !3) * ((height + 3) & !3) / 2,
            Self::Dxt5 => ((width + 3) & !3) * ((height + 3) & !3),
        }
    }
}

#[derive(Debug)]
enum PaaTag {
    AverageColor(u32),
    MaximumColor(u32),
    Swizzle(u32),
    Offsets([u32; 16]),
}

impl PaaTag {
    fn read_from<R: Read>(input: &mut R) -> Result<PaaTag> {
        let mut name_bytes = Vec::new();
        name_bytes.resize(8, 0);
        input.read_exact(&mut name_bytes)?;
        let name: String = name_bytes.iter().rev().map(|byte| *byte as char).collect();
        let length = input.read_u32::<LittleEndian>()?;

        Ok(match name.as_str() {
            "AVGCTAGG" => PaaTag::AverageColor(input.read_u32::<LittleEndian>()?),
            "MAXCTAGG" => PaaTag::MaximumColor(input.read_u32::<LittleEndian>()?),
            "SWIZTAGG" => PaaTag::Swizzle(input.read_u32::<LittleEndian>()?),
            "OFFSTAGG" => PaaTag::Offsets(core::array::from_fn(|_| {
                input.read_u32::<LittleEndian>().unwrap()
            })),
            _ => bail!(PaaError::UnknownTag(name)),
        })
    }
}

#[derive(Debug)]
struct PaaMipmap {
    width: u16,
    height: u16,
    data: Vec<u8>,
}

impl PaaMipmap {
    fn read_from<R: Read>(input: &mut R, kind: &PaaKind) -> Result<Self> {
        let mut width = input.read_u16::<LittleEndian>()?;
        let mut height = input.read_u16::<LittleEndian>()?;
        let size = input.read_u24::<LittleEndian>()?;
        let mut data = Vec::new();
        data.resize(size as usize, 0);
        input.read_exact(&mut data)?;

        let data = /*if width == 1234 && height == 8765 {
            width = input.read_u16::<LittleEndian>()?;
            height = input.read_u16::<LittleEndian>()?;

            todo!()
        } else */if width & 0x8000 != 0 {
            width &= 0x7FFF;

            minilzo::decompress(&data, kind.size(width as usize, height as usize))?
        } else {
            data
        };

        Ok(Self {
            width,
            height,
            data,
        })
    }
}
