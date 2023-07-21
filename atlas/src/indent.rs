use std::fmt;

#[derive(Clone, Copy)]
pub struct Indent {
    pub(crate) horizontal: u32,
    pub(crate) vertical: u32,
}

impl Indent {
    const MAX_HORIZONTAL: u32 = 4;
    const MAX_VERTICAL: u32 = 4;

    /// Creates a new indent.
    ///
    /// # Errors
    /// This function returns an [error](TooLarge) if the indent is too large.
    pub const fn new(horizontal: u32, vertical: u32) -> Result<Self, TooLarge> {
        if horizontal > Self::MAX_HORIZONTAL {
            return Err(TooLarge::Horizontal(horizontal));
        }

        if vertical > Self::MAX_VERTICAL {
            return Err(TooLarge::Vertical(vertical));
        }

        Ok(Self {
            horizontal,
            vertical,
        })
    }
}

pub enum TooLarge {
    Horizontal(u32),
    Vertical(u32),
}

impl fmt::Display for TooLarge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ty, value, max) = match self {
            Self::Horizontal(value) => ("horizontal", value, Indent::MAX_HORIZONTAL),
            Self::Vertical(value) => ("vertical", value, Indent::MAX_VERTICAL),
        };

        write!(f, "{ty} value {value} is greater than maximum {max}")
    }
}
