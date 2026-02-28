use fret_core::Color;

fn srgb_f32_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_f32_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_u8_to_linear(u: u8) -> f32 {
    srgb_f32_to_linear(u as f32 / 255.0)
}

fn linear_to_srgb_u8(c: f32) -> u8 {
    let srgb = linear_f32_to_srgb(c.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    (srgb * 255.0).round() as u8
}

/// Convert a `0xRRGGBB` sRGB hex value into a `fret_core::Color` in linear space.
pub fn linear_from_hex_rgb(hex: u32) -> Color {
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

/// Convert a linear `fret_core::Color` to a `0xRRGGBB` sRGB hex value.
pub fn hex_rgb_from_linear(color: Color) -> u32 {
    let r = linear_to_srgb_u8(color.r) as u32;
    let g = linear_to_srgb_u8(color.g) as u32;
    let b = linear_to_srgb_u8(color.b) as u32;
    (r << 16) | (g << 8) | b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_rgb_roundtrips_through_linear() {
        for &hex in &[
            0x00_00_00, 0xff_ff_ff, 0x16_a3_4a, // Tailwind green-600
            0x25_63_eb, // Tailwind blue-600
            0xca_8a_04, // Tailwind yellow-600
            0xdc_26_26, // Tailwind red-600
            0xea_58_0c, // Tailwind orange-600
        ] {
            let linear = linear_from_hex_rgb(hex);
            let back = hex_rgb_from_linear(linear);
            assert_eq!(back, hex, "expected {hex:06x} to roundtrip");
        }
    }
}
