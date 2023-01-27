use anyhow::{bail, Result};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use fasthash::lookup3;
use std::io::{Read, Seek, SeekFrom};
use thiserror::Error;

#[derive(Error, Debug)]
enum CascError {
    #[error("invalid size")]
    InvalidSize,
    #[error("unknown block type")]
    UnknownBlockType,
    #[error("unknown version")]
    UnknownVersion,
    #[error("checksum mismatch")]
    ChecksumMismatch,
}

#[derive(Debug)]
struct SharedMemory {
    data_path: String,
    versions: Vec<u32>,
    free_spaces: Vec<Data>,
}

impl SharedMemory {
    fn read_from<R: Read + Seek>(input: &mut R) -> Result<Self> {
        if input.read_u32::<LittleEndian>()? /* Header Type */ != 4 {
            bail!(CascError::UnknownBlockType)
        }
        let header_size = input.read_u32::<LittleEndian>()?;
        let mut path = vec![0; 0x100];
        input.read_exact(&mut path)?;
        let mut versions = Vec::with_capacity(0x10);
        for _ in 0..(header_size - input.stream_position()? as u32 - 0x10 * 4) / (4 * 2) {
            input.read_u32::<LittleEndian>()?;
            input.read_u32::<LittleEndian>()?;
        }
        for _ in 0..0x10 {
            versions.push(input.read_u32::<LittleEndian>()?);
        }

        if input.read_u32::<LittleEndian>()? /* Free Space Type */ != 1 {
            bail!(CascError::UnknownBlockType)
        }
        let free_space_size = input.read_u32::<LittleEndian>()?;
        let mut free_spaces = Vec::with_capacity(free_space_size as usize);
        input.seek(SeekFrom::Current(0x18))?;
        for _ in 0..free_space_size {
            let length = Data::read_from(input, 0, 5, 0, 30)?;
            free_spaces.push(Data {
                key: vec![],
                file: 0,
                offset: 0,
                length: length.offset,
            });
        }
        input.seek(SeekFrom::Current(((1090 - free_space_size) * 5) as i64))?;
        for index in 0..free_space_size {
            let file_offset = Data::read_from(input, 0, 5, 0, 30)?;
            let entry = &mut free_spaces[index as usize];
            entry.file = file_offset.file;
            entry.offset = file_offset.offset;
        }

        Ok(Self {
            data_path: std::str::from_utf8(
                &path[0..path
                    .as_slice()
                    .iter()
                    .position(|&value| value == b'\0')
                    .unwrap_or(path.len())],
            )?
            .to_string(),
            versions,
            free_spaces,
        })
    }
}

#[derive(Debug)]
struct Index {
    bucket: u16,
    entry_length_size: u8,
    entry_location_size: u8,
    entry_key_size: u8,
    entry_segment_bits: u8,
    limit: u64,
    entries: Vec<Data>,
}

impl Index {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut header_data = vec![0; input.read_u32::<LittleEndian>()? as usize];
        let header_hash = input.read_u32::<LittleEndian>()?;
        input.read_exact(&mut header_data)?;
        if header_hash != lookup3::hash32(&header_data) {
            bail!(CascError::ChecksumMismatch)
        }
        let mut header_data = header_data.as_slice();
        if header_data.read_u16::<LittleEndian>()? /* Version */ != 7 {
            bail!(CascError::UnknownVersion)
        }
        let bucket = header_data.read_u16::<LittleEndian>()?;
        let entry_length_size = header_data.read_u8()?;
        let entry_location_size = header_data.read_u8()?;
        let entry_key_size = header_data.read_u8()?;
        let entry_segment_bits = header_data.read_u8()?;
        let limit = header_data.read_u64::<LittleEndian>()?;
        input.read_exact(&mut [0; 0x8])?;

        let mut entries_data = vec![0; input.read_u32::<LittleEndian>()? as usize];
        let entries_hash = input.read_u32::<LittleEndian>()?;
        input.read_exact(&mut entries_data)?;
        /*if entries_hash != lookup3::hash32(&entries_data) {
            bail!(CascError::ChecksumMismatch)
        }*/
        let mut entries_data = entries_data.as_slice();
        let entries_count = entries_data.len()
            / (entry_length_size + entry_location_size + entry_key_size) as usize;
        let mut entries = Vec::with_capacity(entries_count);
        for _ in 0..entries_count {
            entries.push(Data::read_from(
                &mut entries_data,
                entry_length_size,
                entry_location_size,
                entry_key_size,
                entry_segment_bits,
            )?);
        }

        Ok(Self {
            bucket,
            entry_length_size,
            entry_location_size,
            entry_key_size,
            entry_segment_bits,
            limit,
            entries,
        })
    }
}

#[derive(Debug)]
struct Data {
    key: Vec<u8>,
    file: u64,
    offset: u64,
    length: u64,
}

impl Data {
    fn read_from<R: Read>(
        input: &mut R,
        length_size: u8,
        location_size: u8,
        key_size: u8,
        segment_bits: u8,
    ) -> Result<Self> {
        let mut key = vec![0; key_size as usize];
        input.read_exact(&mut key)?;
        let offset_size = (segment_bits + 7) / 8;
        let file_size = location_size - offset_size;
        let mut file = input.read_uint::<BigEndian>(file_size as usize)?;
        let mut offset = input.read_uint::<BigEndian>(offset_size as usize)?;
        let extra_bits = (offset_size * 8) - segment_bits;
        file = (file << extra_bits) | (offset >> segment_bits);
        offset = offset & ((1 << 32 - extra_bits) - 1);
        let length = if length_size == 0 {
            0
        } else {
            input.read_uint::<LittleEndian>(length_size as usize)?
        };

        Ok(Self {
            key,
            file,
            offset,
            length,
        })
    }

    fn read<R: Read + Seek>(&self, input: &mut R) -> Result<()> {
        input.seek(SeekFrom::Start(self.offset))?;
        let mut key = [0; 0x10];
        input.read_exact(&mut key)?;
        if input.read_u32::<LittleEndian>()? != self.length as u32 {
            bail!(CascError::InvalidSize);
        }
        input.read_u16::<LittleEndian>()?;
        {
            let checksum_mark = input.stream_position()?;
            input.seek(SeekFrom::Start(self.offset))?;
            let mut checksum_data = vec![0; (checksum_mark - self.offset) as usize];
            input.read_exact(&mut checksum_data)?;
            if input.read_u32::<LittleEndian>()?
                != lookup3::hash32_with_seed(checksum_data, 0x3D6BE971)
            {
                bail!(CascError::ChecksumMismatch)
            }
        }
        {
            let offset = ((self.offset & 0x3FFFFFFF) | (self.file & 3) << 30) as u32;
            let checksum_offset =
                ((input.stream_position()? & 0x3FFFFFFF) | (self.file & 3) << 30) as u32;
            input.seek(SeekFrom::Start(self.offset))?;
            let mut hashed_data = [0u8; 4];
            for i in offset..checksum_offset {
                hashed_data[(i & 3) as usize] ^= input.read_u8()?;
            }
            let encoded_offset: [u8; 4] = unsafe {
                std::mem::transmute(
                    OFFSET_ENCODE_TABLE[((checksum_offset + 4) & 0xF) as usize]
                        ^ (checksum_offset + 4),
                )
            };
            let checksum: [_; 4] = core::array::from_fn(|i| {
                let j = (i + checksum_offset as usize) & 3;
                hashed_data[j] ^ encoded_offset[j]
            });
            let checksum: u32 = unsafe { std::mem::transmute(checksum) };
            if input.read_u32::<LittleEndian>()? != checksum {
                bail!(CascError::ChecksumMismatch)
            }
        }

        Ok(())
    }
}

static OFFSET_ENCODE_TABLE: &'static [u32] = &[
    0x049396B8, 0x72A82A9B, 0xEE626CCA, 0x9917754F, 0x15DE40B1, 0xF5A8A9B6, 0x421EAC7E, 0xA9D55C9A,
    0x317FD40C, 0x04FAF80D, 0x3D6BE971, 0x52933CFD, 0x27F64B7D, 0xC6F5C11B, 0xD5757E3A, 0x6C388745,
];
