use anyhow::Result;
use std::io::{Read, Seek};
use byteorder::{LittleEndian, ReadBytesExt};

pub struct ChunkedData {
    pub id: u32,
    pub chunks: Vec<ChunkedDataChunk>,
}

pub struct ChunkedDataChunk {
    pub id: u32,
    pub tag: u16,
    pub version: u16,
    pub data: Vec<u8>,
}

impl ChunkedData {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u32::<LittleEndian>()? != 0xF123456F {}
        let id = input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        let mut chunks = Vec::new();
        while let Ok(chunk_id) = input.read_u32::<LittleEndian>() {
            let chunk_size = input.read_u32::<LittleEndian>()?;
            let chunk_data_size = input.read_u32::<LittleEndian>()?;
            let chunk_tag = input.read_u16::<LittleEndian>()?;
            let chunk_version = input.read_u16::<LittleEndian>()?;
            let mut chunk_data = vec![0; chunk_data_size as usize];
            input.read_exact(&mut chunk_data)?;

            chunks.push(ChunkedDataChunk {
                id: chunk_id,
                tag: chunk_tag,
                version: chunk_version,
                data: chunk_data,
            })
        }

        Ok(Self {
            id,
            chunks,
        })
    }
}


