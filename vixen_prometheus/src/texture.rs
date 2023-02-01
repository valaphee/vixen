use std::path::PathBuf;

use anyhow::Result;
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use prometheus::guid::Guid;
use prometheus::texture::{TextureFlags, TextureHeader, TexturePayloadHeader};

#[derive(Default)]
pub struct TeTextureLoader;

impl AssetLoader for TeTextureLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { load_texture(bytes, load_context).await })
    }

    fn extensions(&self) -> &[&str] {
        &["004"]
    }
}

async fn load_texture<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<()> {
    let (header_bytes, data_) = bytes.split_at(std::mem::size_of::<TextureHeader>());
    let header: &TextureHeader = bytemuck::from_bytes(header_bytes);

    let mut image = Image::default();
    image.texture_descriptor.size = Extent3d {
        width: header.width as u32,
        height: header.height as u32,
        depth_or_array_layers: /*header.depth_or_array_layers as u32*/1,
    };
    image.texture_descriptor.mip_level_count = header.mip_level_count as u32;

    let flags = TextureFlags::from_bits(header.flags).unwrap();
    image.texture_descriptor.dimension = if flags.contains(TextureFlags::D1) {
        TextureDimension::D1
    } else if flags.contains(TextureFlags::D2) {
        TextureDimension::D2
    } else if flags.contains(TextureFlags::D3) {
        TextureDimension::D3
    } else if flags.contains(TextureFlags::Cubemap) {
        TextureDimension::D2
    } else {
        todo!()
    };

    image.texture_descriptor.format = match header.format {
        10 => TextureFormat::Rgba16Float,
        29 => TextureFormat::Rgba8UnormSrgb,
        30 => TextureFormat::Rgba8UnormSrgb,
        72 => TextureFormat::Bc1RgbaUnorm,
        73 => TextureFormat::Bc1RgbaUnormSrgb,
        81 => TextureFormat::Bc4RUnorm,
        84 => TextureFormat::Bc5RgUnorm,
        96 => TextureFormat::Bc6hRgbUfloat,
        99 => TextureFormat::Bc7RgbaUnorm,
        100 => TextureFormat::Bc7RgbaUnormSrgb,
        _ => todo!(),
    };

    let mut data = Vec::with_capacity(header.size as usize);
    if header.payload_count != 0 {
        let mut guid = Guid::from(
            load_context
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        );
        for payload_id in (1..header.payload_count).rev() {
            guid.type_ = 0x04D;
            guid.locale = payload_id;

            let payload_bytes = load_context
                .read_asset_bytes(PathBuf::from(guid.to_raw().to_string()))
                .await?;
            let (payload_header_bytes, payload_data) =
                payload_bytes.split_at(std::mem::size_of::<TexturePayloadHeader>());
            let payload_header: &TexturePayloadHeader = bytemuck::from_bytes(payload_header_bytes);
            data.extend_from_slice(&payload_data[..payload_header.size as usize])
        }

        let (payload_header_bytes, payload_data) =
            data_.split_at(std::mem::size_of::<TexturePayloadHeader>());
        let payload_header: &TexturePayloadHeader = bytemuck::from_bytes(payload_header_bytes);
        data.extend_from_slice(&payload_data[..payload_header.size as usize])
    } else {
        data.extend_from_slice(&data_[..header.size as usize])
    }
    image.data = data;

    load_context.set_default_asset(LoadedAsset::new(image));

    Ok(())
}
