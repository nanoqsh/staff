use {
    crate::mesh::{IndexOverflow, Mesh, Vert},
    std::{
        fmt,
        io::{self, Write},
        iter,
    },
};

pub(crate) struct Parameters<P, M> {
    pub verbose: bool,
    pub pos_fn: P,
    pub map_fn: M,
}

pub(crate) struct Element {
    pub name: String,
    pub mesh: Mesh,
}

pub(crate) fn parse<P, M>(params: Parameters<P, M>, src: &str) -> Result<Vec<Element>, Error>
where
    P: Fn([f32; 3]) -> [f32; 3],
    M: Fn([f32; 2]) -> [f32; 2],
{
    use collada::{document::ColladaDocument, PrimitiveElement, TVertex, Vertex};

    let mut output = Vec::new();

    let document = ColladaDocument::from_str(src).map_err(Error::Document)?;
    let set = document.get_obj_set().ok_or(Error::Geometry)?;
    for object in set.objects {
        if params.verbose {
            println!("read {} .. ", object.name);
            _ = io::stdout().flush();
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
            mesh,
        });

        if params.verbose {
            println!("done");
        }
    }

    Ok(output)
}

pub(crate) enum Error {
    Document(&'static str),
    Geometry,
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
            Self::Geometry => write!(f, "failed to parse geometry"),
            Self::NoTextureMap => write!(f, "the texture map not found"),
            Self::IndexOverflow(err) => write!(f, "{err}"),
        }
    }
}
