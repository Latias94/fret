use fret_core::Color;

/// Convert a `0xRRGGBB` sRGB hex value into a `fret_core::Color` in linear space.
pub fn linear_from_hex_rgb(hex: u32) -> Color {
    Color::from_srgb_hex_rgb(hex)
}

/// Convert a linear `fret_core::Color` to a `0xRRGGBB` sRGB hex value.
pub fn hex_rgb_from_linear(color: Color) -> u32 {
    color.to_srgb_hex_rgb()
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
