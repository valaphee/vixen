use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::Result;
use csv::ReaderBuilder;
use serde::Deserialize;

use prometheus::casc::Casc;
use prometheus::tact::{BuildInfo, Encoding, RootFile};
use prometheus::tact_manifest::ContentManifest;

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
    {
        let content_manifest = ContentManifest::read_from(
            storage
                .get(
                    &encoding
                        .get(&root_files["TactManifest/Win_SPWin_RCN_EExt.cmf"])
                        .unwrap(),
                )
                .unwrap()
                .as_slice(),
            "TactManifest/Win_SPWin_RCN_EExt.cmf".to_string(),
        )
        .unwrap();
        for asset in content_manifest.assets {
            assets.insert(asset.guid, asset.md5);
        }
    }

    Ok(())
}
