use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom};
use thiserror::Error;

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

#[derive(Error, Debug)]
enum P3dError {
    #[error("invalid magic")]
    InvalidMagic,
    #[error("unknown version: {0}")]
    UnknownVersion(String),
}

async fn load_mlod<'a, 'b>(bytes: &'a [u8], load_context: &'a mut LoadContext<'b>) -> Result<()> {
    let file = Mlod::read_from(&mut Cursor::new(bytes.to_vec()))?;

    for (i, model) in file.0.iter().enumerate() {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();

        let mut indices = Vec::new();
        let mut index = 0;

        for face in &model.faces {
            // Add indices (CCW winding order)
            match face.vertex_count {
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
                _ => {}
            }

            // Add vertices
            for vertex in &face.vertices[..face.vertex_count as usize] {
                positions.push(
                    model
                        .points
                        .get((vertex.point_index) as usize)
                        .unwrap()
                        .position,
                );
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
struct Mlod(Vec<P3dm>);

impl Mlod {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u32::<BigEndian>()? /* MLOD */ != 0x4D4C4F44 {
            bail!(P3dError::InvalidMagic)
        }
        let version = input.read_u32::<LittleEndian>()?;
        if version != 0x101 {
            bail!(P3dError::UnknownVersion(version.to_string()))
        }

        let lod_count = input.read_u32::<LittleEndian>()?;
        let mut lods = Vec::with_capacity(lod_count as usize);
        for _ in 0..lod_count {
            lods.push(P3dm::read_from(input)?);
        }

        Ok(Self(lods))
    }
}

#[derive(Debug)]
struct P3dm {
    flags: u32,
    points: Vec<P3dmPoint>,
    normals: Vec<[f32; 3]>,
    faces: Vec<P3dmFace>,
    tags: Vec<P3dmTag>,
    resolution: f32,
}

impl P3dm {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u32::<BigEndian>()? /* P3DM */ != 0x5033444D {
            bail!(P3dError::InvalidMagic)
        }
        let major_version = input.read_u32::<LittleEndian>()?;
        let minor_version = input.read_u32::<LittleEndian>()?;
        if major_version != 0x1C && minor_version != 0x101 {
            bail!(P3dError::UnknownVersion(format!("{major_version}.{minor_version}")))
        }

        let point_count = input.read_u32::<LittleEndian>()?;
        let normal_count = input.read_u32::<LittleEndian>()?;
        let face_count = input.read_u32::<LittleEndian>()?;
        let flags = input.read_u32::<LittleEndian>()?;
        let mut points = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            points.push(P3dmPoint::read_from(input)?);
        }
        let mut normals = Vec::with_capacity(normal_count as usize);
        for _ in 0..normal_count {
            normals.push(core::array::from_fn(|_| {
                input.read_f32::<LittleEndian>().unwrap()
            }));
        }
        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(P3dmFace::read_from(input)?);
        }

        if input.read_u32::<BigEndian>()? /* TAGG */ != 0x54414747 {
            bail!(P3dError::InvalidMagic)
        }
        let mut tags = Vec::new();
        loop {
            let tag: P3dmTag = P3dmTag::read_from(input)?;
            if tag.name == "#EndOfFile#" {
                break;
            }
            tags.push(tag);
        }

        let resolution = input.read_f32::<LittleEndian>()?;

        Ok(Self {
            flags,
            points,
            normals,
            faces,
            tags,
            resolution,
        })
    }
}

#[derive(Debug)]
struct P3dmPoint {
    position: [f32; 3],
    flags: u32,
}

impl P3dmPoint {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            position: core::array::from_fn(|_| input.read_f32::<LittleEndian>().unwrap()),
            flags: input.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug)]
struct P3dmFace {
    vertex_count: u32,
    vertices: [P3dmVertex; 4],
    flags: u32,
    texture_name: String,
    material_name: String,
}

impl P3dmFace {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            vertex_count: input.read_u32::<LittleEndian>()?,
            vertices: core::array::from_fn(|_| P3dmVertex::read_from(input).unwrap()),
            flags: input.read_u32::<LittleEndian>()?,
            texture_name: read_asciiz(input)?,
            material_name: read_asciiz(input)?,
        })
    }
}

#[derive(Debug)]
struct P3dmVertex {
    point_index: u32,
    normal_index: u32,
    uv: [f32; 2],
}

impl P3dmVertex {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            point_index: input.read_u32::<LittleEndian>()?,
            normal_index: input.read_u32::<LittleEndian>()?,
            uv: core::array::from_fn(|_| input.read_f32::<LittleEndian>().unwrap()),
        })
    }
}

#[derive(Debug)]
struct P3dmTag {
    active: bool,
    name: String,
    data: Vec<u8>,
}

impl P3dmTag {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            active: input.read_u8()? != 0,
            name: read_asciiz(input)?,
            data: {
                let mut data = vec![0; input.read_u32::<LittleEndian>()? as usize];
                input.read_exact(&mut data)?;
                data
            },
        })
    }
}

struct Odol {
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
}

impl Odol {
    fn read_from<R: Read + Seek>(input: &mut R) -> Result<Self> {
        if input.read_u32::<BigEndian>()? /* ODOL */ != 0x4F444F4C {
            bail!(P3dError::InvalidMagic)
        }
        let version = input.read_u32::<LittleEndian>()?;
        input.seek(SeekFrom::Current(5))?;
        let lod_count = input.read_u32::<LittleEndian>()?;
        let mut lod_resolutions = Vec::with_capacity(lod_count as usize);
        for _ in 0..lod_count {
            lod_resolutions.push(input.read_f32::<LittleEndian>()?);
        }
        let index = input.read_u32()?;
        let mem_lod_sphere = input.read_f32()?;
        let geo_lod_sphere = input.read_f32()?;
        let point_flags: [_; 3] = core::array::from_fn(|_| input.read_u32::<LittleEndian>()?);
        let offset: [_; 3] = core::array::from_fn(|_| input.read_f32::<LittleEndian>()?);
        let map_color = input.read_u32::<LittleEndian>()?;
        let map_color_2 = input.read_u32::<LittleEndian>()?;
        let view_density = input.read_f32::<LittleEndian>()?;
        let aabb_min: [_; 3] = core::array::from_fn(|_| input.read_f32::<LittleEndian>()?);
        let aabb_max: [_; 3] = core::array::from_fn(|_| input.read_f32::<LittleEndian>()?);
        let center_of_gravity: [_; 3] = core::array::from_fn(|_| input.read_f32::<LittleEndian>()?);
        let offset_2: [_; 3] = core::array::from_fn(|_| input.read_f32::<LittleEndian>()?);
        let cog_offset: [_; 3] = core::array::from_fn(|_| input.read_f32::<LittleEndian>()?);
        input.seek(SeekFrom::Current(196 + 1))?;
        let mut lod_start_addresses = Vec::with_capacity(lod_count as usize);
        for _ in 0..lod_count {
            lod_start_addresses.push(input.read_u32::<LittleEndian>()?);
        }
        let mut lod_end_addresses = Vec::with_capacity(lod_count as usize);
        for _ in 0..lod_count {
            lod_end_addresses.push(input.read_u32::<LittleEndian>()?);
        }

        Ok(Self {
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
        })
    }
}

struct OdolLod {

}

impl OdolLod {
    fn read_from<R: Read + Seek>(input: &mut R) -> Result<Self> {
        Ok(OdolLod {

        })
    }
}

struct OdolLodMaterial {

}

impl OdolLodMaterial {
    fn read_from<R: Read + Seek>(input: &mut R) -> Result<Self> {
        Ok(OdolLodMaterial {

        })
    }
}

#[inline]
fn read_asciiz<R: Read>(input: &mut R) -> Result<String> {
    let mut data = Vec::new();
    loop {
        let value = input.read_u8()?;
        if value == 0u8 {
            break;
        }
        data.push(value as char);
    }

    Ok(data.iter().collect())
}
