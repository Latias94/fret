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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MaterialStateLayerInteraction {
    Hovered,
    Focused,
    Pressed,
}

impl MaterialStateLayerInteraction {
    fn sys_opacity_key(self) -> &'static str {
        match self {
            Self::Hovered => "md.sys.state.hover.state-layer-opacity",
            Self::Focused => "md.sys.state.focus.state-layer-opacity",
            Self::Pressed => "md.sys.state.pressed.state-layer-opacity",
        }
    }

    fn fallback_opacity(self) -> f32 {
        match self {
            Self::Hovered => 0.08,
            Self::Focused | Self::Pressed => 0.1,
        }
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

    pub fn color_comp_or_sys_chain(&self, comp_key: &str, sys_keys: &[&str]) -> Color {
        debug_assert!(
            comp_key.starts_with("md.comp."),
            "expected md.comp.* key, got: {comp_key}"
        );
        debug_assert!(!sys_keys.is_empty(), "expected at least one md.sys.* key");
        debug_assert!(
            sys_keys.iter().all(|key| key.starts_with("md.sys.")),
            "expected md.sys.* fallback keys, got: {sys_keys:?}"
        );

        let fallback_sys_key = sys_keys.last().copied().unwrap_or("md.sys.color.surface");

        self.theme
            .color_by_key(comp_key)
            .or_else(|| sys_keys.iter().find_map(|key| self.theme.color_by_key(key)))
            .unwrap_or_else(|| fallback_color_for_sys(fallback_sys_key))
    }

    pub fn color_comp_or_fallback(&self, comp_key: &str, fallback: Color) -> Color {
        debug_assert!(
            comp_key.starts_with("md.comp."),
            "expected md.comp.* key, got: {comp_key}"
        );
        self.theme.color_by_key(comp_key).unwrap_or(fallback)
    }

    pub fn color_comp_chain(&self, comp_keys: &[&str]) -> Option<Color> {
        debug_assert_comp_keys(comp_keys);
        comp_keys
            .iter()
            .find_map(|key| self.theme.color_by_key(key))
    }

    pub fn color_comp_chain_or_sys(&self, comp_keys: &[&str], sys_key: &str) -> Color {
        debug_assert_comp_keys(comp_keys);
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.color_comp_chain(comp_keys)
            .unwrap_or_else(|| self.color_sys(sys_key))
    }

    pub fn color_comp_chain_or_sys_chain(&self, comp_keys: &[&str], sys_keys: &[&str]) -> Color {
        debug_assert_comp_keys(comp_keys);
        debug_assert!(!sys_keys.is_empty(), "expected at least one md.sys.* key");
        debug_assert!(
            sys_keys.iter().all(|key| key.starts_with("md.sys.")),
            "expected md.sys.* fallback keys, got: {sys_keys:?}"
        );

        let fallback_sys_key = sys_keys.last().copied().unwrap_or("md.sys.color.surface");

        self.color_comp_chain(comp_keys)
            .or_else(|| sys_keys.iter().find_map(|key| self.theme.color_by_key(key)))
            .unwrap_or_else(|| fallback_color_for_sys(fallback_sys_key))
    }

    pub fn color_comp_chain_or_sys_or(
        &self,
        comp_keys: &[&str],
        sys_key: &str,
        fallback: Color,
    ) -> Color {
        debug_assert_comp_keys(comp_keys);
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.color_comp_chain(comp_keys)
            .or_else(|| self.theme.color_by_key(sys_key))
            .unwrap_or(fallback)
    }

    pub fn color_comp_chain_or_fallback(&self, comp_keys: &[&str], fallback: Color) -> Color {
        debug_assert_comp_keys(comp_keys);
        self.color_comp_chain(comp_keys).unwrap_or(fallback)
    }

    pub fn color_comp_or_sys_or(&self, comp_key: &str, sys_key: &str, fallback: Color) -> Color {
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
            .unwrap_or(fallback)
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

    pub fn number_optional(&self, key: Option<&str>, fallback: f32) -> f32 {
        debug_assert!(
            key.is_none_or(|key| key.starts_with("md.")),
            "expected md.* number token key, got: {key:?}"
        );
        key.and_then(|key| self.theme.number_by_key(key))
            .unwrap_or(fallback)
    }

    pub fn number_chain(&self, keys: &[&str], fallback: f32) -> f32 {
        debug_assert!(!keys.is_empty(), "expected at least one md.* key");
        debug_assert!(
            keys.iter().all(|key| key.starts_with("md.")),
            "expected md.* number token keys, got: {keys:?}"
        );
        keys.iter()
            .find_map(|key| self.theme.number_by_key(key))
            .unwrap_or(fallback)
    }

    pub fn number_comp_chain_or_sys(
        &self,
        comp_keys: &[&str],
        sys_key: &str,
        fallback: f32,
    ) -> f32 {
        debug_assert_comp_keys(comp_keys);
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        comp_keys
            .iter()
            .find_map(|key| self.theme.number_by_key(key))
            .or_else(|| self.theme.number_by_key(sys_key))
            .unwrap_or(fallback)
    }

    pub fn color_comp_or_sys_with_opacity(
        &self,
        comp_key: &str,
        sys_key: &str,
        opacity_key: Option<&str>,
        fallback_opacity: f32,
    ) -> (Color, f32) {
        (
            self.color_comp_or_sys(comp_key, sys_key),
            self.number_optional(opacity_key, fallback_opacity),
        )
    }

    pub fn state_layer_opacity(
        &self,
        comp_key: &str,
        interaction: MaterialStateLayerInteraction,
    ) -> f32 {
        self.number_comp_or_sys(
            comp_key,
            interaction.sys_opacity_key(),
            interaction.fallback_opacity(),
        )
    }

    pub fn disabled_state_layer_opacity(&self) -> f32 {
        self.number_sys("md.sys.state.disabled.state-layer-opacity", 0.38)
    }
}

fn debug_assert_comp_keys(comp_keys: &[&str]) {
    debug_assert!(!comp_keys.is_empty(), "expected at least one md.comp.* key");
    debug_assert!(
        comp_keys.iter().all(|key| key.starts_with("md.comp.")),
        "expected md.comp.* keys, got: {comp_keys:?}"
    );
}

fn fallback_color_for_sys(sys_key: &str) -> Color {
    match sys_key {
        "md.sys.color.primary" => Color::from_srgb_hex_rgb(0x67_50_a4),
        "md.sys.color.on-primary" => Color::from_srgb_hex_rgb(0xff_ff_ff),
        "md.sys.color.secondary-container" => Color::from_srgb_hex_rgb(0x4a_44_5f),
        "md.sys.color.on-secondary-container" => Color::from_srgb_hex_rgb(0xe8_de_ff),
        "md.sys.color.surface" => Color::from_srgb_hex_rgb(0x1c_1c_1f),
        "md.sys.color.surface-container" => Color::from_srgb_hex_rgb(0x29_29_2b),
        "md.sys.color.surface-container-low" => Color::from_srgb_hex_rgb(0x21_21_24),
        "md.sys.color.surface-container-high" => Color::from_srgb_hex_rgb(0x2e_2e_31),
        "md.sys.color.surface-container-highest" => Color::from_srgb_hex_rgb(0x33_33_36),
        "md.sys.color.on-surface" => Color::from_srgb_hex_rgb(0xff_ff_ff),
        "md.sys.color.on-surface-variant" => Color::from_srgb_hex_rgb(0xbf_bf_c7),
        "md.sys.color.outline" => Color::from_srgb_hex_rgb(0x8c_8c_94),
        "md.sys.color.outline-variant" => Color::from_srgb_hex_rgb(0x59_59_61),
        "md.sys.color.error" => Color::from_srgb_hex_rgb(0xff_b4_ab),
        "md.sys.color.shadow" => Color::from_srgb_hex_rgb(0x00_00_00),
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

    #[test]
    fn state_layer_interaction_fallbacks_match_material_defaults() {
        assert_eq!(
            MaterialStateLayerInteraction::Hovered.fallback_opacity(),
            0.08
        );
        assert_eq!(
            MaterialStateLayerInteraction::Focused.fallback_opacity(),
            0.1
        );
        assert_eq!(
            MaterialStateLayerInteraction::Pressed.fallback_opacity(),
            0.1
        );
    }
}
