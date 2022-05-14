use bevy::prelude::Color;
use colorous::RAINBOW;

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
