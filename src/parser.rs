use std::collections::HashMap;

use {
    crate::{
        format::{read, Failed, Node},
        mesh::{IndexOverflow, Mesh, Vert},
        skeleton::{Bone, Skeleton},
    },
    collada::ObjSet,
    std::{fmt, iter},
};

type SkSet = Vec<collada::Skeleton>;

#[derive(Clone, Copy)]
pub(crate) struct Parameters {
    pub verbose: bool,
    pub pos_fn: fn([f32; 3]) -> [f32; 3],
    pub map_fn: fn([f32; 2]) -> [f32; 2],
    pub rot_fn: fn([f32; 4]) -> [f32; 4],
}

pub(crate) struct Element {
    pub name: String,
    pub val: Value,
}

pub(crate) enum Value {
    Mesh(Mesh),
    Skeleton(Skeleton),
}

pub(crate) fn parse(params: Parameters, src: &str) -> Result<Vec<Element>, Error> {
    let mut output = Vec::new();
    let doc = read(src)?;

    for geom in doc.geometry {
        if params.verbose {
            println!("read {} ({}) .. ", geom.name, geom.id);
        }

        let mut verts = Vec::new();
        let mut positions_floats = None;
        let mut map_floats = None;
        for source in geom.sources {
            if source.id.ends_with("-positions") {
                positions_floats = Some(source.floats);
            } else if source.id.ends_with("-map-0") {
                map_floats = Some(source.floats);
            }
        }

        let Some(positions_floats) = positions_floats else {
            return Err(Error::NoSource);
        };

        let Some(map_floats) = map_floats else {
            return Err(Error::NoSource);
        };

        let mut max_offset = 1;
        let mut vertices_input = None;
        let mut map_input = None;
        for input in geom.triangles.inputs {
            if input.source.ends_with("-vertices") {
                vertices_input = Some(input.offset);
            } else if input.source.ends_with("-map-0") {
                map_input = Some(input.offset);
            }

            let offset = input.offset + 1;
            if offset > max_offset {
                max_offset = offset;
            }
        }

        let Some(vertices_input) = vertices_input else {
            return Err(Error::NoVertices);
        };

        let Some(map_input) = map_input else {
            return Err(Error::NoTextureMap);
        };

        for index_chunk in geom.triangles.indxs.chunks(max_offset) {
            let pos = *index_chunk.get(vertices_input).ok_or(Error::Index)? as usize;
            let map = *index_chunk.get(map_input).ok_or(Error::Index)? as usize;

            let pos_stride = pos * 3;
            let x = *positions_floats.get(pos_stride).ok_or(Error::Index)?;
            let y = *positions_floats.get(pos_stride + 1).ok_or(Error::Index)?;
            let z = *positions_floats.get(pos_stride + 2).ok_or(Error::Index)?;
            let map_stride = map * 2;
            let u = *map_floats.get(map_stride).ok_or(Error::Index)?;
            let v = *map_floats.get(map_stride + 1).ok_or(Error::Index)?;

            verts.push(Vert {
                pos: (params.pos_fn)([x, y, z]),
                map: (params.map_fn)([u, v]),
            });
        }

        let verts: Vec<_> = verts
            .chunks_exact(3)
            .map(|tri| match tri {
                &[a, b, c] => [a, b, c],
                _ => unreachable!(),
            })
            .collect();

        let mesh = Mesh::from_verts(&verts)?;
        output.push(Element {
            name: geom.name,
            val: Value::Mesh(mesh),
        });
    }

    for node in doc.nodes {
        if params.verbose {
            println!("read {} ({}) .. ", node.name, node.id);
        }

        let name = node.name.clone();
        let mut bones = Bones::default();
        visit_node(node, None, &mut bones)?;

        if bones.is_empty() {
            if params.verbose {
                println!("skipped {name}");
            }

            continue;
        }

        output.push(Element {
            name,
            val: Value::Skeleton(bones.into_skeleton()),
        });
    }

    Ok(output)
}

fn visit_node(node: Node, parent: Option<&str>, bones: &mut Bones) -> Result<(), Error> {
    use glam::Mat4;

    match node.ty.as_str() {
        "NODE" => {}
        "JOINT" => {
            let (_, rot, pos) = {
                let array = node.mat.try_into().map_err(|_| Error::Index)?;
                let mat = Mat4::from_cols_array(&array);
                if mat.determinant() == 0. {
                    let name = node.name;
                    eprintln!("failed to parse the bone {name} since it's determinant is zero");
                    return Ok(());
                }

                mat.to_scale_rotation_translation()
            };

            bones.push(
                node.name.clone(),
                Bone {
                    pos: pos.into(),
                    rot: rot.into(),
                    parent: parent.and_then(|name| bones.get(name)),
                },
            )?;
        }
        _ => return Err(Error::UndefinedNode(node.ty)),
    }

    for child in node.children {
        visit_node(child, Some(&node.name), bones)?;
    }

    Ok(())
}

#[derive(Default)]
struct Bones {
    bones: Vec<Bone>,
    names: HashMap<String, u16>,
}

impl Bones {
    fn into_skeleton(self) -> Skeleton {
        Skeleton { bones: self.bones }
    }

    fn push(&mut self, name: String, bone: Bone) -> Result<(), Error> {
        let id = self
            .bones
            .len()
            .try_into()
            .map_err(|_| Error::ToManyBones)?;

        self.bones.push(bone);
        self.names.insert(name, id);
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.bones.is_empty()
    }

    fn get(&self, name: &str) -> Option<u16> {
        self.names.get(name).copied()
    }
}

pub(crate) fn parse_(params: Parameters, src: &str) -> Result<Vec<Element>, Error> {
    use collada::document::ColladaDocument;

    let mut output = Vec::new();
    let document = ColladaDocument::from_str(src).unwrap();

    if let Some(set) = document.get_obj_set() {
        output.append(&mut parse_objects(params, set)?);
    }

    if let Some(set) = document.get_skeletons() {
        output.append(&mut parse_skeletons(params, set));
    }

    Ok(output)
}

fn parse_objects(params: Parameters, set: ObjSet) -> Result<Vec<Element>, Error> {
    use collada::{PrimitiveElement, TVertex, Vertex};

    let mut output = Vec::new();

    for object in set.objects {
        if params.verbose {
            println!("read {} .. ", object.name);
        }

        let get_vert = |pi, ti| {
            let Vertex { x, y, z } = object.vertices[pi];
            let TVertex { x: u, y: v } = object.tex_vertices[ti];

            Vert {
                pos: (params.pos_fn)([x as f32, y as f32, z as f32]),
                map: (params.map_fn)([u as f32, v as f32]),
            }
        };

        let mut verts = vec![];
        for geometry in object.geometry {
            for element in geometry.mesh {
                let PrimitiveElement::Triangles(triangles) = element else {
                    continue;
                };

                let pverts = triangles.vertices;
                let tverts = triangles.tex_vertices.ok_or(Error::NoTextureMap)?;
                for ((ap, bp, cp), (at, bt, ct)) in iter::zip(pverts, tverts) {
                    verts.push([get_vert(ap, at), get_vert(bp, bt), get_vert(cp, ct)]);
                }
            }
        }

        let mesh = Mesh::from_verts(&verts)?;
        output.push(Element {
            name: object.name,
            val: Value::Mesh(mesh),
        });
    }

    Ok(output)
}

fn parse_skeletons(params: Parameters, set: SkSet) -> Vec<Element> {
    use glam::Mat4;

    let mut output = Vec::new();

    for skeleton in set {
        if params.verbose {
            println!("read skeleton .. ");
        }

        let mut bones = Vec::new();
        for (mat, joint) in iter::zip(skeleton.bind_poses, skeleton.joints) {
            let (_, rot, pos) = {
                let mat = Mat4::from_cols_array_2d(&mat);
                if mat.determinant() == 0. {
                    let name = joint.name;
                    eprintln!("failed to parse the bone {name} since it's determinant is zero");
                    continue;
                }

                mat.to_scale_rotation_translation()
            };

            bones.push(Bone {
                pos: pos.into(),
                rot: rot.into(),
                parent: (!joint.is_root()).then_some(joint.parent_index as u16),
            });
        }

        output.push(Element {
            name: "skeleton".to_owned(),
            val: Value::Skeleton(Skeleton { bones }),
        });
    }

    output
}

pub(crate) enum Error {
    Document(Failed),
    NoSource,
    NoVertices,
    NoTextureMap,
    IndexOverflow(IndexOverflow),
    Index,
    UndefinedNode(String),
    ToManyBones,
}

impl From<Failed> for Error {
    fn from(v: Failed) -> Self {
        Self::Document(v)
    }
}

impl From<IndexOverflow> for Error {
    fn from(v: IndexOverflow) -> Self {
        Self::IndexOverflow(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Document(err) => write!(f, "failed to parse document: {err}"),
            Self::NoSource => write!(f, "source not found"),
            Self::NoVertices => write!(f, "vertices not found"),
            Self::NoTextureMap => write!(f, "the texture map not found"),
            Self::IndexOverflow(err) => write!(f, "{err}"),
            Self::Index => write!(f, "wrong index"),
            Self::UndefinedNode(node) => write!(f, "undefined node {node}"),
            Self::ToManyBones => write!(f, "to many bones"),
        }
    }
}
