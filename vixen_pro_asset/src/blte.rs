use std::io::Read;
use anyhow::{bail, Result};
use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use md5::{Digest, Md5};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlteError {
    #[error("invalid magic")]
    InvalidMagic,
    #[error("invalid size")]
    InvalidSize,
    #[error("unknown encoding mode: {0}")]
    UnknownEncodingMode(char),
    #[error("checksum mismatch")]
    ChecksumMismatch,
}

pub struct Blte;

impl Blte {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Vec<u8>> {
        if input.read_u32::<BigEndian>().unwrap() /* BLTE */ != 0x424C5445 {
            bail!(BlteError::InvalidMagic);
        }
        let header_size = input.read_u32::<BigEndian>().unwrap();
        if input.read_u8().unwrap() != 0xF {
            bail!(BlteError::InvalidMagic);
        }
        let chunk_count = input.read_u24::<BigEndian>().unwrap();
        if header_size != 4 + 4 + 1 + 3 + chunk_count * (16 + 4 + 4) {
            bail!(BlteError::InvalidSize);
        }
        let mut chunks = Vec::with_capacity(chunk_count as usize);
        for _ in 0..chunk_count {
            chunks.push(BlteChunk::read_from(input)?);
        }

        let mut content: Vec<u8> = Vec::new();
        for chunk in chunks.iter() {
            let mut encoded = vec![0; chunk.encoded_size as usize];
            input.read_exact(&mut encoded)?;
            let mut md5 = Md5::new();
            md5.update(&encoded);
            if chunk.md5 != md5.finalize().as_slice() {
                bail!(BlteError::ChecksumMismatch);
            }
            let mut encoded = encoded.as_slice();
            let encoding_mode = encoded.read_u8()?;
            match encoding_mode {
                b'N' => {
                    content.extend(encoded);
                }
                b'Z' => {
                    ZlibDecoder::new(encoded).read_to_end(&mut content)?;
                }
                _ => {
                    bail!(BlteError::UnknownEncodingMode(encoding_mode as char));
                }
            }
        }

        Ok(content)
    }
}

#[derive(Debug)]
struct BlteChunk {
    encoded_size: u32,
    content_size: u32,
    md5: [u8; 16]
}

impl BlteChunk {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            encoded_size: input.read_u32::<BigEndian>()?,
            content_size: input.read_u32::<BigEndian>()?,
            md5: {
                let mut md5 = [0; 16];
                input.read_exact(&mut md5)?;
                md5
            },
        })
    }
}
