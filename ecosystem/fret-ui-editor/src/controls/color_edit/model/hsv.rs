use std::sync::Arc;

use fret_core::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::controls::color_edit) struct HsvColor {
    pub(in crate::controls::color_edit) hue: f32,
    pub(in crate::controls::color_edit) saturation: f32,
    pub(in crate::controls::color_edit) value: f32,
}

pub(in crate::controls::color_edit) fn hsv_from_color(color: Color) -> HsvColor {
    rgb_to_hsv(fret_ui_kit::colors::hex_rgb_from_linear(color))
}

pub(in crate::controls::color_edit) fn rgb_to_hsv(rgb: u32) -> HsvColor {
    let r = ((rgb >> 16) & 0xff) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xff) as f32 / 255.0;
    let b = (rgb & 0xff) as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;
    let hue = if chroma <= f32::EPSILON {
        0.0
    } else if (max - r).abs() <= f32::EPSILON {
        ((g - b) / chroma).rem_euclid(6.0) / 6.0
    } else if (max - g).abs() <= f32::EPSILON {
        (((b - r) / chroma) + 2.0) / 6.0
    } else {
        (((r - g) / chroma) + 4.0) / 6.0
    };
    let saturation = if max <= f32::EPSILON {
        0.0
    } else {
        chroma / max
    };

    HsvColor {
        hue: sanitize_hue(hue),
        saturation: saturation.clamp(0.0, 1.0),
        value: max.clamp(0.0, 1.0),
    }
}

pub(in crate::controls::color_edit) fn hsv_to_rgb(hsv: HsvColor) -> u32 {
    let hue = sanitize_hue(hsv.hue);
    let saturation = sanitize_unit(hsv.saturation);
    let value = sanitize_unit(hsv.value);

    if saturation <= f32::EPSILON {
        let v = unit_to_u8(value);
        return ((v as u32) << 16) | ((v as u32) << 8) | v as u32;
    }

    let scaled_hue = hue * 6.0;
    let sector = scaled_hue.floor() as i32;
    let fraction = scaled_hue - sector as f32;
    let p = value * (1.0 - saturation);
    let q = value * (1.0 - saturation * fraction);
    let t = value * (1.0 - saturation * (1.0 - fraction));
    let (r, g, b) = match sector {
        0 => (value, t, p),
        1 => (q, value, p),
        2 => (p, value, t),
        3 => (p, q, value),
        4 => (t, p, value),
        _ => (value, p, q),
    };

    ((unit_to_u8(r) as u32) << 16) | ((unit_to_u8(g) as u32) << 8) | unit_to_u8(b) as u32
}

pub(in crate::controls::color_edit) fn hsv_to_color_preserving_alpha(
    hsv: HsvColor,
    alpha: f32,
) -> Color {
    color_from_rgb_preserving_alpha(hsv_to_rgb(hsv), alpha)
}

pub(in crate::controls::color_edit) fn color_from_rgb_preserving_alpha(
    rgb: u32,
    alpha: f32,
) -> Color {
    let mut out = Color::from_srgb_hex_rgb(rgb);
    out.a = alpha.clamp(0.0, 1.0);
    out
}

pub(in crate::controls::color_edit) fn hsv_with_sv_from_local_position(
    current: HsvColor,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HsvColor {
    HsvColor {
        hue: current.hue,
        saturation: unit_from_local_x(x, width),
        value: 1.0 - unit_from_local_y(y, height),
    }
}

pub(in crate::controls::color_edit) fn hue_from_local_y(y: f32, height: f32) -> f32 {
    unit_from_local_y(y, height)
}

pub(in crate::controls::color_edit) fn unit_from_local_x(x: f32, width: f32) -> f32 {
    if !width.is_finite() || width <= f32::EPSILON {
        return 0.0;
    }
    sanitize_unit(x / width)
}

pub(in crate::controls::color_edit) fn unit_from_local_y(y: f32, height: f32) -> f32 {
    if !height.is_finite() || height <= f32::EPSILON {
        return 0.0;
    }
    sanitize_unit(y / height)
}

pub(in crate::controls::color_edit) fn unit_from_step(index: usize, steps: usize) -> f32 {
    if steps <= 1 {
        return 0.0;
    }
    (index as f32 / (steps - 1) as f32).clamp(0.0, 1.0)
}

pub(in crate::controls::color_edit) fn sanitize_hue(hue: f32) -> f32 {
    if !hue.is_finite() {
        return 0.0;
    }
    hue.rem_euclid(1.0)
}

pub(in crate::controls::color_edit) fn sanitize_unit(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}

fn unit_to_u8(value: f32) -> u8 {
    (sanitize_unit(value) * 255.0).round() as u8
}

pub(in crate::controls::color_edit) fn sv_picker_a11y_text(hsv: HsvColor) -> Arc<str> {
    Arc::from(format!(
        "S {}%, V {}%",
        (sanitize_unit(hsv.saturation) * 100.0).round() as u8,
        (sanitize_unit(hsv.value) * 100.0).round() as u8
    ))
}

pub(in crate::controls::color_edit) fn hue_percent_text(hue: f32) -> Arc<str> {
    Arc::from(format!("{}%", (sanitize_hue(hue) * 100.0).round() as u8))
}
