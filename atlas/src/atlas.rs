use {
    png::{Error as ImageError, Image},
    std::fmt,
};

/// Make an atlas from images.
///
/// # Errors
/// See [`Error`] type for details.
pub fn atlas<D>(data: D) -> Result<Atlas, Error>
where
    D: IntoIterator<Item = ImageData>,
{
    let sprites: Result<Vec<Sprite>, Error> = data
        .into_iter()
        .map(|ImageData { name, data }| match png::read_png(&data) {
            Ok(image) => Ok(Sprite { image, name }),
            Err(err) => Err(Error::Image { err, name }),
        })
        .collect();

    let mut sprites = sprites?;
    sprites.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    for sprite in sprites {
        // TODO:
        _ = sprite.image;
    }

    Ok(Atlas)
}

struct Sprite {
    image: Image,
    name: Box<str>,
}

pub struct ImageData {
    pub name: Box<str>,
    pub data: Vec<u8>,
}

pub struct Atlas;

pub enum Error {
    Image { err: ImageError, name: Box<str> },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Image { err, name } => write!(f, "with image {name:?}: {err}"),
        }
    }
}
