use {
    crate::color::Color,
    palette::{color_difference::Ciede2000, convert::IntoColorUnclamped, Lab, LinSrgb, Srgb},
    std::{collections::HashMap, iter},
};

pub(crate) struct Exact {
    transfer: HashMap<Color, Color>,
}

impl Exact {
    pub fn new(from: &[Color], to: &[Color]) -> Self {
        Self {
            transfer: iter::zip(from, to).map(|(&f, &t)| (f, t)).collect(),
        }
    }

    pub fn transfer(&mut self, target: Color) -> Option<Color> {
        self.transfer.get(&target).copied()
    }
}

pub(crate) struct Closest {
    colors: Vec<Lab>,
    cache: HashMap<Color, Color>,
}

impl Closest {
    pub fn new(colors: &[Color]) -> Self {
        Self {
            colors: colors
                .iter()
                .map(|&Color([r, g, b])| Srgb::new(r, g, b).into_linear().into_color_unclamped())
                .collect(),
            cache: HashMap::with_capacity(128),
        }
    }

    pub fn transfer(&mut self, target: Color) -> Color {
        *self.cache.entry(target).or_insert_with(|| {
            let target = {
                let Color([r, g, b]) = target;
                Srgb::new(r, g, b).into_linear().into_color_unclamped()
            };

            let diffs = self.colors.iter().map(|col| col.difference(target));
            let (min_idx, _) =
                (0..)
                    .zip(diffs)
                    .fold((0, f32::INFINITY), |min @ (_, min_diff), (idx, diff)| {
                        if diff < min_diff {
                            (idx, diff)
                        } else {
                            min
                        }
                    });

            let linrgb: LinSrgb = self.colors[min_idx].into_color_unclamped();
            let rgb = Srgb::from_linear(linrgb);
            Color(rgb.into())
        })
    }
}
