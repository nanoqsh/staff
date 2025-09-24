use {
    crate::{
        color::Color,
        palette::{Closest, Exact},
    },
    im::{Error as ImageError, Image, Rgb},
    std::{collections::HashSet, fmt},
};

/// Collects color palette from the png image.
///
/// # Errors
/// See [`Error`] for details.
pub fn collect(data: &[u8], sort: bool) -> Result<Vec<Color>, Error> {
    let im = im::decode_png(data)?.into_rgb();
    let mut colors = HashSet::new();
    let mut out = vec![];
    for Rgb(rgb) in im.pixels() {
        let col = Color(*rgb);
        if colors.insert(col) {
            out.push(col);
        }
    }

    if sort {
        out.sort_unstable();
    }

    Ok(out)
}

pub enum RepaintMode<'palette> {
    Closest {
        colors: &'palette [Color],
    },
    Exact {
        from: &'palette [Color],
        to: &'palette [Color],
    },
}

/// Repaints the png image with given palette colors.
///
/// # Errors
/// See [`Error`] for details.
pub fn repaint(data: &[u8], mode: RepaintMode<'_>) -> Result<Vec<u8>, Error> {
    let transfer: &mut dyn FnMut(_) -> _ = match mode {
        RepaintMode::Closest { colors } => {
            if colors.is_empty() {
                return Err(Error::EmptyPalette);
            }

            let mut palette = Closest::new(colors);
            &mut move |target| Some(palette.transfer(target))
        }
        RepaintMode::Exact { from, to } => {
            if from.is_empty() || to.is_empty() {
                return Err(Error::EmptyPalette);
            }

            let mut palette = Exact::new(from, to);
            &mut move |target| palette.transfer(target)
        }
    };

    let mut im = im::decode_png(data)?.into_rgb();
    for Rgb(rgb) in im.pixels_mut() {
        let target = Color(*rgb);
        let Color(new) = transfer(target).ok_or(Error::TranferFailed(target))?;
        *rgb = new;
    }

    let im = Image::Rgb(im);
    let png = im::encode_png(&im)?;
    Ok(png)
}

pub enum Error {
    Image(ImageError),
    EmptyPalette,
    TranferFailed(Color),
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
            Self::TranferFailed(col) => write!(f, "failed to transfer color {col}"),
        }
    }
}
