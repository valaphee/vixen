use anyhow::{bail, Result};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use md5::{Digest, Md5};
use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlteError {
    #[error("unsupported")]
    Unsupported,
    #[error("unknown encoding mode: {0}")]
    UnknownEncodingMode(char),
    #[error("integrity error")]
    IntegrityError,
}

pub struct Blte;

impl Blte {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Vec<u8>> {
        if input.read_u32::<BigEndian>().unwrap() != u32::from_be_bytes(*b"BLTE") {
            bail!(BlteError::Unsupported);
        }
        input.read_u32::<BigEndian>().unwrap();
        input.read_u8().unwrap();
        let chunk_count = input.read_u24::<BigEndian>().unwrap();
        let mut chunks = Vec::with_capacity(chunk_count as usize);
        for _ in 0..chunk_count {
            chunks.push(BlteChunk::read_from(input)?);
        }

        let mut content =
            Vec::with_capacity(chunks.iter().map(|chunk| chunk.content_size).sum::<u32>() as usize);
        for chunk in chunks.iter() {
            let mut encoded = vec![0; chunk.encoded_size as usize];
            input.read_exact(&mut encoded)?;
            let mut md5 = Md5::new();
            md5.update(&encoded);
            if chunk.md5 != md5.finalize().as_slice() {
                bail!(BlteError::IntegrityError);
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
    md5: [u8; 16],
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
