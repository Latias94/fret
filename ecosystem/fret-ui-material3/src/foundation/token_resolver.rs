use fret_core::Color;
use fret_ui::Theme;

pub(crate) fn alpha_mul(mut color: Color, multiplier: f32) -> Color {
    color.a = (color.a * multiplier).clamp(0.0, 1.0);
    color
}

pub(crate) fn blend_over(base: Color, overlay: Color, opacity: f32) -> Color {
    let a = (overlay.a * opacity).clamp(0.0, 1.0);
    if a <= 0.0 {
        return base;
    }

    let inv = 1.0 - a;
    Color {
        r: overlay.r * a + base.r * inv,
        g: overlay.g * a + base.g * inv,
        b: overlay.b * a + base.b * inv,
        a: a + base.a * inv,
    }
}

#[derive(Clone, Copy)]
pub struct MaterialTokenResolver<'a> {
    theme: &'a Theme,
}

impl<'a> MaterialTokenResolver<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    pub fn color_sys(&self, sys_key: &str) -> Color {
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.theme
            .color_by_key(sys_key)
            .unwrap_or_else(|| fallback_color_for_sys(sys_key))
    }

    pub fn color_comp_or_sys(&self, comp_key: &str, sys_key: &str) -> Color {
        debug_assert!(
            comp_key.starts_with("md.comp."),
            "expected md.comp.* key, got: {comp_key}"
        );
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.theme
            .color_by_key(comp_key)
            .or_else(|| self.theme.color_by_key(sys_key))
            .unwrap_or_else(|| fallback_color_for_sys(sys_key))
    }

    pub fn number_sys(&self, sys_key: &str, fallback: f32) -> f32 {
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.theme.number_by_key(sys_key).unwrap_or(fallback)
    }

    pub fn number_comp_or_sys(&self, comp_key: &str, sys_key: &str, fallback: f32) -> f32 {
        debug_assert!(
            comp_key.starts_with("md.comp."),
            "expected md.comp.* key, got: {comp_key}"
        );
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.theme
            .number_by_key(comp_key)
            .or_else(|| self.theme.number_by_key(sys_key))
            .unwrap_or(fallback)
    }
}

fn fallback_color_for_sys(sys_key: &str) -> Color {
    match sys_key {
        "md.sys.color.primary" => Color::from_srgb_hex_rgb(0x67_50_a4),
        "md.sys.color.on-primary" => Color::from_srgb_hex_rgb(0xff_ff_ff),
        "md.sys.color.surface" => Color::from_srgb_hex_rgb(0x1c_1c_1f),
        "md.sys.color.surface-container" => Color::from_srgb_hex_rgb(0x29_29_2b),
        "md.sys.color.surface-container-highest" => Color::from_srgb_hex_rgb(0x33_33_36),
        "md.sys.color.on-surface" => Color::from_srgb_hex_rgb(0xff_ff_ff),
        "md.sys.color.on-surface-variant" => Color::from_srgb_hex_rgb(0xbf_bf_c7),
        "md.sys.color.outline" => Color::from_srgb_hex_rgb(0x8c_8c_94),
        "md.sys.color.outline-variant" => Color::from_srgb_hex_rgb(0x59_59_61),
        _ => Color::from_srgb_hex_rgb(0xff_00_ff),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    #[test]
    fn alpha_mul_clamps_alpha() {
        assert_eq!(alpha_mul(color(0.1, 0.2, 0.3, 0.5), 3.0).a, 1.0);
        assert_eq!(alpha_mul(color(0.1, 0.2, 0.3, 0.5), -1.0).a, 0.0);
    }

    #[test]
    fn blend_over_uses_overlay_alpha_times_opacity() {
        let blended = blend_over(color(0.0, 0.0, 1.0, 1.0), color(1.0, 0.0, 0.0, 0.5), 0.5);
        assert!((blended.r - 0.25).abs() < 1e-6);
        assert!((blended.g - 0.0).abs() < 1e-6);
        assert!((blended.b - 0.75).abs() < 1e-6);
        assert!((blended.a - 1.0).abs() < 1e-6);
    }
}
