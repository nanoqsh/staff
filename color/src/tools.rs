use {
    crate::{
        color::Color,
        image::{self as im, Error as ImageError},
        palette::Palette,
    },
    image::Rgb,
    std::{collections::HashSet, fmt},
};

/// Collects color palette from the png image.
///
/// # Errors
/// See [`Error`] for details.
pub fn collect(data: &[u8]) -> Result<Vec<Color>, Error> {
    let im = im::read_png(data)?;
    let mut colors = HashSet::new();
    for &Rgb(col) in im.pixels() {
        colors.insert(Color(col));
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

    let mut im = im::read_png(data)?;
    for Rgb(col) in im.pixels_mut() {
        let target = Color(*col);
        let Color(new) = palette.closest(target);
        *col = new;
    }

    let png = im::write_png(&im)?;
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
