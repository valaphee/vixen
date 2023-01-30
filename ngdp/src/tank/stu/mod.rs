use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

pub mod de;
pub mod error;

#[derive(Debug)]
struct Instance {
    hash: u32,
    parent_hash: u32,
    unknown8: u32,
    size: u32,
}

impl Instance {
    fn read_from<R: Read>(input: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            hash: input.read_u32::<LittleEndian>()?,
            parent_hash: input.read_u32::<LittleEndian>()?,
            unknown8: input.read_u32::<LittleEndian>()?,
            size: input.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug)]
struct InlineArray {
    type_hash: u32,
    size: u32,
}

impl InlineArray {
    fn read_from<R: Read>(input: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            type_hash: input.read_u32::<LittleEndian>()?,
            size: input.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Clone)]
struct Field {
    hash: u32,
    size: u32,
}

impl Field {
    fn read_from<R: Read>(input: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            hash: input.read_u32::<LittleEndian>()?,
            size: input.read_u32::<LittleEndian>()?,
        })
    }
}

fn read_bag<R: Read + Seek, T, F: Fn(&mut R) -> std::io::Result<T>>(
    input: &mut R,
    consume: F,
) -> std::io::Result<Vec<T>> {
    let size = input.read_u32::<LittleEndian>()?;
    let offset = input.read_u32::<LittleEndian>()?;

    let position = input.stream_position()?;
    let mut value = Vec::with_capacity(size as usize);
    input.seek(SeekFrom::Start(offset as u64))?;
    for _ in 0..size {
        value.push(consume(input)?);
    }
    input.seek(SeekFrom::Start(position))?;

    Ok(value)
}
