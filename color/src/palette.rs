use {
    crate::color::Color,
    palette::{color_difference::Ciede2000, convert::IntoColorUnclamped, Lab, LinSrgb, Srgb},
    std::collections::HashMap,
};

pub(crate) struct Palette {
    colors: Vec<Lab>,
    cache: HashMap<Color, Color>,
}

impl Palette {
    pub fn new(colors: &[Color]) -> Self {
        Self {
            colors: colors
                .iter()
                .map(|&Color([r, g, b])| Srgb::new(r, g, b).into_linear().into_color_unclamped())
                .collect(),
            cache: HashMap::with_capacity(128),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn closest(&mut self, target: Color) -> Color {
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
