use {
    crate::pack::{self, Pack, Rect},
    png::{Error as ImageError, Format, Image},
    std::{collections::BTreeMap, fmt},
};

pub struct ImageData {
    pub name: Box<str>,
    pub data: Vec<u8>,
}

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
        .map(|ImageData { name, data }| match png::decode_png(&data) {
            Ok(image) => Ok(Sprite { image, name }),
            Err(err) => Err(Error { err, name }),
        })
        .collect();

    let mut sprites = sprites?;
    sprites.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    Atlas::pack(sprites)
}

pub struct Atlas {
    pub png: Vec<u8>,
    pub map: BTreeMap<Box<str>, Rect>,
}

impl Atlas {
    fn pack(sprites: Vec<Sprite>) -> Result<Self, Error> {
        use std::iter;

        let entries: Vec<_> = sprites
            .iter()
            .map(|Sprite { image, .. }| image.dimensions())
            .collect();

        let format = sprites
            .iter()
            .map(|Sprite { image, .. }| image.format())
            .max()
            .unwrap_or(Format::Gray);

        let sprites: Vec<_> = sprites
            .into_iter()
            .map(|sprite| Sprite {
                image: sprite.image.into_format(format),
                ..sprite
            })
            .collect();

        let Pack { rects, side } = pack::pack(&entries);
        let mut map = Image::empty((side, side), format);
        for (Sprite { image, .. }, rect) in iter::zip(&sprites, &rects) {
            map.copy_from(image, rect.point());
        }

        Ok(Self {
            png: png::encode_png(&map)?,
            map: sprites
                .into_iter()
                .map(|Sprite { name, .. }| name)
                .zip(rects)
                .collect(),
        })
    }
}

struct Sprite {
    image: Image,
    name: Box<str>,
}

pub struct Error {
    err: ImageError,
    name: Box<str>,
}

impl From<ImageError> for Error {
    fn from(err: ImageError) -> Self {
        Self {
            err,
            name: Box::default(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { err, name } = self;
        if name.is_empty() {
            write!(f, "{err}")
        } else {
            write!(f, "with an image {name:?}: {err}")
        }
    }
}
