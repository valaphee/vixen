use std::io::{Read, Seek, SeekFrom};

use anyhow::Result;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use fasthash::lookup3;

#[derive(Debug)]
struct SharedMemory {
    data_path: String,
    versions: Vec<u32>,
    free_spaces: Vec<Entry>,
}

impl SharedMemory {
    fn read_from<R: Read + Seek>(input: &mut R) -> Result<Self> {
        if input.read_u32::<LittleEndian>()? /* Header Type */ != 4 {}
        let header_size = input.read_u32::<LittleEndian>()?;
        let mut path = vec![0; 0x100];
        path.resize(path.capacity(), 0);
        input.read_exact(&mut path)?;
        for _ in 0..(header_size - input.stream_position()? as u32 - 16 * 4) / (4 * 2) {
            let block_size = input.read_u32::<LittleEndian>()?;
            let block_offset = input.read_u32::<LittleEndian>()?;
        }
        let mut versions = Vec::with_capacity(0x10);
        for _ in 0..16 {
            versions.push(input.read_u32::<LittleEndian>()?);
        }

        if input.read_u32::<LittleEndian>()? /* Free Space Type */ != 1 {}
        let mut free_spaces = Vec::with_capacity(input.read_u32::<LittleEndian>()? as usize);
        input.seek(SeekFrom::Current(0x18))?;
        for _ in 0..free_spaces.capacity() {
            let length = Entry::read_from(input, 0, 5, 0, 30)?;
            free_spaces.push(Entry {
                key: vec![],
                file: 0,
                offset: 0,
                length: length.offset,
            });
        }
        input.seek(SeekFrom::Current(((1090 - free_space_size) * 5) as i64))?;
        for index in 0..free_space_size {
            let file_offset = Entry::read_from(input, 0, 5, 0, 30)?;
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
    entries: Vec<Entry>,
}

impl Index {
    fn read_from<R: Read + Seek>(input: &mut R) -> Result<Self> {
        let mut header_data = vec![0; input.read_u32::<LittleEndian>()? as usize];
        let header_hash = input.read_u32::<LittleEndian>()?;
        input.read_exact(&mut header_data)?;
        if header_hash != lookup3::hash32(&header_data) {}
        let mut header_data = header_data.as_slice();
        if header_data.read_u16::<LittleEndian>()? /* Version */ != 7 {}
        let bucket = header_data.read_u16::<LittleEndian>()?;
        let entry_length_size = header_data.read_u8()?;
        let entry_location_size = header_data.read_u8()?;
        let entry_key_size = header_data.read_u8()?;
        let entry_segment_bits = header_data.read_u8()?;
        let limit = header_data.read_u64::<LittleEndian>()?;
        input.seek(SeekFrom::Current(0x08))?;

        let mut entries_data = vec![0; input.read_u32::<LittleEndian>()? as usize];
        let entries_hash = input.read_u32::<LittleEndian>()?;
        input.read_exact(&mut entries_data)?;
        if entries_hash != lookup3::hash32(&entries_data) {}
        let mut entries_data = entries_data.as_slice();
        let mut entries = Vec::with_capacity((entries_data.len() / (entry_length_size + entry_location_size + entry_key_size) as usize));
        for _ in 0..entries.capacity() {
            entries.push(Entry::read_from(
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
struct Entry {
    key: Vec<u8>,
    file: u64,
    offset: u64,
    length: u64,
}

impl Entry {
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

    fn read<R : Read + Seek>(
        &self,
        input: &mut R
    ) -> Result<()> {
        input.seek(SeekFrom::Start(self.offset))?;
        let mut key = vec![0; 0x10];
        input.read_exact(&mut key)?;
        if input.read_u32::<LittleEndian>()? != self.length as u32 {}
        let flags = input.read_u16::<LittleEndian>()?;
        {
            let checksum_mark = input.stream_position()?;
            input.seek(SeekFrom::Start(self.offset))?;
            let mut checksum_data = vec![0; (checksum_mark - self.offset) as usize];
            input.read_exact(&mut checksum_data)?;
            if input.read_u32::<LittleEndian>()? != lookup3::hash32_with_seed(checksum_data, 0x3D6BE971) {}
        }
        {
            let offset = ((self.offset & 0x3FFFFFFF) | (self.file & 3) << 30) as u32;
            let checksum_offset = ((input.stream_position()? & 0x3FFFFFFF) | (self.file & 3) << 30) as u32;
            input.seek(SeekFrom::Start(self.offset))?;
            let mut hashed_data = [0u8; 4];
            for i in offset..checksum_offset {
                hashed_data[(i & 3) as usize] ^= input.read_u8()?;
            }
            let encoded_offset: [u8; 4] = unsafe { std::mem::transmute(OFFSET_ENCODE_TABLE[((checksum_offset + 4) & 0xF) as usize] ^ (checksum_offset + 4)) };
            let checksum: [_; 4] = core::array::from_fn(|i| {
                let j = (i + checksum_offset as usize) & 3;
                hashed_data[j] ^ encoded_offset[j]
            });
            let checksum: u32 = unsafe { std::mem::transmute(checksum) };
            if input.read_u32::<LittleEndian>()? != checksum {}
        }

        Ok(())
    }
}

static OFFSET_ENCODE_TABLE: &'static [u32] = &[
    0x049396b8, 0x72a82a9b, 0xee626cca, 0x9917754f, 0x15de40b1, 0xf5a8a9b6, 0x421eac7e, 0xa9d55c9a,
    0x317fd40c, 0x04faf80d, 0x3d6be971, 0x52933cfd, 0x27f64b7d, 0xc6f5c11b, 0xd5757e3a, 0x6c388745,
];
