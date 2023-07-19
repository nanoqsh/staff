use {
    crate::pack::{self, Pack, Rect},
    png::{Error as ImageError, Format, Image},
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
        .map(|ImageData { name, data }| match png::decode_png(&data) {
            Ok(image) => Ok(Sprite { image, name }),
            Err(err) => Err(Error { err, name }),
        })
        .collect();

    let mut sprites = sprites?;
    sprites.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    Ok(Atlas::pack(sprites))
}

pub struct ImageData {
    pub name: Box<str>,
    pub data: Vec<u8>,
}

pub struct Atlas {
    map: Image,
}

impl Atlas {
    fn pack(sprites: Vec<Sprite>) -> Self {
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
        for (sprite, Rect { point, .. }) in iter::zip(sprites, rects) {
            map.copy_from(&sprite.image, point);
        }

        Self { map }
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { err, name } = self;
        write!(f, "with an image {name:?}: {err}")
    }
}
