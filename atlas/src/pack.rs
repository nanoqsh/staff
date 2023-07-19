use serde::Serialize;

type Size = (u32, u32);
type Point = (u32, u32);

#[derive(Clone, Copy, Serialize)]
#[serde(into = "[u32; 4]")]
pub struct Rect {
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

pub(crate) fn pack(entries: &[Size]) -> Pack {
    let mut side = initial_side(entries);
    loop {
        match try_pack(entries, side) {
            Some(rects) => return Pack { rects, side },
            None => side = side.pow(2),
        }
    }
}

fn initial_side(entries: &[Size]) -> u32 {
    const MIN_INITIAL_SIDE: u32 = 128;

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

fn try_pack(entries: &[Size], side: u32) -> Option<Vec<Rect>> {
    let mut x = 0;
    let mut y = 0;
    let mut max_height = 0;

    entries
        .iter()
        .map(|&(width, height)| {
            max_height = max_height.max(height);

            if x + width > side {
                x = 0;
                y += max_height;
                max_height = 0;
            }

            if y + height > side {
                return None;
            }

            let point = (x, y);
            x += width;

            Some(Rect {
                size: (width, height),
                point,
            })
        })
        .collect()
}
