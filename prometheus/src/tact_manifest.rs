use std::io::Read;

use aes::cipher::block_padding::NoPadding;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use byteorder::{LittleEndian, ReadBytesExt};
use sha1::{Digest, Sha1};

#[derive(Debug)]
pub struct ContentManifest {
    entries: Vec<ContentManifestEntry>,
    pub assets: Vec<ContentManifestAsset>,
}

impl ContentManifest {
    pub fn read_from(mut input: &[u8], file_name: String) -> Result<Self> {
        let header = bytemuck::from_bytes::<ContentManifestHeader>(
            &input[..std::mem::size_of::<ContentManifestHeader>()],
        );
        input = &input[std::mem::size_of::<ContentManifestHeader>()..];

        if input.read_u32::<LittleEndian>()? >> 8 /* cmf */ != 0x636D66 {}
        let input = {
            if header.build_version != 109168 {}
            let mut file_name_sha1 = Sha1::new();
            file_name_sha1.update(file_name);
            let file_name_sha1 = file_name_sha1.finalize();
            let key = {
                let mut buffer = [0u8; 0x20];
                let mut kidx = buffer.len() as u32 * header.build_version;
                let mut okidx = kidx;
                for i in 0..buffer.len() {
                    buffer[i] = KEYTABLE[(kidx % 0x200) as usize];
                    kidx = header.build_version.wrapping_sub(kidx);
                }
                buffer
            };
            let iv = {
                let mut buffer = [0u8; 16];
                let mut kidx = KEYTABLE[(header.build_version & 0x1FF) as usize] as u32;
                let mut okidx = kidx;
                for i in 0..buffer.len() {
                    buffer[i] = KEYTABLE[(kidx % 0x200) as usize];
                    kidx = kidx
                        .wrapping_add(header.build_version.wrapping_mul(header.asset_count) % 7);
                    buffer[i] ^= file_name_sha1[kidx
                        .wrapping_sub(73)
                        .rem_euclid(file_name_sha1.len() as u32)
                        as usize]
                }
                buffer
            };
            Aes256CbcDec::new_from_slices(&key, &iv)
                .unwrap()
                .decrypt_padded_vec_mut::<NoPadding>(input)
                .unwrap()
        };
        let mut input = input.as_slice();

        let mut entries = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.entry_count {
            entries.push(ContentManifestEntry::read_from(&mut input)?);
        }
        let mut assets = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.asset_count {
            assets.push(ContentManifestAsset::read_from(&mut input)?);
        }

        Ok(ContentManifest { assets, entries })
    }
}

#[derive(Debug)]
struct ContentManifestEntry {
    index: u32,
    hash_a: u64,
    hash_b: u64,
}

impl ContentManifestEntry {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(ContentManifestEntry {
            index: input.read_u32::<LittleEndian>()?,
            hash_a: input.read_u64::<LittleEndian>()?,
            hash_b: input.read_u64::<LittleEndian>()?,
        })
    }
}

#[derive(Debug)]
pub struct ContentManifestAsset {
    pub guid: u64,
    size: u32,
    unknown_0c: u8,
    pub md5: [u8; 0x10],
}

impl ContentManifestAsset {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(ContentManifestAsset {
            guid: input.read_u64::<LittleEndian>()?,
            size: input.read_u32::<LittleEndian>()?,
            unknown_0c: input.read_u8()?,
            md5: {
                let mut md5 = [0; 0x10];
                input.read_exact(&mut md5)?;
                md5
            },
        })
    }
}

#[derive(Debug)]
struct ResourceGraph {
    unknown_00: u32,
    build_version: u32,
    unknown_08: u32,
    unknown_0c: u32,
    unknown_10: u32,
    unknown_14: u32,
    type_bundle_index_count: u32,
    type_bundle_index_block_size: u32,
    unknown_30: u32,
    unknown_34: u32,
    graph_block_size: u32,
    packages: Vec<ResourceGraphPackage>,
    skins: Vec<ResourceGraphSkin>,
}

impl ResourceGraph {
    pub fn read_from(mut input: &[u8], file_name: String) -> Result<Self> {
        let unknown_00 = input.read_u32::<LittleEndian>()?;
        let build_version = input.read_u32::<LittleEndian>()?;
        let unknown_08 = input.read_u32::<LittleEndian>()?;
        let unknown_0c = input.read_u32::<LittleEndian>()?;
        let unknown_10 = input.read_u32::<LittleEndian>()?;
        let unknown_14 = input.read_u32::<LittleEndian>()?;
        let package_count = input.read_u32::<LittleEndian>()?;
        let package_block_size = input.read_u32::<LittleEndian>()?;
        let skin_count = input.read_u32::<LittleEndian>()?;
        let skin_block_size = input.read_u32::<LittleEndian>()?;
        let type_bundle_index_count = input.read_u32::<LittleEndian>()?;
        let type_bundle_index_block_size = input.read_u32::<LittleEndian>()?;
        let unknown_30 = input.read_u32::<LittleEndian>()?;
        let unknown_34 = input.read_u32::<LittleEndian>()?;
        let graph_block_size = input.read_u32::<LittleEndian>()?;

        if input.read_u32::<LittleEndian>()? >> 8 /* trg */ != 0x677274 {}
        let input = {
            if build_version != 109168 {}
            let mut file_name_sha1 = Sha1::new();
            file_name_sha1.update(file_name);
            let file_name_sha1 = file_name_sha1.finalize();
            let key = {
                let mut buffer = [0u8; 0x20];
                let mut kidx = buffer.len() as u32 * build_version;
                let mut okidx = kidx;
                for i in 0..buffer.len() {
                    buffer[i] = KEYTABLE[(kidx % 0x200) as usize];
                    kidx = build_version.wrapping_sub(kidx);
                }
                buffer
            };
            let iv = {
                let mut buffer = [0u8; 16];
                let mut kidx = KEYTABLE[(build_version & 0x1FF) as usize] as u32;
                let mut okidx = kidx;
                for i in 0..buffer.len() {
                    buffer[i] = KEYTABLE[(kidx % 0x200) as usize];
                    kidx = kidx.wrapping_add(build_version.wrapping_mul(skin_count) % 7);
                    buffer[i] ^= file_name_sha1
                        [(kidx.wrapping_sub(73) % file_name_sha1.len() as u32) as usize]
                }
                buffer
            };
            Aes256CbcDec::new_from_slices(&key, &iv)
                .unwrap()
                .decrypt_padded_vec_mut::<NoPadding>(input)
                .unwrap()
        };
        let mut input = input.as_slice();

        let mut packages = Vec::with_capacity(package_count as usize);
        for _ in 0..package_count {
            packages.push(ResourceGraphPackage::read_from(&mut input)?);
        }
        let mut skins = Vec::with_capacity(skin_count as usize);
        for _ in 0..skin_count {
            skins.push(ResourceGraphSkin::read_from(&mut input)?);
        }

        Ok(Self {
            unknown_00,
            build_version,
            unknown_08,
            unknown_0c,
            unknown_10,
            unknown_14,
            type_bundle_index_count,
            type_bundle_index_block_size,
            unknown_30,
            unknown_34,
            graph_block_size,
            packages,
            skins,
        })
    }
}

#[derive(Debug)]
struct ResourceGraphPackage {
    asset_guid: u64,
    resource_key_id: u64,
    unknown_10: u32,
    unknown_14: u32,
    unknown_18: u32,
    unknown_1c: u8,
}

impl ResourceGraphPackage {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            asset_guid: input.read_u64::<LittleEndian>()?,
            resource_key_id: input.read_u64::<LittleEndian>()?,
            unknown_10: input.read_u32::<LittleEndian>()?,
            unknown_14: input.read_u32::<LittleEndian>()?,
            unknown_18: input.read_u32::<LittleEndian>()?,
            unknown_1c: input.read_u8()?,
        })
    }
}

#[derive(Debug)]
struct ResourceGraphSkin {
    asset_ptr: u64,
    skin_guid: u64,
    unknown_10: u32,
    unknown_14: u32,
    unknown_18: u32,
    unknown_1c: u32,
    unknown_20: u32,
    unknown_26: u16,
    assets: Vec<ResourceGraphSkinAsset>,
}

impl ResourceGraphSkin {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let asset_ptr = input.read_u64::<LittleEndian>()?;
        let skin_guid = input.read_u64::<LittleEndian>()?;
        let unknown_10 = input.read_u32::<LittleEndian>()?;
        let unknown_14 = input.read_u32::<LittleEndian>()?;
        let unknown_18 = input.read_u32::<LittleEndian>()?;
        let unknown_1c = input.read_u32::<LittleEndian>()?;
        let unknown_20 = input.read_u32::<LittleEndian>()?;
        let asset_count = input.read_u16::<LittleEndian>()?;
        let unknown_26 = input.read_u16::<LittleEndian>()?;
        let mut assets = Vec::with_capacity(asset_count as usize);
        for _ in 0..asset_count {
            assets.push(ResourceGraphSkinAsset::read_from(input)?);
        }

        Ok(Self {
            asset_ptr,
            skin_guid,
            unknown_10,
            unknown_14,
            unknown_18,
            unknown_1c,
            unknown_20,
            unknown_26,
            assets,
        })
    }
}

#[derive(Debug)]
struct ResourceGraphSkinAsset {
    source_asset: u64,
    dest_asset: u64,
    unknown_10: u64,
    unknown_18: u32,
    unknown_1c: u32,
}

impl ResourceGraphSkinAsset {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            source_asset: input.read_u64::<LittleEndian>()?,
            dest_asset: input.read_u64::<LittleEndian>()?,
            unknown_10: input.read_u64::<LittleEndian>()?,
            unknown_18: input.read_u32::<LittleEndian>()?,
            unknown_1c: input.read_u32::<LittleEndian>()?,
        })
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ContentManifestHeader {
    build_version: u32,
    u32_0: u32,
    u32_1: u32,
    u32_2: u32,
    u32_3: u32,
    u32_4: u32,
    u32_5: u32,
    asset_patch_record_count: u32,
    asset_count: u32,
    entry_patch_record_count: u32,
    entry_count: u32,
}

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

static KEYTABLE: &[u8] = &[
    0xCB, 0x08, 0x59, 0x42, 0x68, 0xE7, 0xC9, 0xE7, 0x34, 0x4A, 0x62, 0xB2, 0xD2, 0x66, 0x22, 0x49,
    0x05, 0x93, 0x09, 0x3E, 0x32, 0x32, 0xE8, 0x82, 0x62, 0x34, 0xDD, 0xA9, 0x8B, 0x4C, 0x3D, 0xF9,
    0xFF, 0xF2, 0x42, 0x3A, 0x3B, 0xC7, 0x8B, 0x0B, 0xDC, 0x73, 0x9F, 0x1A, 0x28, 0x1C, 0x01, 0xF8,
    0xAA, 0x7A, 0xFD, 0x0C, 0xC5, 0x38, 0x98, 0xE6, 0x37, 0x39, 0xE0, 0xC6, 0xCA, 0xB6, 0x58, 0x2D,
    0xB6, 0x56, 0xFE, 0x7A, 0x11, 0x22, 0xB5, 0xDA, 0xDE, 0x35, 0xF2, 0xE9, 0xE9, 0x0A, 0x45, 0x70,
    0xE2, 0x8E, 0x1E, 0xED, 0xAE, 0x49, 0x0D, 0x91, 0x9F, 0x87, 0xC6, 0x3F, 0x17, 0xB9, 0x15, 0x47,
    0xC8, 0x96, 0x7A, 0xA9, 0x48, 0x9F, 0xD5, 0x31, 0x0F, 0xEE, 0xB8, 0xDE, 0xFA, 0x47, 0x06, 0xFA,
    0xA4, 0x34, 0x26, 0xB4, 0x72, 0x84, 0xBD, 0x74, 0x61, 0x2B, 0x81, 0xB5, 0x81, 0x0E, 0x1D, 0x2F,
    0x70, 0xBF, 0xBF, 0x28, 0x10, 0x9C, 0xBE, 0x08, 0x09, 0xC4, 0x74, 0x55, 0x38, 0x68, 0xB8, 0x2B,
    0xAC, 0x43, 0x1D, 0xEE, 0xD9, 0x1B, 0xB9, 0xDE, 0xE7, 0xFF, 0xDB, 0xF3, 0x4E, 0x04, 0x8B, 0x70,
    0x5F, 0xFE, 0x2D, 0xC8, 0x4A, 0x82, 0xAD, 0x1D, 0xF0, 0x68, 0xE3, 0x13, 0x05, 0x32, 0x8E, 0x8C,
    0x4B, 0x77, 0x89, 0x69, 0xAF, 0xFC, 0x85, 0xA6, 0xE3, 0x56, 0x70, 0xC7, 0x66, 0xEC, 0xCB, 0xE0,
    0x39, 0x59, 0x06, 0xBF, 0xBE, 0x6F, 0x8B, 0x83, 0xA2, 0xF8, 0xE3, 0xE7, 0x74, 0x78, 0x30, 0xD9,
    0xA5, 0x96, 0xAD, 0x73, 0x74, 0x45, 0xB4, 0x07, 0xC3, 0x68, 0xE8, 0x81, 0xB9, 0xEC, 0xF6, 0xED,
    0x12, 0x0F, 0xC9, 0x88, 0x2A, 0xCE, 0xF1, 0x26, 0xAB, 0xFA, 0x04, 0xBE, 0xE7, 0x7C, 0x4A, 0x64,
    0xF4, 0x90, 0xB8, 0xF2, 0x9C, 0xF4, 0x8C, 0x61, 0xDA, 0x15, 0x66, 0xDF, 0x7C, 0xDE, 0x28, 0x2E,
    0xE6, 0x02, 0x7C, 0x36, 0xFA, 0xD4, 0xD5, 0xE8, 0x01, 0x7D, 0x4B, 0xDF, 0x5F, 0x37, 0x9F, 0x51,
    0xC5, 0x16, 0xEE, 0x30, 0xA4, 0x3A, 0xB9, 0xFC, 0x27, 0x96, 0x4C, 0xB8, 0xC0, 0x2D, 0xF8, 0x42,
    0x36, 0x3A, 0x99, 0x52, 0xFE, 0x1F, 0xCB, 0x1A, 0xB8, 0x3C, 0xD8, 0x3A, 0x49, 0x05, 0xC8, 0x0E,
    0x49, 0x38, 0x34, 0x2E, 0x13, 0x2D, 0x51, 0x1B, 0x7C, 0xAC, 0xC9, 0x38, 0x10, 0x84, 0xF6, 0x4E,
    0xAD, 0x45, 0xB6, 0x87, 0x1D, 0x6A, 0xC2, 0xF1, 0x14, 0xDE, 0xA3, 0x94, 0xD0, 0xF2, 0x1D, 0x8C,
    0x45, 0x2A, 0xFD, 0x5E, 0x8F, 0x3A, 0x1E, 0x68, 0x04, 0x26, 0xBE, 0x8C, 0x79, 0x7F, 0x46, 0x4A,
    0xB6, 0x6B, 0xE2, 0x99, 0xB4, 0x11, 0x6F, 0x1D, 0xCE, 0x3D, 0xF9, 0xDD, 0x00, 0x69, 0x63, 0xD4,
    0x0E, 0x74, 0x74, 0x99, 0x66, 0x5F, 0x28, 0xF1, 0xF9, 0x11, 0x83, 0x00, 0x75, 0x8C, 0x2B, 0x67,
    0x78, 0xB0, 0x63, 0xA5, 0x23, 0x9B, 0xDF, 0x07, 0xD5, 0xEA, 0xAB, 0xFB, 0xA0, 0xBD, 0xC4, 0x41,
    0xBB, 0xE5, 0xC8, 0xC2, 0x12, 0x8D, 0x3F, 0x43, 0x31, 0x59, 0x01, 0x78, 0x93, 0xEC, 0xB9, 0x46,
    0x32, 0xFF, 0x90, 0x1D, 0xEE, 0xA9, 0x1B, 0x53, 0x61, 0x75, 0x24, 0x67, 0x1B, 0x69, 0x05, 0x77,
    0x24, 0x85, 0x22, 0xFD, 0x8E, 0x5E, 0xA6, 0x69, 0xFA, 0x94, 0x10, 0xFC, 0xC1, 0x65, 0xA9, 0x95,
    0x6B, 0xC0, 0x0B, 0xDE, 0x0B, 0x03, 0x76, 0x5D, 0x01, 0xF9, 0x9D, 0x77, 0x20, 0xDA, 0x36, 0x87,
    0x6E, 0xDE, 0x9E, 0x35, 0xE3, 0xAD, 0xEE, 0x8E, 0x22, 0xD2, 0xB4, 0x44, 0xDB, 0x46, 0x73, 0xFB,
    0x17, 0x51, 0x69, 0x9D, 0xDF, 0x20, 0x64, 0xBC, 0xD1, 0x54, 0x69, 0xA4, 0x6B, 0x8F, 0xA4, 0xDB,
    0x0D, 0x35, 0xAE, 0x5C, 0xF2, 0x35, 0x19, 0x9A, 0xE4, 0xA9, 0x61, 0xDB, 0x04, 0xEB, 0x06, 0xD2,
];
