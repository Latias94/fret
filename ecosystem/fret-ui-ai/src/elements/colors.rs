use fret_core::Color;

fn srgb_f32_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_u8_to_linear(u: u8) -> f32 {
    srgb_f32_to_linear(u as f32 / 255.0)
}

pub(crate) fn linear_from_hex_rgb(hex: u32) -> Color {
    let r = ((hex >> 16) & 0xff) as u8;
    let g = ((hex >> 8) & 0xff) as u8;
    let b = (hex & 0xff) as u8;
    Color {
        r: srgb_u8_to_linear(r),
        g: srgb_u8_to_linear(g),
        b: srgb_u8_to_linear(b),
        a: 1.0,
    }
}
