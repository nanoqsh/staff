use {
    serde::Serialize,
    std::{collections::HashMap, fmt},
};

#[derive(Default)]
pub struct Skeleton {
    bones: Vec<Bone>,
    names: HashMap<String, u16>,
}

impl Skeleton {
    pub(crate) fn push(&mut self, name: String, bone: Bone) -> Result<(), ToManyBones> {
        let id = self.bones.len().try_into().map_err(|_| ToManyBones)?;
        self.bones.push(bone);
        self.names.insert(name, id);

        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.bones.is_empty()
    }

    pub(crate) fn get(&self, name: &str) -> Option<u16> {
        self.names.get(name).copied()
    }

    pub fn bones(&self) -> &[Bone] {
        &self.bones
    }
}

pub struct ToManyBones;

impl fmt::Display for ToManyBones {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "to many bones")
    }
}

#[derive(Serialize)]
pub struct Bone {
    pub name: String,
    pub pos: [f32; 3],
    pub rot: [f32; 4],
    pub parent: Option<u16>,
}
