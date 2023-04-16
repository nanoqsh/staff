use std::{error, fmt, str};

#[derive(Clone, Copy)]
pub enum Target {
    Mesh,
    Skeleton,
    Action,
}

impl str::FromStr for Target {
    type Err = Unknown;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mesh" => Ok(Self::Mesh),
            "skeleton" => Ok(Self::Skeleton),
            "action" => Ok(Self::Action),
            _ => Err(Unknown),
        }
    }
}

#[derive(Debug)]
pub struct Unknown;

impl fmt::Display for Unknown {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown target")
    }
}

impl error::Error for Unknown {}
