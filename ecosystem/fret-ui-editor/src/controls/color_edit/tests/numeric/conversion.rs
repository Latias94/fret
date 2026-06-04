use super::super::*;

#[test]
fn hsv_conversion_matches_primary_colors() {
    let red = rgb_to_hsv(0xff_00_00);
    assert_hsv_close(red, 0.0, 1.0, 1.0);

    let green = rgb_to_hsv(0x00_ff_00);
    assert_hsv_close(green, 1.0 / 3.0, 1.0, 1.0);

    let blue = rgb_to_hsv(0x00_00_ff);
    assert_hsv_close(blue, 2.0 / 3.0, 1.0, 1.0);

    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 0.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0xff_00_00
    );
    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 1.0 / 3.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0x00_ff_00
    );
    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 2.0 / 3.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0x00_00_ff
    );
}

#[test]
fn hsv_conversion_handles_grayscale_without_unstable_hue() {
    assert_hsv_close(rgb_to_hsv(0x00_00_00), 0.0, 0.0, 0.0);
    assert_hsv_close(rgb_to_hsv(0x80_80_80), 0.0, 0.0, 128.0 / 255.0);
    assert_hsv_close(rgb_to_hsv(0xff_ff_ff), 0.0, 0.0, 1.0);

    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 0.42,
            saturation: 0.0,
            value: 128.0 / 255.0,
        }),
        0x80_80_80
    );
}

#[test]
fn hsv_conversion_roundtrips_color_presets() {
    for entry in default_color_edit_palette().iter() {
        let hsv = rgb_to_hsv(entry.rgb);
        assert_eq!(hsv_to_rgb(hsv), entry.rgb);
    }
}
