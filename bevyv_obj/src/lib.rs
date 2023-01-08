use std::io::BufRead;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    render::mesh::PrimitiveTopology
};
use thiserror::Error;

#[derive(Default)]
pub struct ObjPlugin;

impl Plugin for ObjPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<ObjLoader>();
    }
}

#[derive(Default)]
struct ObjLoader;

impl AssetLoader for ObjLoader {
    fn load<'a>(&'a self, bytes: &'a [u8], load_context: &'a mut LoadContext) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            Ok(load_obj(bytes, load_context).await?)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["obj"]
    }
}

#[derive(Error, Debug)]
enum ObjError {
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
    #[error("index out of range")]
    IndexOutOfRange(u32),
    #[error("unknown statement: {0}")]
    UnknownStatement(String),
}

async fn load_obj<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), ObjError> {
    let mut vs: Vec<Vec<f32>> = Vec::new();
    let mut vts: Vec<Vec<f32>> = Vec::new();
    let mut vns: Vec<Vec<f32>> = Vec::new();
    let mut fs: Vec<Vec<Vec<u32>>> = Vec::new();

    // Parse file
    for line in bytes.lines().map(|line| line.unwrap()) {
        // Skip comments
        if line.starts_with('#') {
            continue;
        }

        let mut line_iter = line.split_whitespace();
        if let Some(stmt) = line_iter.next() {
            match stmt {
                "v" => {
                    vs.push(line_iter.map(|elem| elem.parse().unwrap()).collect());
                }
                "vt" => {
                    vts.push(line_iter.map(|elem| elem.parse().unwrap()).collect());
                }
                "vn" => {
                    vns.push(line_iter.map(|elem| elem.parse().unwrap()).collect());
                }
                "f" => {
                    fs.push(line_iter.map(|face| face.split('/').map(|elem| if elem.is_empty() { 0 } else { elem.parse().unwrap() }).collect()).collect());
                }
                _ => {
                    /*return Err(ObjError::UnknownStatement(stmt.to_string()));*/
                }
            }
        }
    }

    // Generate non-indexed mesh
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    for f in fs {
        for e in f {
            let v_index = e.get(0).unwrap_or(&0).to_owned();
            if v_index == 0 {
                return Err(ObjError::WrongNumberOfArguments);
            }

            {
                let v = vs.get((v_index - 1) as usize).ok_or(ObjError::IndexOutOfRange(v_index))?;
                if v.len() < 3 {
                    return Err(ObjError::WrongNumberOfArguments);
                }
                positions.push([v[0], v[1], v[2]]);
                if v.len() >= 6 {
                    colors.push([v[4], v[5], v[6]]);
                }
            }

            let vt_index = e.get(1).unwrap_or(&0).to_owned();
            if vt_index != 0 {
                let vt = vts.get((vt_index - 1) as usize).ok_or(ObjError::IndexOutOfRange(vt_index))?;
                if vt.len() < 1 {
                    return Err(ObjError::WrongNumberOfArguments);
                }
                uvs.push([vt[0], *vt.get(1).unwrap_or(&0.0)]);
            }

            let vn_index = e.get(2).unwrap_or(&0).to_owned();
            if vn_index != 0 {
                let vn = vns.get((vn_index - 1) as usize).ok_or(ObjError::IndexOutOfRange(vn_index))?;
                if vn.len() < 3 {
                    return Err(ObjError::WrongNumberOfArguments);
                }
                normals.push([vn[0], vn[1], vn[2]]);
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    if !positions.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    }
    if !colors.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    if !uvs.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
    if normals.is_empty() {
        mesh.compute_flat_normals();
    } else {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }
    load_context.set_default_asset(LoadedAsset::new(mesh));

    Ok(())
}
