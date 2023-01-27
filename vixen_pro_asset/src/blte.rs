use std::io::Read;
use anyhow::{bail, Result};
use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use md5::{Digest, Md5};
use thiserror::Error;

#[derive(Error, Debug)]
enum BlteError {
    #[error("invalid magic")]
    InvalidMagic,
    #[error("unknown encoding mode: {0}")]
    UnknownEncodingMode(char),
    #[error("checksum mismatch")]
    ChecksumMismatch,
}

#[derive(Debug)]
pub struct Blte {
    flags: u8,
    pub content: Vec<u8>
}

impl Blte {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u32::<BigEndian>().unwrap() /* BLTE */ != 0x424C5445 {
            bail!(BlteError::InvalidMagic);
        }
        let size = input.read_u32::<BigEndian>().unwrap();
        let flags = input.read_u8().unwrap();
        let mut chunks = Vec::with_capacity(input.read_u24::<BigEndian>().unwrap() as usize);
        for _ in 0..chunks.capacity() {
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

        Ok(Self {
            flags,
            content
        })
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
