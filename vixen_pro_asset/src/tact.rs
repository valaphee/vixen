use std::collections::HashMap;
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
    pub c_to_e_keys: HashMap<Vec<u8>, EncodingCToEKey>,
    pub e_key_specs: HashMap<Vec<u8>, EncodingEKeySpec>,
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
        for _ in 0..c_to_e_key_page_count {
            c_to_e_key_page_table.push(EncodingPage::read_from(input, c_key_size)?);
        }

        let mut c_to_e_keys = HashMap::new();
        for c_to_e_key_page in &mut c_to_e_key_page_table {
            let mut c_to_e_key_page_data = vec![0; c_to_e_key_page_size as usize * 1024];
            input.read_exact(&mut c_to_e_key_page_data)?;
            let mut md5 = Md5::new();
            md5.update(&c_to_e_key_page_data);
            if c_to_e_key_page.md5 != md5.finalize().as_slice() {
                bail!(TactError::ChecksumMismatch);
            }
            let mut c_to_e_key_page_data = c_to_e_key_page_data.as_slice();
            while let Ok(c_to_e_key) = EncodingCToEKey::read_from(&mut c_to_e_key_page_data, c_key_size, e_key_size) {
                c_to_e_keys.insert(c_to_e_key.c_key.clone(), c_to_e_key);
            }
        }

        let mut e_key_spec_page_table = Vec::with_capacity(e_key_spec_page_count as usize);
        for _ in 0..e_key_spec_page_count {
            e_key_spec_page_table.push(EncodingPage::read_from(input, e_key_size)?);
        }

        let mut e_key_specs = HashMap::new();
        for e_key_spec_page in &mut e_key_spec_page_table {
            let mut e_key_spec_page_data = vec![0; e_key_spec_page_size as usize * 1024];
            input.read_exact(&mut e_key_spec_page_data)?;
            let mut md5 = Md5::new();
            md5.update(&e_key_spec_page_data);
            if e_key_spec_page.md5 != md5.finalize().as_slice() {
                bail!(TactError::ChecksumMismatch);
            }
            let mut e_key_spec_page_data = e_key_spec_page_data.as_slice();
            while let Ok(e_key_spec) = EncodingEKeySpec::read_from(&mut e_key_spec_page_data, e_key_size, &e_specs) {
                e_key_specs.insert(e_key_spec.e_key.clone(), e_key_spec);
            }
        }

        Ok(Self {
            c_to_e_keys,
            e_key_specs,
        })
    }
}

struct EncodingPage {
    first_key: Vec<u8>,
    md5: [u8; 0x10],
}

impl EncodingPage {
    fn read_from<R: Read>(input: &mut R, key_size: u8) -> Result<Self> {
        Ok(Self {
            first_key: {
                let mut first_key = vec![0; key_size as usize];
                input.read_exact(&mut first_key)?;
                first_key
            },
            md5: {
                let mut md5 = [0; 0x10];
                input.read_exact(&mut md5)?;
                md5
            },
        })
    }
}

#[derive(Debug)]
pub struct EncodingCToEKey {
    c_key: Vec<u8>,
    pub c_size: u64,
    pub e_keys: Vec<Vec<u8>>,
}

impl EncodingCToEKey {
    fn read_from<R: Read>(input: &mut R, c_key_size: u8, e_key_size: u8) -> Result<Self> {
        let e_key_count = input.read_u8()?;
        let c_size = input.read_uint::<BigEndian>(5)?;
        let mut c_key = vec![0; c_key_size as usize];
        input.read_exact(&mut c_key)?;
        let mut e_keys = Vec::with_capacity(e_key_count as usize);
        for _ in 0..e_key_count {
            let mut e_key = vec![0; e_key_size as usize];
            input.read_exact(&mut e_key)?;
            e_keys.push(e_key);
        }

        Ok(Self {
            c_key,
            c_size,
            e_keys,
        })
    }
}

#[derive(Debug)]
pub struct EncodingEKeySpec {
    e_key: Vec<u8>,
    pub e_size: u64,
    pub e_spec: String,
}

impl EncodingEKeySpec {
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
