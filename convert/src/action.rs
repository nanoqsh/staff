use {serde::Serialize, std::collections::HashMap};

type Animations = HashMap<String, Vec<Animation>>;

#[derive(Default)]
pub struct Action {
    pub animations: Animations,
}

impl Action {
    pub fn animations(&self) -> &Animations {
        &self.animations
    }
}

#[derive(Serialize)]
pub struct Animation {
    pub chan: Channel,
    pub keys: Vec<Keyframe>,
}

#[derive(Serialize)]
pub enum Channel {
    #[serde(rename = "rx")]
    RotationX,
    #[serde(rename = "ry")]
    RotationY,
    #[serde(rename = "rz")]
    RotationZ,
}

#[derive(Serialize)]
pub struct Keyframe {
    #[serde(rename = "x")]
    pub input: f32,
    #[serde(rename = "y")]
    pub output: f32,
    #[serde(rename = "i")]
    pub interpolation: Interpolation,
}

#[derive(Serialize)]
pub enum Interpolation {
    #[serde(rename = "l")]
    Linear,
    #[serde(rename = "b")]
    Bezier { l: [f32; 2], r: [f32; 2] },
}
