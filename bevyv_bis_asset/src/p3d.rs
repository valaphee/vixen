use anyhow::Result;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

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
    fn read_from<R: Read>(input: &mut R) -> Result<Mlod> {
        if input.read_u32::<LittleEndian>()? != 0x444F4C4D { // Magic "MLOD"
        }
        if input.read_u32::<LittleEndian>()? != 0x101 { // Version
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
    points: Vec<P3dModelPoint>,
    normals: Vec<[f32; 3]>,
    faces: Vec<P3dModelFace>,
    tags: Vec<P3dModelTag>,
    resolution: f32,
}

impl P3dm {
    fn read_from<R: Read>(input: &mut R) -> Result<P3dm> {
        if input.read_u32::<LittleEndian>()? != 0x4D443350 { // Magic "P3DM"
        }
        if input.read_u32::<LittleEndian>()? != 0x1C || input.read_u32::<LittleEndian>()? != 0x100 { // Version
        }

        let point_count = input.read_u32::<LittleEndian>()?;
        let normal_count = input.read_u32::<LittleEndian>()?;
        let face_count = input.read_u32::<LittleEndian>()?;
        let flags = input.read_u32::<LittleEndian>()?;
        let mut points = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            points.push(P3dModelPoint::read_from(input)?);
        }
        let mut normals = Vec::with_capacity(normal_count as usize);
        for _ in 0..normal_count {
            normals.push(core::array::from_fn(|_| {
                input.read_f32::<LittleEndian>().unwrap()
            }));
        }
        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(P3dModelFace::read_from(input)?);
        }

        if input.read_u32::<LittleEndian>()? != 0x47474154 { // Magic "TAGG"
        }
        let mut tags = Vec::new();
        loop {
            let tag: P3dModelTag = P3dModelTag::read_from(input)?;
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
struct P3dModelPoint {
    position: [f32; 3],
    flags: u32,
}

impl P3dModelPoint {
    fn read_from<R: Read>(input: &mut R) -> Result<P3dModelPoint> {
        Ok(Self {
            position: core::array::from_fn(|_| input.read_f32::<LittleEndian>().unwrap()),
            flags: input.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug)]
struct P3dModelFace {
    vertex_count: u32,
    vertices: [P3dModelVertex; 4],
    flags: u32,
    texture_name: String,
    material_name: String,
}

impl P3dModelFace {
    fn read_from<R: Read>(input: &mut R) -> Result<P3dModelFace> {
        Ok(Self {
            vertex_count: input.read_u32::<LittleEndian>()?,
            vertices: core::array::from_fn(|_| P3dModelVertex::read_from(input).unwrap()),
            flags: input.read_u32::<LittleEndian>()?,
            texture_name: read_asciiz(input)?,
            material_name: read_asciiz(input)?,
        })
    }
}

#[derive(Debug)]
struct P3dModelVertex {
    point_index: u32,
    normal_index: u32,
    uv: [f32; 2],
}

impl P3dModelVertex {
    fn read_from<R: Read>(input: &mut R) -> Result<P3dModelVertex> {
        Ok(Self {
            point_index: input.read_u32::<LittleEndian>()?,
            normal_index: input.read_u32::<LittleEndian>()?,
            uv: core::array::from_fn(|_| input.read_f32::<LittleEndian>().unwrap()),
        })
    }
}

#[derive(Debug)]
struct P3dModelTag {
    active: bool,
    name: String,
    data: Vec<u8>,
}

impl P3dModelTag {
    fn read_from<R: Read>(input: &mut R) -> Result<P3dModelTag> {
        Ok(Self {
            active: input.read_u8()? != 0,
            name: read_asciiz(input)?,
            data: {
                let mut data = Vec::new();
                data.resize(input.read_u32::<LittleEndian>()? as usize, 0);
                input.read_exact(&mut data)?;
                data
            },
        })
    }
}

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
