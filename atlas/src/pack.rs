use {serde::Serialize, std::fmt};

type Size = (u32, u32);
type Point = (u32, u32);

#[derive(Clone, Copy, Serialize)]
#[serde(into = "[u32; 4]")]
pub(crate) struct Rect {
    size: Size,
    point: Point,
}

impl Rect {
    pub(crate) fn point(self) -> Point {
        self.point
    }
}

impl From<Rect> for [u32; 4] {
    fn from(
        Rect {
            size: (w, h),
            point: (x, y),
        }: Rect,
    ) -> Self {
        [x, y, w, h]
    }
}

pub(crate) struct Pack {
    pub rects: Vec<Rect>,
    pub side: u32,
}

pub(crate) fn pack(entries: &[Size], margin: Margin) -> Pack {
    let mut side = initial_side(entries);
    loop {
        match try_pack(entries, side, margin) {
            Some(rects) => return Pack { rects, side },
            None => side *= 2,
        }
    }
}

fn initial_side(entries: &[Size]) -> u32 {
    const MIN_INITIAL_SIDE: u32 = 64;

    let max_size = entries
        .iter()
        .map(|&(width, height)| u32::max(width, height))
        .max()
        .unwrap_or_default();

    let area: u32 = entries.iter().map(|(width, height)| width * height).sum();
    let area_sqrt = (area as f32).sqrt().ceil() as u32;
    let side = u32::max(max_size, area_sqrt).next_power_of_two();
    u32::max(side, MIN_INITIAL_SIDE)
}

fn try_pack(entries: &[Size], side: u32, margin: Margin) -> Option<Vec<Rect>> {
    let mut x = margin.horizontal;
    let mut y = margin.vertical;
    let mut max_height = 0;

    entries
        .iter()
        .map(|&(width, height)| {
            max_height = max_height.max(height);

            if x + width + margin.horizontal > side {
                x = margin.horizontal;
                y += max_height + margin.vertical;
                max_height = 0;
            }

            if y + height + margin.vertical > side {
                return None;
            }

            let point = (x, y);
            x += width + margin.horizontal;

            Some(Rect {
                size: (width, height),
                point,
            })
        })
        .collect()
}

#[derive(Clone, Copy)]
pub struct Margin {
    horizontal: u32,
    vertical: u32,
}

impl Margin {
    const MAX_HORIZONTAL: u32 = 4;
    const MAX_VERTICAL: u32 = 4;

    /// Creates a new margin.
    ///
    /// # Errors
    /// This function returns an error if the margin is [too large](TooLarge).
    pub fn new(horizontal: u32, vertical: u32) -> Result<Self, TooLarge> {
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

/// The margin is too large.
pub enum TooLarge {
    Horizontal(u32),
    Vertical(u32),
}

impl fmt::Display for TooLarge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ty, margin, max) = match self {
            Self::Horizontal(margin) => ("horizontal", margin, Margin::MAX_HORIZONTAL),
            Self::Vertical(margin) => ("vertical", margin, Margin::MAX_VERTICAL),
        };

        write!(f, "{ty} margin {margin} is greater than maximum {max}",)
    }
}
