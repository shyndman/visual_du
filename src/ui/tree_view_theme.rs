use bevy::prelude::{Color, Component};
use colorous::RAINBOW;
use valuable_derive::Valuable;

const HSL_MIN_LIGHTNESS: f32 = 0.62;
const HSL_MAX_LIGHTNESS: f32 = 0.9;

pub fn hsl_get_color(t: f32, depth: u16) -> Color {
    let lightness_fraction = ((depth - 1) as f32).clamp(0.0, 5.0) / 5.0;
    let lightness =
        HSL_MIN_LIGHTNESS + lightness_fraction * (HSL_MAX_LIGHTNESS - HSL_MIN_LIGHTNESS);
    Color::hsl((200.0 + (t * 360.0)) % 360.0, 1.0, lightness)
}

pub fn less_angry_rainbow_get_color(t: f32, depth: u16) -> Color {
    let colorous_color = RAINBOW.eval_continuous((t as f64) % 1.0);
    let bevy_color = Color::rgb_u8(colorous_color.r, colorous_color.g, colorous_color.b);

    if let Color::Hsla {
        hue,
        saturation,
        lightness,
        alpha: _,
    } = bevy_color.as_hsla()
    {
        Color::Hsla {
            hue,
            saturation,
            lightness: lightness.powf(0.8_f32.powi((depth as i32) - 1)),
            alpha: 1.0,
        }
    } else {
        panic!("This can't happen");
    }
}

#[derive(Component, Clone, Copy, Debug, Valuable)]
pub struct DescendentColorRange {
    /// [0..1]
    start: f32,
    /// [0..1]
    end: f32,
}

impl DescendentColorRange {
    fn len(&self) -> f32 {
        self.end - self.start
    }

    pub fn sub_range(&self, fraction_start: f32, fraction_len: f32) -> DescendentColorRange {
        let start = self.start + fraction_start * self.len();
        DescendentColorRange {
            start,
            end: start + fraction_len * self.len(),
        }
    }

    pub fn get_color(&self, t: f32, depth: u16) -> Color {
        less_angry_rainbow_get_color(self.start + t * self.len(), depth)
    }
}

impl Default for DescendentColorRange {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 1.0,
        }
    }
}
