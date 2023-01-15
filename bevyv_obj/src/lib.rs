use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    render::mesh::PrimitiveTopology,
};
use std::io::BufRead;
use thiserror::Error;

/// Wavefront OBJ asset loader.
#[derive(Default)]
pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { load_obj(bytes, load_context).await })
    }

    fn extensions(&self) -> &[&str] {
        &["obj"]
    }
}

#[derive(Error, Debug)]
enum ObjError {
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
    #[error("index out of range: {0}")]
    IndexOutOfRange(i32),
    #[error("unsupported statement: {0}")]
    UnsupportedStatement(String),
}

// See http://paulbourke.net/dataformats/obj/
async fn load_obj<'a, 'b>(bytes: &'a [u8], load_context: &'a mut LoadContext<'b>) -> Result<()> {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut vertices_texture: Vec<[f32; 2]> = Vec::new();
    let mut vertices_normal: Vec<[f32; 3]> = Vec::new();

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    // Go through all lines, one by one
    for line in bytes.lines().map(|line| line.unwrap()) {
        // Ignore comments
        if line.starts_with('#') {
            continue;
        }

        // And split line into statement and arguments
        let mut line_iter = line.split_whitespace();
        if let Some(statement) = line_iter.next() {
            match statement {
                "v" => {
                    vertices.push([
                        line_iter
                            .next()
                            .ok_or(ObjError::WrongNumberOfArguments)?
                            .parse()?,
                        line_iter
                            .next()
                            .ok_or(ObjError::WrongNumberOfArguments)?
                            .parse()?,
                        line_iter
                            .next()
                            .ok_or(ObjError::WrongNumberOfArguments)?
                            .parse()?,
                    ]);
                }
                "vt" => {
                    vertices_texture.push([
                        line_iter
                            .next()
                            .ok_or(ObjError::WrongNumberOfArguments)?
                            .parse()?,
                        line_iter
                            .next()
                            .map(|elem| elem.parse().unwrap())
                            .unwrap_or(0.0),
                    ]);
                }
                "vn" => {
                    vertices_normal.push([
                        line_iter
                            .next()
                            .ok_or(ObjError::WrongNumberOfArguments)?
                            .parse()?,
                        line_iter
                            .next()
                            .ok_or(ObjError::WrongNumberOfArguments)?
                            .parse()?,
                        line_iter
                            .next()
                            .ok_or(ObjError::WrongNumberOfArguments)?
                            .parse()?,
                    ]);
                }
                "f" => {
                    let element_a = line_iter.next().ok_or(ObjError::WrongNumberOfArguments)?;
                    let mut element_b = line_iter.next().ok_or(ObjError::WrongNumberOfArguments)?;
                    for element_c in line_iter {
                        for element in [element_a, element_b, element_c] {
                            let mut indices = element.split('/');

                            // Vertex, has to be always present
                            if let Some(index_str) = indices.next() {
                                let index: i32 = index_str.parse()?;
                                positions.push(
                                    *vertices
                                        .get(absolute_index(index, vertices.len()))
                                        .ok_or(ObjError::IndexOutOfRange(index))?,
                                );
                            } else {
                                bail!(ObjError::WrongNumberOfArguments);
                            }

                            // Vertex Texture
                            if let Some(index_str) =
                                indices.next().filter(|index_str| !index_str.is_empty())
                            {
                                let index: i32 = index_str.parse()?;
                                uvs.push(
                                    *vertices_texture
                                        .get(absolute_index(index, vertices_texture.len()))
                                        .ok_or(ObjError::IndexOutOfRange(index))?,
                                );
                            }

                            // Vertex Normal
                            if let Some(index_str) =
                                indices.next().filter(|index_str| !index_str.is_empty())
                            {
                                let index: i32 = index_str.parse()?;
                                normals.push(
                                    *vertices_normal
                                        .get(absolute_index(index, vertices_texture.len()))
                                        .ok_or(ObjError::IndexOutOfRange(index))?,
                                );
                            }

                            element_b = element_c;
                        }
                    }
                }
                _ => { /*return Err(ObjError::UnknownStatement(stmt.to_string()));*/ }
            }
        }
    }

    // Create mesh, and set it as default asset
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    if !normals.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    } else {
        mesh.compute_flat_normals()
    }
    if !uvs.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
    load_context.set_default_asset(LoadedAsset::new(mesh));

    Ok(())
}

fn absolute_index(index: i32, relative_to: usize) -> usize {
    (if index.is_negative() {
        relative_to as i32 - index
    } else {
        index
    } - 1) as usize
}
