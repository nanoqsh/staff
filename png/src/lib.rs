use {
    image::{
        codecs::png::{PngDecoder, PngEncoder},
        ColorType, DynamicImage, GrayImage, ImageEncoder, ImageError, RgbImage, RgbaImage,
    },
    std::fmt,
};

pub enum Image {
    Gray(GrayImage),
    Rgb(RgbImage),
    Rgba(RgbaImage),
}

impl Image {
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

    fn color_type(&self) -> ColorType {
        match self {
            Self::Gray(_) => ColorType::L8,
            Self::Rgb(_) => ColorType::Rgb8,
            Self::Rgba(_) => ColorType::Rgba8,
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

    #[must_use]
    pub fn into_rgb(self) -> RgbImage {
        match self {
            Self::Gray(im) => DynamicImage::from(im).into_rgb8(),
            Self::Rgb(im) => im,
            Self::Rgba(im) => DynamicImage::from(im).into_rgb8(),
        }
    }
}

/// Reads the png image from bytes.
///
/// # Errors
/// See [`Error`] for details.
pub fn read_png(data: &[u8]) -> Result<Image, Error> {
    let decoder = PngDecoder::new(data)?;
    let im = DynamicImage::from_decoder(decoder)?;
    Image::from_dynamic(im)
}

/// Writes the png image in a bytes buffer.
///
/// # Errors
/// See [`Error`] for details.
pub fn write_png(im: &Image) -> Result<Vec<u8>, Error> {
    const DEFAULT_BUFFER_CAP: usize = 256;

    let mut buf = Vec::with_capacity(DEFAULT_BUFFER_CAP);
    let encoder = PngEncoder::new(&mut buf);
    let (width, height) = im.dimensions();
    encoder.write_image(im.as_bytes(), width, height, im.color_type())?;
    Ok(buf)
}

pub enum Error {
    Image(ImageError),
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
