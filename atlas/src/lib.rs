mod atlas;
mod pack;

pub use crate::{
    atlas::{make, Atlas, Error, ImageData, Map},
    pack::{Margin, TooLarge},
};
