use {
    crate::{
        action::{Action, Channel, Interpolation, Keyframe},
        format::{read, Document, Failed, Name, Node},
        mesh::{IndexOverflow, Mesh, Vert},
        params::{verbose, Parameters},
        skeleton::{Bone, Skeleton, ToManyBones},
        target::Target,
    },
    std::fmt,
};

pub struct Element {
    pub name: String,
    pub val: Value,
}

pub enum Value {
    Mesh(Mesh),
    Skeleton(Skeleton),
    Action(Action),
}

pub fn parse(src: &str, target: Target) -> Result<Vec<Element>, Error> {
    let mut output = vec![];
    let doc = read(src)?;

    match target {
        Target::Mesh => parse_meshes(doc, &mut output)?,
        Target::Skeleton => parse_skeletons(doc, &mut output)?,
        Target::Action => parse_actions(doc, &mut output)?,
    }

    Ok(output)
}

fn parse_meshes(doc: Document, output: &mut Vec<Element>) -> Result<(), Error> {
    let params = Parameters::get();
    for geom in doc.geometry {
        verbose!("read {} ({}) .. ", geom.name, geom.id);

        let mut verts = vec![];
        let mut positions_floats = vec![];
        let mut map_floats = vec![];
        for source in geom.sources {
            if source.id.ends_with("-positions") {
                positions_floats = source.floats;
            } else if source.id.ends_with("-map-0") {
                map_floats = source.floats;
            }
        }

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

    Ok(())
}

fn parse_skeletons(doc: Document, output: &mut Vec<Element>) -> Result<(), Error> {
    fn visit_node(node: Node, parent: Option<&str>, sk: &mut Skeleton) -> Result<(), Error> {
        use glam::Mat4;

        match node.ty.as_str() {
            "NODE" => {}
            "JOINT" => {
                let (_, rot, pos) = {
                    let array = node.mat.try_into().map_err(|_| Error::MatSize)?;
                    let mat = Mat4::from_cols_array(&array).transpose();
                    if mat.determinant() == 0. {
                        let name = node.name;
                        eprintln!("failed to parse the bone {name} since it's determinant is zero");
                        return Ok(());
                    }

                    mat.to_scale_rotation_translation()
                };

                let params = Parameters::get();
                sk.push(
                    node.name.clone(),
                    Bone {
                        name: node.name.clone(),
                        pos: (params.pos_fn)(pos.into()),
                        rot: (params.rot_fn)(rot.into()),
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

    for node in doc.nodes {
        verbose!("read {} ({}) .. ", node.name, node.id);

        let name = node.name.clone();
        let mut sk = Skeleton::default();
        visit_node(node, None, &mut sk)?;

        if sk.is_empty() {
            verbose!("skipped {name}");
            continue;
        }

        output.push(Element {
            name,
            val: Value::Skeleton(sk),
        });
    }

    Ok(())
}

fn parse_actions(doc: Document, output: &mut Vec<Element>) -> Result<(), Error> {
    use std::iter;

    fn to_rads(deg: f32) -> f32 {
        use std::f32::consts::PI;

        const M: f32 = PI / 180.;

        deg * M
    }

    let params = Parameters::get();
    let mut action = Action::default();
    for anim in doc.animations {
        if anim.sources.is_empty() {
            continue;
        }

        verbose!("read {} ({}) .. ", anim.name, anim.id);

        let (chan, bone) = {
            let mut parts = anim.id.rsplit("___");
            let chan = match parts.next().ok_or(Error::AnimationId)? {
                "rotation_euler_X" => Channel::RotationX,
                "rotation_euler_Y" => Channel::RotationY,
                "rotation_euler_Z" => Channel::RotationZ,
                _ => return Err(Error::AnimationId),
            };

            let bone = parts.next().ok_or(Error::AnimationId)?.to_owned();
            (chan, bone)
        };

        let mut inputs = vec![];
        let mut outputs = vec![];
        let mut names = vec![];
        let mut intangent = vec![];
        let mut outtangent = vec![];
        for source in anim.sources {
            if source.id.ends_with("-input") {
                inputs = source.floats;
            } else if source.id.ends_with("-output") {
                outputs = source.floats;
            } else if source.id.ends_with("-interpolation") {
                names = source.names;
            } else if source.id.ends_with("-intangent") {
                intangent = source.floats;
            } else if source.id.ends_with("-outtangent") {
                outtangent = source.floats;
            }
        }

        if inputs.len() != outputs.len() || inputs.len() != names.len() {
            return Err(Error::ArrayLen);
        }

        let mut keys = vec![];
        let ns = iter::zip(0.., names);
        let io = iter::zip(inputs, outputs);
        for ((idx, name), (input, output)) in iter::zip(ns, io) {
            let (x, y) = (input, to_rads(output));
            let [input, output] = (params.act_fn)([x, y]);
            let int = match name {
                Name::Linear => Interpolation::Linear,
                Name::Bezier => {
                    let stride = idx * 2;
                    let lx = intangent.get(stride).ok_or(Error::Index)?;
                    let ly = intangent.get(stride + 1).ok_or(Error::Index)?;
                    let rx = outtangent.get(stride).ok_or(Error::Index)?;
                    let ry = outtangent.get(stride + 1).ok_or(Error::Index)?;
                    Interpolation::Bezier((params.bez_fn)([lx - x, ly - y, rx - x, ry - y]))
                }
            };

            keys.push(Keyframe { input, output, int })
        }

        action.push(bone, chan, keys);
    }

    if action.is_empty() {
        verbose!("skipped action");
    }

    output.push(Element {
        name: "action".to_owned(),
        val: Value::Action(action),
    });

    Ok(())
}

pub enum Error {
    Document(Failed),
    NoVertices,
    NoTextureMap,
    Index,
    MatSize,
    ArrayLen,
    AnimationId,
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
            Self::NoVertices => write!(f, "vertices not found"),
            Self::NoTextureMap => write!(f, "the texture map not found"),
            Self::Index => write!(f, "wrong index"),
            Self::MatSize => write!(f, "wrong matrix size"),
            Self::ArrayLen => write!(f, "wrong array length"),
            Self::AnimationId => write!(f, "invalid animation id"),
            Self::UndefinedNode(node) => write!(f, "undefined node {node}"),
            Self::IndexOverflow(err) => write!(f, "{err}"),
            Self::ToManyBones(err) => write!(f, "{err}"),
        }
    }
}
