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
    pub(crate) fn push(&mut self, bone: Bone) -> Result<(), ToManyBones> {
        let idx = self.bones.len().try_into().map_err(|_| ToManyBones)?;
        assert!(bone.parent < Some(idx), "parent nodes must come first");

        self.names.insert(bone.name.clone(), idx);
        self.bones.push(bone);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push() {
        let mut skeleton = Skeleton::default();
        assert!(skeleton.push(Bone::with_parent(None)).is_ok());
        assert!(skeleton.push(Bone::with_parent(Some(0))).is_ok());
        assert!(skeleton.push(Bone::with_parent(Some(0))).is_ok());
        assert!(skeleton.push(Bone::with_parent(Some(1))).is_ok());
        assert!(skeleton.push(Bone::with_parent(Some(2))).is_ok());
    }

    impl Bone {
        fn with_parent(parent: Option<u16>) -> Self {
            Self {
                name: String::default(),
                pos: [0.; 3],
                rot: [0.; 4],
                parent,
            }
        }
    }
}
