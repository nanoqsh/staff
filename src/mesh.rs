use {
    serde::Serialize,
    std::{
        fmt,
        hash::{Hash, Hasher},
    },
};

type Face = [u16; 3];

#[derive(Serialize)]
pub(crate) struct Mesh {
    pub verts: Vec<Vert>,
    pub faces: Vec<Face>,
}

impl Mesh {
    pub fn from_verts(verts: &[[Vert; 3]]) -> Result<Self, IndexOverflow> {
        Self::make_indices(verts)
    }

    fn make_indices(verts: &[[Vert; 3]]) -> Result<Self, IndexOverflow> {
        use std::collections::HashMap;

        let mut indxs_map = HashMap::with_capacity(verts.len() / 2);
        let mut new_verts = Vec::with_capacity(verts.len() / 2);
        let faces = verts
            .iter()
            .map(|verts| {
                verts.map(|vert| {
                    let new_index = indxs_map.len() as u16;
                    let &mut index = indxs_map.entry(vert).or_insert_with(|| {
                        new_verts.push(vert);
                        new_index
                    });
                    index
                })
            })
            .collect();

        if indxs_map.len() > u16::MAX as usize {
            return Err(IndexOverflow);
        }

        Ok(Self {
            verts: new_verts,
            faces,
        })
    }
}

pub(crate) struct IndexOverflow;

impl fmt::Display for IndexOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "index overflow")
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(into = "[f32; 5]")]
pub(crate) struct Vert {
    pub pos: [f32; 3],
    pub map: [f32; 2],
}

impl From<Vert> for [f32; 5] {
    fn from(vert: Vert) -> Self {
        let Vert {
            pos: [x, y, z],
            map: [u, v],
        } = vert;
        [x, y, z, u, v]
    }
}

impl PartialEq for Vert {
    fn eq(&self, other: &Self) -> bool {
        self.pos.map(f32::to_ne_bytes) == other.pos.map(f32::to_ne_bytes)
            && self.map.map(f32::to_ne_bytes) == other.map.map(f32::to_ne_bytes)
    }
}

impl Eq for Vert {}

impl Hash for Vert {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.pos.map(f32::to_ne_bytes).hash(state);
        self.map.map(f32::to_ne_bytes).hash(state);
    }
}
