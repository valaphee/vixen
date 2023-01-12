use anyhow::Result;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use bincode::{
    config::legacy,
    de::{read::Reader, Decoder},
    error::DecodeError,
    impl_borrow_decode, Decode,
};

#[derive(Default)]
pub struct P3dLoader;

impl AssetLoader for P3dLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { load_mlod(bytes, load_context).await })
    }

    fn extensions(&self) -> &[&str] {
        &["p3d"]
    }
}

async fn load_mlod<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<()> {
    let file: Mlod = bincode::decode_from_slice(bytes, legacy().skip_fixed_array_length())?.0;

    for (i, model) in file.lods.iter().enumerate() {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();

        let mut indices = Vec::new();
        let mut index = 0;

        for face in &model.faces {
            // Add indices (CCW winding order)
            match face.kind {
                3 => {
                    indices.push(index + 2);
                    indices.push(index + 1);
                    indices.push(index + 0);
                    index += 3;
                }
                4 => {
                    indices.push(index + 3);
                    indices.push(index + 2);
                    indices.push(index + 1);
                    indices.push(index + 1);
                    indices.push(index + 0);
                    indices.push(index + 3);
                    index += 4;
                }
                _ => {},
            }

            // Add vertices
            for vertex in &face.vertices[..face.kind as usize] {
                positions.push(model.points.get((vertex.point_index) as usize).unwrap().position);
                let normal = model.normals.get((vertex.normal_index) as usize).unwrap();
                normals.push([-normal[0], -normal[1], -normal[2]]);
                uvs.push(vertex.uv);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        load_context.set_labeled_asset(i.to_string().as_str(), LoadedAsset::new(mesh));
    }

    Ok(())
}

#[derive(Debug)]
pub struct Asciiz(String);

impl Decode for Asciiz {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let mut string = Vec::new();
        loop {
            let value: u8 = Decode::decode(decoder)?;
            if value == 0u8 {
                break;
            }
            string.push(value);
        }

        Ok(Asciiz(std::str::from_utf8(&string).unwrap().to_string()))
    }
}
impl_borrow_decode!(Asciiz);

#[derive(Debug)]
struct Mlod {
    version: u32,
    lods: Vec<P3dm>,
}

impl Decode for Mlod {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let _mlod: [u8; 4] = Decode::decode(decoder)?;

        Ok(Self {
            version: Decode::decode(decoder)?,
            lods: {
                let lod_count: u32 = Decode::decode(decoder)?;
                let mut lods = Vec::with_capacity(lod_count as usize);
                for _ in 0..lod_count {
                    lods.push(Decode::decode(decoder)?);
                }
                lods
            },
        })
    }
}
impl_borrow_decode!(Mlod);

#[derive(Debug)]
struct P3dm {
    major_version: u32,
    minor_version: u32,
    flags: u32,
    points: Vec<P3dModelPoint>,
    normals: Vec<[f32; 3]>,
    faces: Vec<P3dModelFace>,
    tags: Vec<P3dModelTag>,
    resolution: f32,
}

impl Decode for P3dm {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let _p3dm: [u8; 4] = Decode::decode(decoder)?;
        let major_version: u32 = Decode::decode(decoder)?;
        let minor_version: u32 = Decode::decode(decoder)?;
        let point_count: u32 = Decode::decode(decoder)?;
        let normal_count: u32 = Decode::decode(decoder)?;
        let face_count: u32 = Decode::decode(decoder)?;
        let flags: u32 = Decode::decode(decoder)?;
        let mut points = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            points.push(Decode::decode(decoder)?);
        }
        let mut normals = Vec::with_capacity(normal_count as usize);
        for _ in 0..normal_count {
            normals.push(Decode::decode(decoder)?);
        }
        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(Decode::decode(decoder)?);
        }

        let _tagg: [u8; 4] = Decode::decode(decoder)?;
        let mut tags = Vec::new();
        loop {
            let tag: P3dModelTag = Decode::decode(decoder)?;
            if tag.name.0 == "#EndOfFile#" {
                break;
            }
            tags.push(tag);
        }

        let resolution: f32 = Decode::decode(decoder)?;

        Ok(Self {
            major_version,
            minor_version,
            flags,
            points,
            normals,
            faces,
            tags,
            resolution,
        })
    }
}
impl_borrow_decode!(P3dm);

#[derive(Debug, Decode)]
struct P3dModelPoint {
    position: [f32; 3],
    flags: u32,
}

#[derive(Debug, Decode)]
struct P3dModelFace {
    kind: u32,
    vertices: [P3dModelVertex; 4],
    flags: u32,
    texture_name: Asciiz,
    material_name: Asciiz,
}

#[derive(Debug, Decode)]
struct P3dModelVertex {
    point_index: u32,
    normal_index: u32,
    uv: [f32; 2],
}

#[derive(Debug)]
struct P3dModelTag {
    active: bool,
    name: Asciiz,
    data: Vec<u8>,
}

impl Decode for P3dModelTag {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            active: Decode::decode(decoder)?,
            name: Decode::decode(decoder)?,
            data: {
                let length = u32::decode(decoder)? as usize;
                let reader = decoder.reader();
                let value = reader.peek_read(length).unwrap().to_vec();
                reader.consume(length);
                value
            },
        })
    }
}
impl_borrow_decode!(P3dModelTag);

#[derive(Debug)]
struct Odol {
    version: u32,
    index: u32,
    mem_lod_sphere: f32,
    geo_lod_sphere: f32,
    point_flags: [u32; 3],
    offset: [f32; 3],
    map_color: u32,
    map_color_2: u32,
    view_density: f32,
    aabb_min: [f32; 3],
    aabb_max: [f32; 3],
    center_of_gravity: [f32; 3],
    offset_2: [f32; 3],
    cog_offset: [f32; 3],
    materials: Vec<LodMaterial>,
}
impl_borrow_decode!(Odol);

impl Decode for Odol {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let _odol: [u8; 4] = Decode::decode(decoder)?;
        let version: u32 = Decode::decode(decoder)?;
        decoder.reader().consume(5);
        let lod_count: u32 = Decode::decode(decoder)?;
        let mut lod_resolutions: Vec<f32> = Vec::with_capacity(lod_count as usize);
        for _ in 0..lod_count {
            lod_resolutions.push(Decode::decode(decoder)?);
        }
        let index: u32 = Decode::decode(decoder)?;
        let mem_lod_sphere: f32 = Decode::decode(decoder)?;
        let geo_lod_sphere: f32 = Decode::decode(decoder)?;
        let point_flags: [u32; 3] = Decode::decode(decoder)?;
        let offset: [f32; 3] = Decode::decode(decoder)?;
        let map_color: u32 = Decode::decode(decoder)?;
        let map_color_2: u32 = Decode::decode(decoder)?;
        let view_density: f32 = Decode::decode(decoder)?;
        let aabb_min: [f32; 3] = Decode::decode(decoder)?;
        let aabb_max: [f32; 3] = Decode::decode(decoder)?;
        let center_of_gravity: [f32; 3] = Decode::decode(decoder)?;
        let offset_2: [f32; 3] = Decode::decode(decoder)?;
        let cog_offset: [f32; 3] = Decode::decode(decoder)?;
        decoder.reader().consume(196 + 1);
        let mut lod_start_addresses: Vec<u32> = Vec::with_capacity(lod_count as usize);
        for _ in 0..lod_count {
            lod_start_addresses.push(Decode::decode(decoder)?);
        }
        let mut lod_end_addresses: Vec<u32> = Vec::with_capacity(lod_count as usize);
        for _ in 0..lod_count {
            lod_end_addresses.push(Decode::decode(decoder)?);
        }
        decoder.reader().consume(28 + 68);
        let texture_count: u32 = Decode::decode(decoder)?;
        let mut textures: Vec<Asciiz> = Vec::with_capacity(texture_count as usize);
        for _ in 0..texture_count {
            textures.push(Decode::decode(decoder)?);
        }
        decoder.reader().consume(304);
        let texture_count: u32 = Decode::decode(decoder)?;
        let mut textures: Vec<Asciiz> = Vec::with_capacity(texture_count as usize);
        for _ in 0..texture_count {
            textures.push(Decode::decode(decoder)?);
        }
        let material_count: u32 = Decode::decode(decoder)?;
        let mut materials: Vec<LodMaterial> = Vec::with_capacity(material_count as usize);
        for _ in 0..material_count {
            materials.push(Decode::decode(decoder)?);
        }

        Ok(Self {
            version,
            index,
            mem_lod_sphere,
            geo_lod_sphere,
            point_flags,
            offset,
            map_color,
            map_color_2,
            view_density,
            aabb_min,
            aabb_max,
            center_of_gravity,
            offset_2,
            cog_offset,
            materials
        })
    }
}

#[derive(Debug)]
struct LodMaterial {
    name: Asciiz,
    kind: u32,
    emissive: [f32; 4],
    ambient: [f32; 4],
    diffuse: [f32; 4],
    forced_diffuse: [f32; 4],
    specular: [f32; 4],
    specular_2: [f32; 4],
    specular_power: f32,
    pixel_shader_id: u32,
    vertex_shader_id: u32,
    surface_name: Asciiz,
    render_flags: u32,
    textures: Vec<LodMaterialTexture>,
    texture_transforms: Vec<LodMaterialTextureTransform>,
}
impl_borrow_decode!(LodMaterial);

impl Decode for LodMaterial {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let name: Asciiz = Decode::decode(decoder)?;
        let kind: u32 = Decode::decode(decoder)?;
        let emissive: [f32; 4] = Decode::decode(decoder)?;
        let ambient: [f32; 4] = Decode::decode(decoder)?;
        let diffuse: [f32; 4] = Decode::decode(decoder)?;
        let forced_diffuse: [f32; 4] = Decode::decode(decoder)?;
        let specular: [f32; 4] = Decode::decode(decoder)?;
        let specular_2: [f32; 4] = Decode::decode(decoder)?;
        let specular_power: f32 = Decode::decode(decoder)?;
        let pixel_shader_id: u32 = Decode::decode(decoder)?;
        let vertex_shader_id: u32 = Decode::decode(decoder)?;
        decoder.reader().consume(8);
        let surface_name: Asciiz = Decode::decode(decoder)?;
        decoder.reader().consume(4);
        let render_flags: u32 = Decode::decode(decoder)?;
        let texture_count: u32 = Decode::decode(decoder)?;
        let texture_transform_count: u32 = Decode::decode(decoder)?;
        let mut textures: Vec<LodMaterialTexture> = Vec::with_capacity(texture_count as usize);
        for _ in 0..texture_count {
            textures.push(Decode::decode(decoder)?);
        }
        let mut texture_transforms: Vec<LodMaterialTextureTransform> = Vec::with_capacity(texture_transform_count as usize);
        for _ in 0..texture_transform_count {
            texture_transforms.push(Decode::decode(decoder)?);
        }

        Ok(Self {
            name,
            kind,
            emissive,
            ambient,
            diffuse,
            forced_diffuse,
            specular,
            specular_2,
            specular_power,
            pixel_shader_id,
            vertex_shader_id,
            surface_name,
            render_flags,
            textures,
            texture_transforms,
        })
    }
}

#[derive(Debug, Decode)]
struct LodMaterialTexture {
    filter: u32,
    name: Asciiz,
    transform_index: u32,
    unknown_bool: bool,
}

#[derive(Debug, Decode)]
struct LodMaterialTextureTransform {
    source: u32,
    transform: [[f32; 3]; 4],
}
