use {serde::Serialize, std::collections::BTreeMap};

type Animations = BTreeMap<String, Vec<Animation>>;

#[derive(Default)]
pub struct Action {
    animations: Animations,
}

impl Action {
    pub(crate) fn push(&mut self, bone: String, chan: Channel, keys: Vec<Keyframe>) {
        self.animations
            .entry(bone)
            .or_default()
            .push(Animation { chan, keys });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }

    pub fn animations(&self) -> &Animations {
        &self.animations
    }
}

#[derive(Serialize)]
pub struct Animation {
    chan: Channel,
    keys: Vec<Keyframe>,
}

#[derive(Serialize)]
pub(crate) enum Channel {
    #[serde(rename = "rx")]
    RotationX,
    #[serde(rename = "ry")]
    RotationY,
    #[serde(rename = "rz")]
    RotationZ,
}

#[derive(Clone, Copy, Serialize)]
#[serde(into = "(f32, f32, Interpolation)")]
pub(crate) struct Keyframe {
    pub input: f32,
    pub output: f32,
    pub int: Interpolation,
}

impl From<Keyframe> for (f32, f32, Interpolation) {
    fn from(Keyframe { input, output, int }: Keyframe) -> Self {
        (input, output, int)
    }
}

#[derive(Clone, Copy, Serialize)]
pub(crate) enum Interpolation {
    #[serde(rename = "l")]
    Linear,
    #[serde(rename = "b")]
    Bezier([f32; 4]),
}
