use {
    crate::mesh::{IndexOverflow, Mesh, Vert},
    std::{array::TryFromSliceError, fmt},
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
    use dae_parser::{
        ArrayElement, Document, Geometry, GeometryElement, LocalMap, Primitive, Semantic,
    };

    let mut output = Vec::new();

    let document: Document = src.parse().map_err(|_| Error::Document)?;
    let LocalMap(geometry): LocalMap<Geometry> =
        document.local_map().map_err(|_| Error::Geometry)?;

    for (name, geom) in geometry {
        if params.verbose {
            println!("read {name}.. ");
        }

        let GeometryElement::Mesh(mesh) = &geom.element else {
            if params.verbose {
                println!("skip");
            }

            continue;
        };

        let mut positions = Vec::new();
        let mut texture_map = Vec::new();

        for source in &mesh.sources {
            let count = source.accessor.count;
            let stride = source.accessor.stride;

            let id = source.id.as_deref().unwrap_or_default();
            if params.verbose {
                println!("    source {id}");
            }

            match &source.array {
                Some(ArrayElement::Float(arr)) if id.ends_with("-positions") => {
                    positions = Vec::with_capacity(count);
                    for f in arr.chunks(stride) {
                        positions.push(f.try_into()?);
                    }
                }
                Some(ArrayElement::Float(arr)) if id.ends_with("-map-0") => {
                    texture_map = Vec::with_capacity(count);
                    for f in arr.chunks(stride) {
                        texture_map.push(f.try_into()?);
                    }
                }
                _ => {}
            }
        }

        let mut verts = vec![];
        for element in &mesh.elements {
            let Primitive::Triangles(ts) = element else {
                continue;
            };

            let stride = ts.inputs.len();

            let mut offset_texcoord = 0;
            let mut offset_vertex = 0;

            for inp in &ts.inputs.inputs {
                match inp.semantic {
                    Semantic::TexCoord => offset_texcoord = inp.offset,
                    Semantic::Vertex => offset_vertex = inp.offset,
                    _ => {}
                }
            }

            if let Some(data) = &ts.data.prim {
                for n in data.as_ref().chunks(stride) {
                    let index_texcoord = n[offset_texcoord as usize];
                    let index_vertex = n[offset_vertex as usize];

                    let pos = positions[index_vertex as usize];
                    let map = texture_map[index_texcoord as usize];

                    verts.push(Vert {
                        pos: (params.pos_fn)(pos),
                        map: (params.map_fn)(map),
                    });
                }
            }
        }

        let mesh = Mesh::from_verts(&verts)?;
        output.push(Element {
            name: name.to_owned(),
            mesh,
        });

        if params.verbose {
            println!("done");
        }
    }

    Ok(output)
}

pub(crate) enum Error {
    IndexOverflow(IndexOverflow),
    TryFromSlice(TryFromSliceError),
    Document,
    Geometry,
}

impl From<IndexOverflow> for Error {
    fn from(v: IndexOverflow) -> Self {
        Self::IndexOverflow(v)
    }
}

impl From<TryFromSliceError> for Error {
    fn from(v: TryFromSliceError) -> Self {
        Self::TryFromSlice(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IndexOverflow(err) => write!(f, "{err}"),
            Self::TryFromSlice(err) => write!(f, "{err}"),
            Self::Document => write!(f, "failed to parse document"),
            Self::Geometry => write!(f, "failed to parse geometry"),
        }
    }
}
