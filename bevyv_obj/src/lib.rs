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
        Box::pin(async move { Ok(load_obj(bytes, load_context).await?) })
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

    for line in bytes.lines().map(|line| line.unwrap()) {
        if line.starts_with('#') {
            continue;
        }

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
                    let mut element_count = 0;

                    for element in line_iter {
                        let mut indices = element.split('/');

                        // Vertex
                        if let Some(index_str) = indices.next() {
                            let index: i32 = index_str.parse()?;
                            let absolute_index = if index.is_negative() {
                                (vertices.len() as i32) - index
                            } else {
                                index.to_owned()
                            };
                            positions.push(
                                *vertices
                                    .get((absolute_index - 1) as usize)
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
                            let absolute_index = if index.is_negative() {
                                (vertices.len() as i32) - index
                            } else {
                                index.to_owned()
                            };
                            uvs.push(
                                *vertices_texture
                                    .get((absolute_index - 1) as usize)
                                    .ok_or(ObjError::IndexOutOfRange(index))?,
                            );
                        }

                        // Vertex Normal
                        if let Some(index_str) =
                            indices.next().filter(|index_str| !index_str.is_empty())
                        {
                            let index: i32 = index_str.parse()?;
                            let absolute_index = if index.is_negative() {
                                (vertices.len() as i32) - index
                            } else {
                                index.to_owned()
                            };
                            normals.push(
                                *vertices_normal
                                    .get((absolute_index - 1) as usize)
                                    .ok_or(ObjError::IndexOutOfRange(index))?,
                            );
                        }

                        element_count += 1;
                    }

                    if element_count != 3 {
                        bail!(ObjError::UnsupportedStatement(line));
                    }
                }
                _ => { /*return Err(ObjError::UnknownStatement(stmt.to_string()));*/ }
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    load_context.set_default_asset(LoadedAsset::new(mesh));

    Ok(())
}
