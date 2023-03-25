use {
    crate::{
        mesh::{IndexOverflow, Mesh, Vert},
        skeleton::{Bone, Skeleton},
    },
    collada::ObjSet,
    std::{fmt, iter},
};

type SkSet = Vec<collada::Skeleton>;

#[derive(Clone, Copy)]
pub(crate) struct Parameters<'a> {
    pub verbose: bool,
    pub pos_fn: &'a dyn Fn([f32; 3]) -> [f32; 3],
    pub map_fn: &'a dyn Fn([f32; 2]) -> [f32; 2],
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
    use collada::document::ColladaDocument;

    let mut output = Vec::new();
    let document = ColladaDocument::from_str(src).map_err(Error::Document)?;

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
    Document(&'static str),
    NoTextureMap,
    IndexOverflow(IndexOverflow),
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
            Self::NoTextureMap => write!(f, "the texture map not found"),
            Self::IndexOverflow(err) => write!(f, "{err}"),
        }
    }
}
