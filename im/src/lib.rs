use {
    image::{
        codecs::png::{PngDecoder, PngEncoder},
        ColorType, DynamicImage, GrayImage, ImageEncoder, ImageError, RgbImage, RgbaImage,
    },
    std::fmt,
};

use image::GenericImage;
pub use image::Rgb;

/// The image format.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Format {
    Gray = 1,
    Rgb = 3,
    Rgba = 4,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Gray => write!(f, "gray"),
            Self::Rgb => write!(f, "rgb"),
            Self::Rgba => write!(f, "rgba"),
        }
    }
}

impl From<Format> for ColorType {
    fn from(format: Format) -> Self {
        match format {
            Format::Gray => Self::L8,
            Format::Rgb => Self::Rgb8,
            Format::Rgba => Self::Rgba8,
        }
    }
}

#[must_use]
pub enum Image {
    Gray(GrayImage),
    Rgb(RgbImage),
    Rgba(RgbaImage),
}

impl Image {
    pub fn empty((width, height): (u32, u32), format: Format) -> Self {
        match format {
            Format::Gray => Self::Gray(GrayImage::new(width, height)),
            Format::Rgb => Self::Rgb(RgbImage::new(width, height)),
            Format::Rgba => Self::Rgba(RgbaImage::new(width, height)),
        }
    }

    fn from_dynamic(im: DynamicImage) -> Result<Self, Error> {
        match im {
            DynamicImage::ImageLuma8(im) => Ok(Self::Gray(im)),
            DynamicImage::ImageRgb8(im) => Ok(Self::Rgb(im)),
            DynamicImage::ImageRgba8(im) => Ok(Self::Rgba(im)),
            _ => Err(Error::UnsupportedFormat),
        }
    }

    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Gray(im) => im,
            Self::Rgb(im) => im,
            Self::Rgba(im) => im,
        }
    }

    #[must_use]
    pub fn format(&self) -> Format {
        match self {
            Self::Gray(_) => Format::Gray,
            Self::Rgb(_) => Format::Rgb,
            Self::Rgba(_) => Format::Rgba,
        }
    }

    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Gray(im) => im.dimensions(),
            Self::Rgb(im) => im.dimensions(),
            Self::Rgba(im) => im.dimensions(),
        }
    }

    /// Copies another image into this image.
    ///
    /// # Panics
    /// This function panics if the image formats are different.
    pub fn copy_from(&mut self, from: &Self, (x, y): (u32, u32)) {
        let copied = match (self, from) {
            (Self::Gray(im), Self::Gray(from)) => im.copy_from(from, x, y),
            (Self::Rgb(im), Self::Rgb(from)) => im.copy_from(from, x, y),
            (Self::Rgba(im), Self::Rgba(from)) => im.copy_from(from, x, y),
            _ => panic!("different image formats"),
        };

        copied.expect("copy image");
    }

    #[must_use]
    pub fn into_gray(self) -> GrayImage {
        match self {
            Self::Gray(im) => im,
            Self::Rgb(im) => DynamicImage::from(im).into_luma8(),
            Self::Rgba(im) => DynamicImage::from(im).into_luma8(),
        }
    }

    #[must_use]
    pub fn into_rgb(self) -> RgbImage {
        match self {
            Self::Gray(im) => DynamicImage::from(im).into_rgb8(),
            Self::Rgb(im) => im,
            Self::Rgba(im) => DynamicImage::from(im).into_rgb8(),
        }
    }

    #[must_use]
    pub fn into_rgba(self) -> RgbaImage {
        match self {
            Self::Gray(im) => DynamicImage::from(im).into_rgba8(),
            Self::Rgb(im) => DynamicImage::from(im).into_rgba8(),
            Self::Rgba(im) => im,
        }
    }

    pub fn into_format(self, format: Format) -> Self {
        if self.format() == format {
            return self;
        }

        match format {
            Format::Gray => Self::Gray(self.into_gray()),
            Format::Rgb => Self::Rgb(self.into_rgb()),
            Format::Rgba => Self::Rgba(self.into_rgba()),
        }
    }
}

/// Decodes the png image from bytes.
///
/// # Errors
/// See [`Error`] for details.
pub fn decode_png(data: &[u8]) -> Result<Image, Error> {
    let decoder = PngDecoder::new(data)?;
    let im = DynamicImage::from_decoder(decoder)?;
    Image::from_dynamic(im)
}

/// Encodes the png image in a bytes buffer.
///
/// # Errors
/// See [`Error`] for details.
pub fn encode_png(im: &Image) -> Result<Vec<u8>, Error> {
    const DEFAULT_BUFFER_CAP: usize = 256;

    let mut buf = Vec::with_capacity(DEFAULT_BUFFER_CAP);
    let encoder = PngEncoder::new(&mut buf);
    let (width, height) = im.dimensions();
    encoder.write_image(im.as_bytes(), width, height, im.format().into())?;
    Ok(buf)
}

/// The png image error.
pub enum Error {
    /// Error while working png data.
    Image(ImageError),

    /// A format is not supported.
    UnsupportedFormat,
}

impl From<ImageError> for Error {
    fn from(v: ImageError) -> Self {
        Self::Image(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Image(err) => write!(f, "image error: {err}"),
            Self::UnsupportedFormat => write!(f, "unsupported format"),
        }
    }
}
