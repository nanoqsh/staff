use {
    crate::{
        format::{read, Failed, Node},
        mesh::{IndexOverflow, Mesh, Vert},
        skeleton::{Bone, Skeleton, ToManyBones},
    },
    std::fmt,
};

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
        let mut sk = Skeleton::default();
        visit_node(node, None, &mut sk)?;

        if sk.is_empty() {
            if params.verbose {
                println!("skipped {name}");
            }

            continue;
        }

        output.push(Element {
            name,
            val: Value::Skeleton(sk),
        });
    }

    Ok(output)
}

fn visit_node(node: Node, parent: Option<&str>, sk: &mut Skeleton) -> Result<(), Error> {
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

            sk.push(
                node.name.clone(),
                Bone {
                    name: node.name.clone(),
                    pos: pos.into(),
                    rot: rot.into(),
                    parent: parent.and_then(|name| sk.get(name)),
                },
            )?;
        }
        _ => return Err(Error::UndefinedNode(node.ty)),
    }

    for child in node.children {
        visit_node(child, Some(&node.name), sk)?;
    }

    Ok(())
}

pub(crate) enum Error {
    Document(Failed),
    NoSource,
    NoVertices,
    NoTextureMap,
    Index,
    UndefinedNode(String),
    IndexOverflow(IndexOverflow),
    ToManyBones(ToManyBones),
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

impl From<ToManyBones> for Error {
    fn from(v: ToManyBones) -> Self {
        Self::ToManyBones(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Document(err) => write!(f, "failed to parse document: {err}"),
            Self::NoSource => write!(f, "source not found"),
            Self::NoVertices => write!(f, "vertices not found"),
            Self::NoTextureMap => write!(f, "the texture map not found"),
            Self::Index => write!(f, "wrong index"),
            Self::UndefinedNode(node) => write!(f, "undefined node {node}"),
            Self::IndexOverflow(err) => write!(f, "{err}"),
            Self::ToManyBones(err) => write!(f, "{err}"),
        }
    }
}
