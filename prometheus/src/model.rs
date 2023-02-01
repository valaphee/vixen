use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub struct ModelChunk {
    bounding_box: [f32; 16],
    materials: Vec<u64>
}

impl ModelChunk {
    pub fn read_from<R: Read + Seek>(input: &mut R) -> Result<Self> {
        input.read_u64::<LittleEndian>()?;
        let material_offset = input.read_u64::<LittleEndian>()?;
        let bounding_box = [
            input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?,
            input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?,
            input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?,
            input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?,
            input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?,
            input.read_f32::<LittleEndian>()?,
        ];
        input.read_u64::<LittleEndian>()?;
        input.read_u64::<LittleEndian>()?;
        let material_count = input.read_u16::<LittleEndian>()?;
        input.seek(SeekFrom::Start(material_offset))?;
        let mut materials = Vec::with_capacity(material_count as usize);
        for _ in 0..material_count {
            materials.push(input.read_u64::<LittleEndian>()?);
        }

        Ok(Self {
            bounding_box,
            materials,
        })
    }
}

pub struct ModelRenderMesh;

impl ModelRenderMesh {
    pub fn read_from<R: Read + Seek>(input: &mut R) -> Result<Self> {
        let vertex_buffer_descriptor_offset = input.read_u64::<LittleEndian>()?;
        let index_buffer_descriptor_offset = input.read_u64::<LittleEndian>()?;
        let submesh_descriptor_offset = input.read_u64::<LittleEndian>()?;
        let vertex_count = input.read_u32::<LittleEndian>()?;
        let submesh_count = input.read_u32::<LittleEndian>()?;
        let material_count = input.read_u16::<LittleEndian>()?;
        let vertex_buffer_descriptor_count = input.read_u8()?;
        let index_buffer_descriptor_count = input.read_u8()?;
        input.read_u32::<LittleEndian>()?;

        // submesh

        input.seek(SeekFrom::Start(vertex_buffer_descriptor_offset))?;
        let mut vertex_buffer_descriptors = Vec::with_capacity(vertex_buffer_descriptor_count as usize);
        for _ in 0..vertex_buffer_descriptor_count {
            vertex_buffer_descriptors.push(VertexBufferDescriptor::read_from(input)?);
        }

        input.seek(SeekFrom::Start(index_buffer_descriptor_offset))?;
        let mut index_buffer_descriptors = Vec::with_capacity(index_buffer_descriptor_count as usize);
        for _ in 0..index_buffer_descriptor_count {
            index_buffer_descriptors.push(IndexBufferDescriptor::read_from(input)?);
        }

        for vertex_buffer_descriptor in vertex_buffer_descriptors {
            input.seek(SeekFrom::Start(vertex_buffer_descriptor.vertex_element_descriptor_offset))?;
            let mut vertex_element_descriptors = Vec::with_capacity(vertex_buffer_descriptor.vertex_element_descriptor_count as usize);
            for _ in 0..vertex_buffer_descriptor.vertex_element_descriptor_count {
                vertex_element_descriptors.push(VertexElementDescriptor::read_from(input)?);
            }
        }

        Ok(Self {})
    }
}

struct SubmeshDescriptor {

}

impl SubmeshDescriptor {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        input.read_u64::<LittleEndian>()?;
        input.read_u64::<LittleEndian>()?;
        input.read_f32::<LittleEndian>()?;
        let vertex_start = input.read_u32::<LittleEndian>()?;
        let index_start = input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        let index_count = input.read_u32::<LittleEndian>()?;
        let indices_to_draw = input.read_u32::<LittleEndian>()?;
        let vertices_to_draw = input.read_u16::<LittleEndian>()?;
        input.read_u16::<LittleEndian>()?;
        let index_buffer = input.read_u8()?;
        input.read_u8()?;
        input.read_u8()?;
        input.read_u8()?;
        input.read_u8()?;
        input.read_u8()?;
        input.read_u8()?;
        input.read_u8()?;
        let vertex_buffer = input.read_u8()?;


        Ok(Self {})
    }
}

struct VertexBufferDescriptor {
    vertex_count: u32,
    vertex_element_descriptor_count: u8,
    vertex_element_descriptor_offset: u64,
}

impl VertexBufferDescriptor {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let vertex_count = input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u8()?;
        input.read_u8()?;
        let vertex_element_descriptor_count = input.read_u8()?;
        input.read_u8()?;
        input.read_u32::<LittleEndian>()?;
        let vertex_element_descriptor_offset = input.read_u64::<LittleEndian>()?;
        input.read_u64::<LittleEndian>()?;
        input.read_u64::<LittleEndian>()?;

        Ok(VertexBufferDescriptor {
            vertex_count,
            vertex_element_descriptor_count,
            vertex_element_descriptor_offset,
        })
    }
}

struct VertexElementDescriptor {
    type_: u8,
    index: u8,
    format: u8,
    slot: u8,
}

impl VertexElementDescriptor {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let type_ = input.read_u8()?;
        let index = input.read_u8()?;
        let format = input.read_u8()?;
        let slot = input.read_u8()?;
        input.read_u32::<LittleEndian>()?;

        Ok(VertexElementDescriptor {
            type_,
            index,
            format,
            slot,
        })
    }
}

struct IndexBufferDescriptor {
    index_count: u32,
    format: u32,
}

impl IndexBufferDescriptor {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let index_count = input.read_u32::<LittleEndian>()?;
        let format = input.read_u32::<LittleEndian>()?;
        input.read_u64::<LittleEndian>()?;

        Ok(IndexBufferDescriptor {
            index_count,
            format,
        })
    }
}
