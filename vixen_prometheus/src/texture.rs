use anyhow::Result;
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use prometheus::guid::Guid;
use prometheus::te_texture::{TeTexture, TeTexturePayload, TeTextureFlags};
use std::error::Error;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Default)]
pub struct TeTextureLoader;

impl AssetLoader for TeTextureLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { load_te_texture(bytes, load_context).await })
    }

    fn extensions(&self) -> &[&str] {
        &["004"]
    }
}

async fn load_te_texture<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<()> {
    let te_texture = TeTexture::read_from(&mut Cursor::new(bytes.to_vec()))?;
    let mut te_texture_payloads = Vec::with_capacity(te_texture.payload_count as usize);
    te_texture_payloads.push(te_texture.first_payload.unwrap());
    if te_texture.payload_count > 1 {
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

        for payload_id in 1..te_texture.payload_count {
            guid.type_ = 0x04D;
            guid.locale = payload_id;

            te_texture_payloads.push(TeTexturePayload::read_from(&mut Cursor::new(
                load_context
                    .read_asset_bytes(PathBuf::from(guid.to_raw().to_string()))
                    .await?
                    .to_vec(),
            ))?);
        }
    }

    let mut image = Image::default();
    image.texture_descriptor.size = Extent3d {
        width: te_texture.width as u32,
        height: te_texture.height as u32,
        depth_or_array_layers: te_texture.depth_or_array_layers as u32,
    };
    image.texture_descriptor.mip_level_count = te_texture.mip_level_count as u32;
    image.texture_descriptor.dimension = if te_texture.flags.contains(TeTextureFlags::D1) {
        TextureDimension::D1
    } else if te_texture.flags.contains(TeTextureFlags::D2) {
        TextureDimension::D2
    } else if te_texture.flags.contains(TeTextureFlags::D3) {
        TextureDimension::D3
    } else {
        todo!()
    };
    image.texture_descriptor.format = match te_texture.format {
        10 => TextureFormat::Rgba16Float,
        29 => TextureFormat::Rgba8UnormSrgb,
        30 => TextureFormat::Rgba8UnormSrgb,
        72 => TextureFormat::Bc1RgbaUnorm,
        73 => TextureFormat::Bc1RgbaUnormSrgb,
        81 => TextureFormat::Bc4RUnorm,
        84 => TextureFormat::Bc5RgUnorm,
        99 => TextureFormat::Bc7RgbaUnorm,
        100 => TextureFormat::Bc7RgbaUnormSrgb,
        _ => todo!(),
    };

    let mut data = Vec::new();
    for texture_payload in te_texture_payloads.iter().rev() {
        data.extend(&texture_payload.data);
    }
    image.data = data;

    load_context.set_default_asset(LoadedAsset::new(image));

    Ok(())
}
