use fret_core::{Color, Corners, Px};
use fret_ui::Theme;
use fret_ui::element::{ShadowLayerStyle, ShadowStyle};
use fret_ui::theme::CubicBezier;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion::{cubic_bezier_ease, ms_to_frames};

/// Compute the Material 3 surface tint overlay alpha for a given elevation.
///
/// This matches Compose Material 3:
/// `alpha = ((4.5 * ln(elevation + 1)) + 2) / 100` (and alpha is 0 when elevation is 0).
pub fn surface_tint_alpha_for_elevation(elevation: Px) -> f32 {
    let v = elevation.0;
    if !v.is_finite() || v <= 0.0 {
        return 0.0;
    }

    // `ln(elevation + 1)` is well-defined for v > 0.
    (((4.5f32 * (v + 1.0).ln()) + 2.0) / 100.0).clamp(0.0, 1.0)
}

/// Apply a Material 3 tonal elevation overlay (surface tint) on top of `background`.
pub fn apply_surface_tint(background: Color, surface_tint: Color, elevation: Px) -> Color {
    let alpha = surface_tint_alpha_for_elevation(elevation);
    if alpha <= 0.0 {
        return background;
    }

    let mut overlay = surface_tint;
    overlay.a = (overlay.a * alpha).clamp(0.0, 1.0);
    composite_over(overlay, background)
}

/// Apply a tonal elevation overlay only when `background` is `md.sys.color.surface`.
///
/// This matches Compose Material 3 `Surface`: tonal elevation is only applied when the surface's
/// base color equals `ColorScheme.surface`.
pub fn apply_surface_tint_if_surface(theme: &Theme, background: Color, elevation: Px) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let surface = tokens.color_sys("md.sys.color.surface");
    if !colors_close(background, surface) {
        return background;
    }

    let surface_tint = tokens.color_sys("md.sys.color.surface-tint");
    apply_surface_tint(background, surface_tint, elevation)
}

/// Convert a Material elevation token (dp-like px values or web "level" numbers) into a Fret shadow.
///
/// Source of truth (web box-shadow mapping):
/// - `repo-ref/material-web/elevation/internal/_elevation.scss`
///
/// Note: Material Web's "level" is a discrete 0..=5 index, while the generated v30 token exports
/// in `tokens/versions/v30_0/sass/_md-sys-elevation.scss` currently use dp values
/// (0, 1, 3, 6, 8, 12). This helper accepts either and maps them to a level.
pub fn shadow_for_elevation_with_color(
    theme: &Theme,
    elevation: Px,
    shadow_color: Option<Color>,
    corner_radii: Corners,
) -> Option<ShadowStyle> {
    let level = elevation_to_level(elevation);
    if level == 0 {
        return None;
    }

    let tokens = MaterialTokenResolver::new(theme);
    let base = shadow_color.unwrap_or_else(|| tokens.color_sys("md.sys.color.shadow"));

    let (key, ambient) = material_web_shadow_layers(level, base);
    Some(ShadowStyle {
        primary: key,
        secondary: Some(ambient),
        corner_radii,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialElevationInteraction {
    Pressed,
    Hovered,
    Focused,
}

#[derive(Debug, Clone)]
pub struct AnimatedElevationRuntime {
    initialized: bool,
    current: f32,
    from: f32,
    to: f32,
    start_frame: u64,
    duration_frames: u64,
    easing: CubicBezier,
    active: bool,
    last_interaction: Option<MaterialElevationInteraction>,
}

impl Default for AnimatedElevationRuntime {
    fn default() -> Self {
        Self {
            initialized: false,
            current: 0.0,
            from: 0.0,
            to: 0.0,
            start_frame: 0,
            duration_frames: 0,
            easing: CUBIC_BEZIER_LINEAR,
            active: false,
            last_interaction: None,
        }
    }
}

pub fn animate_material_elevation(
    runtime: &mut AnimatedElevationRuntime,
    now_frame: u64,
    enabled: bool,
    interaction: Option<MaterialElevationInteraction>,
    target: Px,
) -> (Px, bool) {
    let target = target.0.max(0.0);
    if !runtime.initialized {
        runtime.reset(now_frame, target, interaction);
        return (Px(runtime.current), false);
    }

    runtime.advance(now_frame);

    if !enabled {
        // Compose snaps elevation when entering disabled state.
        runtime.reset(now_frame, target, None);
        return (Px(runtime.current), false);
    }

    if (target - runtime.to).abs() > 1e-6 {
        let (duration_ms, easing) =
            material_elevation_transition(runtime.last_interaction, interaction);
        runtime.set_target(now_frame, target, duration_ms, easing);
    }

    runtime.last_interaction = interaction;
    (Px(runtime.current.max(0.0)), runtime.active)
}

impl AnimatedElevationRuntime {
    fn reset(
        &mut self,
        now_frame: u64,
        value: f32,
        interaction: Option<MaterialElevationInteraction>,
    ) {
        self.initialized = true;
        self.current = value;
        self.from = value;
        self.to = value;
        self.start_frame = now_frame;
        self.duration_frames = 0;
        self.easing = CUBIC_BEZIER_LINEAR;
        self.active = false;
        self.last_interaction = interaction;
    }

    fn set_target(&mut self, now_frame: u64, target: f32, duration_ms: u32, easing: CubicBezier) {
        if duration_ms == 0 {
            self.reset(now_frame, target, self.last_interaction);
            return;
        }

        self.from = self.current;
        self.to = target;
        self.start_frame = now_frame;
        self.duration_frames = ms_to_frames(duration_ms).max(1);
        self.easing = easing;
        self.active = true;
    }

    fn advance(&mut self, now_frame: u64) {
        if !self.active {
            return;
        }

        let elapsed = now_frame.saturating_sub(self.start_frame);
        if elapsed >= self.duration_frames {
            self.current = self.to;
            self.active = false;
            return;
        }

        let t = elapsed as f32 / self.duration_frames as f32;
        let eased = cubic_bezier_ease(self.easing, t);
        self.current = self.from + (self.to - self.from) * eased;
    }
}

fn material_elevation_transition(
    from: Option<MaterialElevationInteraction>,
    to: Option<MaterialElevationInteraction>,
) -> (u32, CubicBezier) {
    if to.is_some() {
        return (120, fast_out_slow_in_easing());
    }
    match from {
        Some(MaterialElevationInteraction::Hovered) => (120, elevation_outgoing_easing()),
        Some(_) => (150, elevation_outgoing_easing()),
        None => (0, CUBIC_BEZIER_LINEAR),
    }
}

const CUBIC_BEZIER_LINEAR: CubicBezier = CubicBezier {
    x1: 0.0,
    y1: 0.0,
    x2: 1.0,
    y2: 1.0,
};

fn fast_out_slow_in_easing() -> CubicBezier {
    CubicBezier {
        x1: 0.4,
        y1: 0.0,
        x2: 0.2,
        y2: 1.0,
    }
}

fn elevation_outgoing_easing() -> CubicBezier {
    CubicBezier {
        x1: 0.4,
        y1: 0.0,
        x2: 0.6,
        y2: 1.0,
    }
}

fn elevation_to_level(elevation: Px) -> u8 {
    let v = elevation.0;
    if !v.is_finite() || v <= 0.0 {
        return 0;
    }

    // Prefer the dp-like values produced by the versioned sass exports.
    // This matches `tokens/versions/v30_0/sass/_md-sys-elevation.scss`.
    const EPS: f32 = 1e-3;
    let dp_map = [
        (0.0, 0u8),
        (1.0, 1u8),
        (3.0, 2u8),
        (6.0, 3u8),
        (8.0, 4u8),
        (12.0, 5u8),
    ];
    for (dp, level) in dp_map {
        if (v - dp).abs() <= EPS {
            return level;
        }
    }

    // Fallback for the "web level number" representation (0..=5).
    // Clamp because Material Web's elevation component saturates at level5.
    v.round().clamp(0.0, 5.0) as u8
}

fn material_web_shadow_layers(
    level: u8,
    shadow_color: Color,
) -> (ShadowLayerStyle, ShadowLayerStyle) {
    // Material Web applies opacity as a separate property on each shadow pseudo element.
    // We approximate by baking the opacity into the shadow color alpha.
    let (key_y, key_blur) = match level {
        1 => (1.0, 2.0),
        2 => (1.0, 2.0),
        3 => (1.0, 3.0),
        4 => (2.0, 3.0),
        _ => (4.0, 4.0), // level5+ (saturated)
    };
    let (ambient_y, ambient_blur, ambient_spread) = match level {
        1 => (1.0, 3.0, 1.0),
        2 => (2.0, 6.0, 2.0),
        3 => (4.0, 8.0, 3.0),
        4 => (6.0, 10.0, 4.0),
        _ => (8.0, 12.0, 6.0), // level5+ (saturated)
    };

    // Key shadow (opacity: 0.3)
    let mut key_color = shadow_color;
    key_color.a = (key_color.a * 0.3).clamp(0.0, 1.0);
    let key = ShadowLayerStyle {
        color: key_color,
        offset_x: Px(0.0),
        offset_y: Px(key_y),
        blur: Px(key_blur),
        spread: Px(0.0),
    };

    // Ambient shadow (opacity: 0.15)
    let mut ambient_color = shadow_color;
    ambient_color.a = (ambient_color.a * 0.15).clamp(0.0, 1.0);
    let ambient = ShadowLayerStyle {
        color: ambient_color,
        offset_x: Px(0.0),
        offset_y: Px(ambient_y),
        blur: Px(ambient_blur),
        spread: Px(ambient_spread),
    };

    (key, ambient)
}

fn composite_over(overlay: Color, base: Color) -> Color {
    // Standard "source over" alpha compositing for non-premultiplied colors.
    let oa = overlay.a.clamp(0.0, 1.0);
    let ba = base.a.clamp(0.0, 1.0);

    let out_a = oa + ba * (1.0 - oa);
    if out_a <= 0.0 {
        return Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
    }

    let o_pr = overlay.r * oa;
    let o_pg = overlay.g * oa;
    let o_pb = overlay.b * oa;

    let b_pr = base.r * ba;
    let b_pg = base.g * ba;
    let b_pb = base.b * ba;

    let out_pr = o_pr + b_pr * (1.0 - oa);
    let out_pg = o_pg + b_pg * (1.0 - oa);
    let out_pb = o_pb + b_pb * (1.0 - oa);

    Color {
        r: (out_pr / out_a).clamp(0.0, 1.0),
        g: (out_pg / out_a).clamp(0.0, 1.0),
        b: (out_pb / out_a).clamp(0.0, 1.0),
        a: out_a,
    }
}

fn colors_close(a: Color, b: Color) -> bool {
    let eps = 1e-4;
    (a.r - b.r).abs() <= eps
        && (a.g - b.g).abs() <= eps
        && (a.b - b.b).abs() <= eps
        && (a.a - b.a).abs() <= eps
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn surface_tint_only_applies_to_surface_color() {
        let mut patch = ThemeConfig::default();
        patch
            .colors
            .insert("md.sys.color.surface".to_string(), "#101010".to_string());
        patch.colors.insert(
            "md.sys.color.surface-tint".to_string(),
            "#ff0000".to_string(),
        );
        let (_app, theme) = theme_with_patch(patch);

        let surface = Color::from_srgb_hex_rgb(0x10_10_10);
        let non_surface = Color::from_srgb_hex_rgb(0x20_20_20);
        let tinted = apply_surface_tint_if_surface(&theme, surface, Px(1.0));
        let untouched = apply_surface_tint_if_surface(&theme, non_surface, Px(1.0));

        assert_ne!(tinted, surface);
        assert_eq!(untouched, non_surface);
    }
}
