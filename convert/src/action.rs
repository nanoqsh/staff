use {serde::Serialize, std::collections::BTreeMap};

type Animations = BTreeMap<String, Vec<Keyframe>>;

#[derive(Default)]
pub struct Action {
    animations: Animations,
}

impl Action {
    pub(crate) fn insert_channel(&mut self, bone: String, input: f32, chan: Channel) {
        let keys = self.animations.entry(bone).or_default();
        match keys.binary_search_by(|key| key.input.total_cmp(&input)) {
            Ok(idx) => {
                let key = &mut keys[idx];
                key.val = key.val.with(chan);
            }
            Err(idx) => keys.insert(
                idx,
                Keyframe {
                    input,
                    val: Value::default().with(chan),
                },
            ),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }

    pub fn animations(&self) -> &Animations {
        &self.animations
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(into = "(f32, Value)")]
pub struct Keyframe {
    input: f32,
    val: Value,
}

impl From<Keyframe> for (f32, Value) {
    fn from(Keyframe { input, val }: Keyframe) -> Self {
        (input, val)
    }
}

#[derive(Clone, Copy, Default, Serialize)]
struct Value {
    pub rx: Rotation,
    pub ry: Rotation,
    pub rz: Rotation,
}

impl Value {
    fn with(mut self, chan: Channel) -> Self {
        match chan {
            Channel::RotationX(rx) => self.rx = rx,
            Channel::RotationY(ry) => self.ry = ry,
            Channel::RotationZ(rz) => self.rz = rz,
        }

        self
    }
}

#[derive(Clone, Copy, Default, Serialize)]
#[serde(into = "(f32, Interpolation)")]
pub(crate) struct Rotation {
    pub output: f32,
    pub int: Interpolation,
}

impl From<Rotation> for (f32, Interpolation) {
    fn from(Rotation { output, int }: Rotation) -> Self {
        (output, int)
    }
}

#[derive(Clone, Copy, Default, Serialize)]
pub(crate) enum Interpolation {
    #[default]
    #[serde(rename = "l")]
    Linear,
    #[serde(rename = "b")]
    Bezier([f32; 4]),
}

pub(crate) enum Channel {
    RotationX(Rotation),
    RotationY(Rotation),
    RotationZ(Rotation),
}
