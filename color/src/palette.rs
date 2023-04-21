use {
    crate::color::Color,
    palette::{convert::IntoColorUnclamped, ColorDifference, Lab, LinSrgb, Srgb},
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

            let ds = self
                .colors
                .iter()
                .map(|col| col.get_color_difference(target));

            let mut min = f32::INFINITY;
            let mut min_idx = 0;
            for (idx, d) in (0..).zip(ds) {
                if d < min {
                    min = d;
                    min_idx = idx;
                }
            }

            let linrgb: LinSrgb = self.colors[min_idx].into_color_unclamped();
            let rgb = Srgb::from_linear(linrgb);
            Color(rgb.into())
        })
    }
}
