use {
    crate::{color::Color, palette::Palette},
    im::{Error as ImageError, Image, Rgb},
    std::{collections::HashSet, fmt},
};

/// Collects color palette from the png image.
///
/// # Errors
/// See [`Error`] for details.
pub fn collect(data: &[u8]) -> Result<Vec<Color>, Error> {
    let im = im::decode_png(data)?.into_rgb();
    let mut colors = HashSet::new();
    for Rgb(rgb) in im.pixels() {
        colors.insert(Color(*rgb));
    }

    let mut colors: Vec<_> = colors.into_iter().collect();
    colors.sort_unstable();
    Ok(colors)
}

/// Repaints the png image with given palette colors.
///
/// # Errors
/// See [`Error`] for details.
pub fn repaint(data: &[u8], colors: &[Color]) -> Result<Vec<u8>, Error> {
    let mut palette = Palette::new(colors);
    if palette.is_empty() {
        return Err(Error::EmptyPalette);
    }

    let mut im = im::decode_png(data)?.into_rgb();
    for Rgb(rgb) in im.pixels_mut() {
        let Color(new) = palette.closest(Color(*rgb));
        *rgb = new;
    }

    let im = Image::Rgb(im);
    let png = im::encode_png(&im)?;
    Ok(png)
}

pub enum Error {
    Image(ImageError),
    EmptyPalette,
}

impl From<ImageError> for Error {
    fn from(v: ImageError) -> Self {
        Self::Image(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Image(err) => write!(f, "{err}"),
            Self::EmptyPalette => write!(f, "empty palette"),
        }
    }
}
