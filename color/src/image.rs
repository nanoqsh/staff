use {
    image::{
        codecs::png::{PngDecoder, PngEncoder},
        ColorType, DynamicImage, ImageEncoder, ImageError, RgbImage,
    },
    std::fmt,
};

pub(crate) fn read_png(data: &[u8]) -> Result<RgbImage, Error> {
    let decoder = PngDecoder::new(data)?;
    match DynamicImage::from_decoder(decoder)? {
        DynamicImage::ImageRgb8(d) => Ok(d),
        im @ (DynamicImage::ImageLuma8(_)
        | DynamicImage::ImageLumaA8(_)
        | DynamicImage::ImageRgba8(_)) => Ok(im.to_rgb8()),
        _ => Err(Error::UnsupportedFormat),
    }
}

pub(crate) fn write_png(data: &RgbImage) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::with_capacity(128);
    let encoder = PngEncoder::new(&mut buf);
    let (width, height) = data.dimensions();
    encoder.write_image(data, width, height, ColorType::Rgb8)?;
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
