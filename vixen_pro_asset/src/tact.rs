use std::io::Read;
use anyhow::{bail, Result};
use byteorder::{BigEndian, ReadBytesExt};
use md5::{Digest, Md5};
use thiserror::Error;

#[derive(Error, Debug)]
enum TactError {
    #[error("invalid magic")]
    InvalidMagic,
    #[error("unknown version")]
    UnknownVersion,
    #[error("checksum mismatch")]
    ChecksumMismatch,
}

#[derive(Debug)]
pub struct Encoding {
    pub c_to_e_key_page_table: Vec<CToEKeyPage>,
    e_key_spec_page_table: Vec<EKeySpecPage>,
}

impl Encoding {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u16::<BigEndian>()? /* EN */ != 0x454E {
            bail!(TactError::InvalidMagic);
        }
        if input.read_u8()? != 1 {
            bail!(TactError::UnknownVersion);
        }
        let c_key_size = input.read_u8()?;
        let e_key_size = input.read_u8()?;
        let c_to_e_key_page_size = input.read_u16::<BigEndian>()?;
        let e_key_spec_page_size = input.read_u16::<BigEndian>()?;
        let c_to_e_key_page_count = input.read_u32::<BigEndian>()?;
        let e_key_spec_page_count = input.read_u32::<BigEndian>()?;
        if input.read_u8()? != 0 {
            bail!(TactError::UnknownVersion);
        }

        let mut e_spec_block = vec![0; input.read_u32::<BigEndian>()? as usize];
        input.read_exact(&mut e_spec_block)?;
        let mut e_specs_data = e_spec_block.as_slice();
        let mut e_specs = Vec::new();
        while let Ok(e_spec) = read_asciiz(&mut e_specs_data) {
            e_specs.push(e_spec);
        }

        let mut c_to_e_key_page_table = Vec::with_capacity(c_to_e_key_page_count as usize);
        for _ in 0..c_to_e_key_page_table.capacity() {
            c_to_e_key_page_table.push(CToEKeyPage {
                first_c_key: {
                    let mut first_c_key = vec![0; c_key_size as usize];
                    input.read_exact(&mut first_c_key)?;
                    first_c_key
                },
                md5: {
                    let mut checksum = [0; 0x10];
                    input.read_exact(&mut checksum)?;
                    checksum
                },
                c_to_e_keys: vec![],
            });
        }
        for c_to_e_key_page in &mut c_to_e_key_page_table {
            let mut c_to_e_key_page_data = vec![0; c_to_e_key_page_size as usize * 1024];
            input.read_exact(&mut c_to_e_key_page_data)?;
            let mut md5 = Md5::new();
            md5.update(&c_to_e_key_page_data);
            if c_to_e_key_page.md5 != md5.finalize().as_slice() {
                bail!(TactError::ChecksumMismatch);
            }
            let mut c_to_e_key_page_data = c_to_e_key_page_data.as_slice();
            while let Ok(c_to_e_key) = CToEKey::read_from(&mut c_to_e_key_page_data, c_key_size, e_key_size) {
                c_to_e_key_page.c_to_e_keys.push(c_to_e_key);
            }
        }

        let mut e_key_spec_page_table = Vec::with_capacity(e_key_spec_page_count as usize);
        for _ in 0..e_key_spec_page_table.capacity() {
            e_key_spec_page_table.push(EKeySpecPage {
                first_e_key: {
                    let mut first_c_key = vec![0; c_key_size as usize];
                    input.read_exact(&mut first_c_key)?;
                    first_c_key
                },
                md5: {
                    let mut checksum = [0; 0x10];
                    input.read_exact(&mut checksum)?;
                    checksum
                },
                e_key_specs: vec![],
            });
        }
        for e_key_spec_page in &mut e_key_spec_page_table {
            let mut e_key_spec_page_data = vec![0; e_key_spec_page_size as usize * 1024];
            input.read_exact(&mut e_key_spec_page_data)?;
            let mut md5 = Md5::new();
            md5.update(&e_key_spec_page_data);
            if e_key_spec_page.md5 != md5.finalize().as_slice() {
                bail!(TactError::ChecksumMismatch);
            }
            let mut e_key_spec_page_data = e_key_spec_page_data.as_slice();
            while let Ok(e_key_spec) = EKeySpec::read_from(&mut e_key_spec_page_data, e_key_size, &e_specs) {
                e_key_spec_page.e_key_specs.push(e_key_spec);
            }
        }

        Ok(Self {
            c_to_e_key_page_table,
            e_key_spec_page_table,
        })
    }
}

#[derive(Debug)]
pub struct CToEKeyPage {
    pub first_c_key: Vec<u8>,
    md5: [u8; 0x10],
    pub c_to_e_keys: Vec<CToEKey>
}

#[derive(Debug)]
pub struct CToEKey {
    pub c_key: Vec<u8>,
    e_keys: Vec<Vec<u8>>,
    c_size: u64,
}

impl CToEKey {
    fn read_from<R: Read>(input: &mut R, c_key_size: u8, e_key_size: u8) -> Result<Self> {
        let e_key_count = input.read_u8()?;
        let c_size = input.read_uint::<BigEndian>(5)?;
        let mut c_key = vec![0; c_key_size as usize];
        input.read_exact(&mut c_key)?;
        let mut e_keys = Vec::with_capacity(e_key_count as usize);
        for _ in 0..e_keys.capacity() {
            let mut e_key = vec![0; e_key_size as usize];
            input.read_exact(&mut e_key)?;
            e_keys.push(e_key);
        }

        Ok(Self {
            c_key,
            e_keys,
            c_size,
        })
    }
}

#[derive(Debug)]
pub struct EKeySpecPage {
    first_e_key: Vec<u8>,
    md5: [u8; 0x10],
    e_key_specs: Vec<EKeySpec>
}

#[derive(Debug)]
pub struct EKeySpec {
    e_key: Vec<u8>,
    e_spec: String,
    e_size: u64,
}

impl EKeySpec {
    fn read_from<R: Read>(input: &mut R, e_key_size: u8, e_specs: &Vec<String>) -> Result<Self> {
        Ok(Self {
            e_key: {
                let mut e_key = vec![0; e_key_size as usize];
                input.read_exact(&mut e_key)?;
                e_key
            },
            e_spec: e_specs.get(input.read_u32::<BigEndian>()? as usize).unwrap_or(&"".to_string()).clone(),
            e_size: input.read_uint::<BigEndian>(5)?,
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
