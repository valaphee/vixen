use std::collections::HashMap;

use anyhow::{Error, Result};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    math::Vec3Swizzles,
    prelude::*,
    reflect::TypeUuid,
    render::mesh::{Indices, PrimitiveTopology},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, TypeUuid)]
#[uuid = "108c281c-0d0d-4cc4-8e2e-e09918562681"]
pub struct Skeleton(pub HashMap<String, Handle<Mesh>>);

#[derive(Default)]
pub struct GeometryLoader;

impl AssetLoader for GeometryLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { load_geo_json(bytes, load_context).await })
    }

    fn extensions(&self) -> &[&str] {
        &["geo.json"]
    }
}

async fn load_geo_json<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), Error> {
    let geometry_file = serde_json::from_slice::<GeometryFile>(bytes)?;
    let geometry = geometry_file.geometry.first().unwrap();
    let texture_width = geometry.description.texture_width;
    let texture_height = geometry.description.texture_height;

    let mut bones = HashMap::new();
    for bone in &geometry.bones {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices = Vec::new();
        let mut index = 0;

        if let Some(cubes) = &bone.cubes {
            for cube in cubes {
                let mut transform = Transform::from_translation(cube.origin);
                let euler_rot = cube.rotation.unwrap_or(Vec3::ZERO);
                let quat_rot = Quat::from_euler(
                    EulerRot::XYZ,
                    -euler_rot.x.to_radians(),
                    -euler_rot.y.to_radians(),
                    -euler_rot.z.to_radians(),
                );
                transform.rotate_around(cube.pivot.unwrap_or(Vec3::ZERO), quat_rot);

                let inflate = cube.inflate.or(bone.inflate).unwrap_or(0.);
                let Vec3 {
                    x: size_x,
                    y: size_y,
                    z: size_z,
                } = cube.size + Vec3::splat(inflate);

                fn get_face_uv(
                    cube: &GeometryBoneCube,
                    face: GeometryBoneCubeFace,
                ) -> Option<(Vec2, Vec2)> {
                    let uv_size = match face {
                        GeometryBoneCubeFace::North | GeometryBoneCubeFace::South => cube.size.xy(),
                        GeometryBoneCubeFace::East | GeometryBoneCubeFace::West => cube.size.zy(),
                        GeometryBoneCubeFace::Up | GeometryBoneCubeFace::Down => cube.size.xz(),
                    };
                    match &cube.uv {
                        GeometryBoneCubeUv::Box(uv) => Some(match face {
                            GeometryBoneCubeFace::North => {
                                (Vec2::new(cube.size.z, cube.size.z) + uv.clone(), uv_size)
                            }
                            GeometryBoneCubeFace::West => {
                                (Vec2::new(0., cube.size.z) + uv.clone(), uv_size)
                            }
                            GeometryBoneCubeFace::South => (
                                Vec2::new(cube.size.z + cube.size.x + cube.size.z, cube.size.z)
                                    + uv.clone(),
                                uv_size,
                            ),
                            GeometryBoneCubeFace::East => (
                                Vec2::new(cube.size.z + cube.size.x, cube.size.z) + uv.clone(),
                                uv_size,
                            ),
                            GeometryBoneCubeFace::Up => {
                                (Vec2::new(cube.size.z, 0.) + uv.clone(), uv_size)
                            }
                            GeometryBoneCubeFace::Down => (
                                Vec2::new(cube.size.z + cube.size.x, 0.) + uv.clone(),
                                uv_size,
                            ),
                        }),
                        GeometryBoneCubeUv::PerFace(uv_by_face) => uv_by_face
                            .get(&face)
                            .map(|uv| (uv.uv, uv.uv_size.unwrap_or(uv_size))),
                    }
                }

                if let Some(face_uv) = get_face_uv(cube, GeometryBoneCubeFace::North) {
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, -inflate, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, -inflate, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, size_y, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, size_y, size_z]))
                            .to_array(),
                    );

                    let normal = (quat_rot * Vec3::Z).to_array();
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);

                    let max_uv = face_uv.0 + face_uv.1;
                    let min_u = face_uv.0.x / texture_width as f32;
                    let min_v = face_uv.0.y / texture_height as f32;
                    let max_u = max_uv.x / texture_width as f32;
                    let max_v = max_uv.y / texture_height as f32;
                    uvs.push([min_u, max_v]);
                    uvs.push([max_u, max_v]);
                    uvs.push([max_u, min_v]);
                    uvs.push([min_u, min_v]);

                    indices.push(index + 0);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index + 2);
                    indices.push(index + 3);
                    indices.push(index + 0);
                    index += 4;
                }
                if let Some(face_uv) = get_face_uv(cube, GeometryBoneCubeFace::South) {
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, -inflate, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, size_y, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, size_y, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, -inflate, -inflate]))
                            .to_array(),
                    );

                    let normal = (quat_rot * Vec3::NEG_Z).to_array();
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);

                    let max_uv = face_uv.0 + face_uv.1;
                    let min_u = face_uv.0.x / texture_width as f32;
                    let min_v = face_uv.0.y / texture_height as f32;
                    let max_u = max_uv.x / texture_width as f32;
                    let max_v = max_uv.y / texture_height as f32;
                    uvs.push([max_u, max_v]);
                    uvs.push([max_u, min_v]);
                    uvs.push([min_u, min_v]);
                    uvs.push([min_u, max_v]);

                    indices.push(index + 0);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index + 2);
                    indices.push(index + 3);
                    indices.push(index + 0);
                    index += 4;
                }
                if let Some(face_uv) = get_face_uv(cube, GeometryBoneCubeFace::East) {
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, -inflate, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, size_y, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, size_y, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, -inflate, size_z]))
                            .to_array(),
                    );

                    let normal = (quat_rot * Vec3::X).to_array();
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);

                    let max_uv = face_uv.0 + face_uv.1;
                    let min_u = face_uv.0.x / texture_width as f32;
                    let min_v = face_uv.0.y / texture_height as f32;
                    let max_u = max_uv.x / texture_width as f32;
                    let max_v = max_uv.y / texture_height as f32;
                    uvs.push([max_u, max_v]);
                    uvs.push([max_u, min_v]);
                    uvs.push([min_u, min_v]);
                    uvs.push([min_u, max_v]);

                    indices.push(index + 0);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index + 2);
                    indices.push(index + 3);
                    indices.push(index + 0);
                    index += 4;
                }
                if let Some(face_uv) = get_face_uv(cube, GeometryBoneCubeFace::West) {
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, -inflate, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, -inflate, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, size_y, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, size_y, -inflate]))
                            .to_array(),
                    );

                    let normal = (quat_rot * Vec3::NEG_X).to_array();
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);

                    let max_uv = face_uv.0 + face_uv.1;
                    let min_u = face_uv.0.x / texture_width as f32;
                    let min_v = face_uv.0.y / texture_height as f32;
                    let max_u = max_uv.x / texture_width as f32;
                    let max_v = max_uv.y / texture_height as f32;
                    uvs.push([min_u, max_v]);
                    uvs.push([max_u, max_v]);
                    uvs.push([max_u, min_v]);
                    uvs.push([min_u, min_v]);

                    indices.push(index + 0);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index + 2);
                    indices.push(index + 3);
                    indices.push(index + 0);
                    index += 4;
                }
                if let Some(face_uv) = get_face_uv(cube, GeometryBoneCubeFace::Up) {
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, size_y, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, size_y, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, size_y, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, size_y, -inflate]))
                            .to_array(),
                    );

                    let normal = (quat_rot * Vec3::Y).to_array();
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);

                    let max_uv = face_uv.0 + face_uv.1;
                    let min_u = face_uv.0.x / texture_width as f32;
                    let min_v = face_uv.0.y / texture_height as f32;
                    let max_u = max_uv.x / texture_width as f32;
                    let max_v = max_uv.y / texture_height as f32;
                    uvs.push([min_u, min_v]);
                    uvs.push([max_u, min_v]);
                    uvs.push([max_u, max_v]);
                    uvs.push([min_u, max_v]);

                    indices.push(index + 0);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index + 2);
                    indices.push(index + 3);
                    indices.push(index + 0);
                    index += 4;
                }
                if let Some(face_uv) = get_face_uv(cube, GeometryBoneCubeFace::Down) {
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, -inflate, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, -inflate, -inflate]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([size_x, -inflate, size_z]))
                            .to_array(),
                    );
                    positions.push(
                        transform
                            .transform_point(Vec3::from([-inflate, -inflate, size_z]))
                            .to_array(),
                    );

                    let normal = (quat_rot * Vec3::NEG_Y).to_array();
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);

                    let max_uv = face_uv.0 + face_uv.1;
                    let min_u = face_uv.0.x / texture_width as f32;
                    let min_v = face_uv.0.y / texture_height as f32;
                    let max_u = max_uv.x / texture_width as f32;
                    let max_v = max_uv.y / texture_height as f32;
                    uvs.push([max_u, min_v]);
                    uvs.push([min_u, min_v]);
                    uvs.push([min_u, max_v]);
                    uvs.push([max_u, max_v]);

                    indices.push(index + 0);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index + 2);
                    indices.push(index + 3);
                    indices.push(index + 0);
                    index += 4;
                }
            }
        }
        if let Some(poly_mesh) = &bone.poly_mesh {
            match &poly_mesh.polys {
                GeometryBonePolyMeshPolys::Tri(primitives) => {
                    for primitive in primitives {
                        for vertex in primitive {
                            positions.push(poly_mesh.positions[vertex.x as usize].to_array());
                            normals.push(poly_mesh.normals[vertex.y as usize].to_array());
                            let uv = poly_mesh.uvs[vertex.z as usize];
                            uvs.push(Vec2::new(uv.x, 1. - uv.y).to_array());
                        }

                        indices.push(index + 2);
                        indices.push(index + 1);
                        indices.push(index + 0);
                        index += 3;
                    }
                }
                GeometryBonePolyMeshPolys::Quad(primitives) => {
                    for primitive in primitives {
                        for vertex in primitive {
                            positions.push(poly_mesh.positions[vertex.x as usize].to_array());
                            let normal = poly_mesh.normals[vertex.y as usize];
                            normals.push(Vec3::new(normal.x, 1. - normal.y, normal.z).to_array());
                            let uv = poly_mesh.uvs[vertex.z as usize];
                            uvs.push(Vec2::new(uv.x, 1. - uv.y).to_array());
                        }

                        indices.push(index + 3);
                        indices.push(index + 2);
                        indices.push(index + 1);
                        indices.push(index + 1);
                        indices.push(index + 0);
                        indices.push(index + 3);
                        index += 4;
                    }
                }
                GeometryBonePolyMeshPolys::None(_) => todo!(),
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        bones.insert(
            bone.name.clone(),
            load_context.set_labeled_asset(&bone.name, LoadedAsset::new(mesh)),
        );
    }
    load_context.set_default_asset(LoadedAsset::new(Skeleton(bones)));

    Ok(())
}

/// https://github.com/MicrosoftDocs/minecraft-creator/blob/main/creator/Reference/Content/SchemasReference/Schemas/minecraftSchema_geometry_1.16.0.md
#[derive(Serialize, Deserialize)]
struct GeometryFile {
    pub format_version: String,
    #[serde(rename = "minecraft:geometry")]
    pub geometry: Vec<Geometry>,
}

#[derive(Serialize, Deserialize)]
struct Geometry {
    pub description: GeometryDescription,

    /// Bones define the 'skeleton' of the mob: the parts that can be animated, and to which geometry and other bones are attached.
    pub bones: Vec<GeometryBone>,
}

#[derive(Serialize, Deserialize)]
struct GeometryDescription {
    /// Entity definition and Client Block definition files refer to this geometry via this identifier.
    pub identifier: String,

    /// Assumed width in texels of the texture that will be bound to this geometry.
    pub texture_width: i32,

    /// Assumed height in texels of the texture that will be bound to this geometry.
    pub texture_height: i32,

    /// Width of the visibility bounding box (in model space units).
    pub visible_bounds_width: Option<f32>,

    /// Height of the visible bounding box (in model space units).
    pub visible_bounds_height: Option<f32>,

    /// Offset of the visibility bounding box from the entity location point (in model space units).
    pub visible_bounds_offset: Option<Vec3>,
}

#[derive(Serialize, Deserialize)]
struct GeometryBone {
    /// Animation files refer to this bone via this identifier.
    pub name: String,

    /// Bone that this bone is relative to.  If the parent bone moves, this bone will move along with it.
    pub parent: Option<String>,

    /// The bone pivots around this point (in model space units).
    pub pivot: Option<Vec3>,

    /// This is the initial rotation of the bone around the pivot, pre-animation (in degrees, x-then-y-then-z order).
    pub rotation: Option<Vec3>,

    /// Mirrors the UV's of the unrotated cubes along the x axis, also causes the east/west faces to get flipped.
    pub mirror: Option<bool>,

    /// Grow this box by this additive amount in all directions (in model space units).
    pub inflate: Option<f32>,

    /// This is the list of cubes associated with this bone.
    pub cubes: Option<Vec<GeometryBoneCube>>,

    /// ***EXPERIMENTAL*** A triangle or quad mesh object.  Can be used in conjunction with cubes and texture geometry.
    pub poly_mesh: Option<GeometryBonePolyMesh>,

    /// ***EXPERIMENTAL*** Adds a mesh to the bone's geometry by converting texels in a texture into boxes
    pub texture_meshes: Option<Vec<GeometryBoneTextureMesh>>,
}

#[derive(Serialize, Deserialize)]
struct GeometryBoneCube {
    /// This point declares the unrotated lower corner of cube (smallest x/y/z value in model space units).
    pub origin: Vec3,

    /// The cube extends this amount relative to its origin (in model space units).
    pub size: Vec3,

    /// The cube is rotated by this amount (in degrees, x-then-y-then-z order) around the pivot.
    pub rotation: Option<Vec3>,

    /// If this field is specified, rotation of this cube occurs around this point, otherwise its rotation is around the center of the box.
    pub pivot: Option<Vec3>,

    /// Grow this box by this additive amount in all directions (in model space units), this field overrides the bone's inflate field for this cube only.
    pub inflate: Option<f32>,

    /// Mirrors this cube about the unrotated x axis (effectively flipping the east / west faces), overriding the bone's 'mirror' setting for this cube.
    pub mirror: Option<bool>,

    pub uv: GeometryBoneCubeUv,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GeometryBoneCubeUv {
    /// Specifies the upper-left corner on the texture for the start of the texture mapping for this box.
    Box(Vec2),

    /// This is an alternate per-face uv mapping which specifies each face of the cube.  Omitting a face will cause that face to not get drawn.
    PerFace(HashMap<GeometryBoneCubeFace, GeometryBoneCubeFaceUv>),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
enum GeometryBoneCubeFace {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

#[derive(Serialize, Deserialize)]
struct GeometryBoneCubeFaceUv {
    /// Specifies the uv origin for the face.
    pub uv: Vec2,

    /// The face maps this many texels from the uv origin. If not specified, the box dimensions are used instead.
    pub uv_size: Option<Vec2>,
}

#[derive(Serialize, Deserialize)]
struct GeometryBonePolyMesh {
    /// If true, UVs are assumed to be [0-1].  If false, UVs are assumed to be [0-texture_width] and [0-texture_height] respectively.
    pub normalized_uvs: bool,

    /// Vertex positions for the mesh.  Can be either indexed via the "polys" section, or be a quad-list if mapped 1-to-1 to the normals and UVs sections.
    pub positions: Vec<Vec3>,

    /// Vertex normals.  Can be either indexed via the "polys" section, or be a quad-list if mapped 1-to-1 to the positions and UVs sections.
    pub normals: Vec<Vec3>,

    /// Vertex UVs.  Can be either indexed via the "polys" section, or be a quad-list if mapped 1-to-1 to the positions and normals sections.
    pub uvs: Vec<Vec2>,
    pub polys: GeometryBonePolyMeshPolys,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GeometryBonePolyMeshPolys {
    /// Poly element indices, as an array of polygons, each an array of either three or four vertices, each an array of indices into positions, normals, and UVs (in that order).
    Tri(Vec<[IVec3; 3]>),
    Quad(Vec<[IVec3; 4]>),

    /// If not specifying vertex indices, arrays of data must be a list of tris or quads, set by making this property either "tri_list" or "quad_list"
    None(GeometryBonePolyMeshPolysType),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GeometryBonePolyMeshPolysType {
    TriList,
    QuadList,
}

#[derive(Serialize, Deserialize)]
struct GeometryBoneTextureMesh {
    /// The friendly-named texture to use.
    pub texture: String,

    /// The position of the pivot point after rotation (in *entity space* not texture or bone space) of the texture geometry
    pub position: Vec3,

    /// The pivot point on the texture (in *texture space* not entity or bone space) of the texture geometry
    pub local_pivot: Vec3,

    /// The rotation (in degrees) of the texture geometry relative to the offset
    pub rotation: Vec3,

    /// The scale (in degrees) of the texture geometry relative to the offset
    pub scale: Vec3,
}
