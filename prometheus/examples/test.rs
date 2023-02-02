use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::Result;
use csv::ReaderBuilder;
use serde::Deserialize;

use prometheus::casc::Casc;
use prometheus::guid::Guid;
use prometheus::stu::de::Deserializer;
use prometheus::tact::{BuildInfo, Encoding, RootFile};
use prometheus::tact_manifest::{
    decrypt_cmf, ContentManifestAsset, ContentManifestEntry, ContentManifestHeader,
};

fn main() -> Result<()> {
    let path = PathBuf::from("/drive_c/Program Files (x86)/Overwatch");
    let build_info: BuildInfo = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path(path.join(".build.info"))
        .unwrap()
        .deserialize()
        .next()
        .unwrap()
        .unwrap();
    let mut build_config = HashMap::new();
    for line in BufReader::new(
        File::open(path.join(format!(
            "data/casc/config/{:02x}/{:02x}/{}",
            build_info.build_key[0],
            build_info.build_key[1],
            hex::encode(build_info.build_key)
        )))
        .unwrap(),
    )
    .lines()
    {
        let line = line.unwrap();
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let mut key_value = line.split('=');
        build_config.insert(
            key_value.next().unwrap().trim().to_owned(),
            key_value.next().unwrap().trim().to_owned(),
        );
    }

    let storage = Casc::new(path.join("data/casc/data")).unwrap();
    let encoding = Encoding::read_from(
        &mut storage
            .get(&hex::decode(build_config["encoding"].split(' ').collect::<Vec<_>>()[1]).unwrap())
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    let mut root_files = HashMap::new();
    {
        for entry in ReaderBuilder::new()
            .delimiter(b'|')
            .from_reader(
                storage
                    .get(
                        &encoding
                            .get(&hex::decode(&build_config["root"]).unwrap())
                            .unwrap(),
                    )
                    .unwrap()
                    .as_slice(),
            )
            .deserialize()
        {
            let entry: RootFile = entry.unwrap();
            root_files.insert(entry.file_name, entry.md5);
        }
    }

    let mut assets = HashMap::new();
    let content_manifest_data = storage
        .get(
            &encoding
                .get(&root_files["TactManifest/Win_SPWin_RCN_EExt.cmf"])
                .unwrap(),
        )
        .unwrap();
    let (content_manifest_header_data, content_manifest_data) =
        content_manifest_data.split_at(std::mem::size_of::<ContentManifestHeader>());
    let content_manifest_header: &ContentManifestHeader =
        bytemuck::from_bytes(content_manifest_header_data);
    let content_manifest_data = decrypt_cmf(
        "TactManifest/Win_SPWin_RCN_EExt.cmf",
        content_manifest_header,
        content_manifest_data,
    )
    .unwrap();
    let content_manifest_asset_offset =
        content_manifest_header.entry_count as usize * std::mem::size_of::<ContentManifestEntry>();
    let content_manifest_assets: &[ContentManifestAsset] = bytemuck::cast_slice(
        &content_manifest_data[content_manifest_asset_offset
            ..content_manifest_asset_offset
                + content_manifest_header.asset_count as usize
                    * std::mem::size_of::<ContentManifestAsset>()],
    );
    for asset in content_manifest_assets {
        assets.insert(asset.guid, asset.md5);
    }

    for (guid, md5) in assets {
        let guid = Guid::from(guid);
        if guid.type_ == 0x09F {
            if let Some(e_key) = encoding.get(&md5) {
                let mut data = storage.get(&e_key)?;
                let mut deserializer = Deserializer::from_slice(&mut data)?;
                let map_header = MapHeader::deserialize(&mut deserializer)?;
                println!("{:?}", map_header);
            }
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct MapHeader {
    #[serde(rename = "506FA8D8")]
    map_name: String,
}
