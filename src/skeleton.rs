use serde::Serialize;

#[derive(Serialize)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
}

#[derive(Serialize)]
pub struct Bone {
    pub pos: [f32; 3],
    pub rot: [f32; 4],
    pub parent: Option<u16>,
}
