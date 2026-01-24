use fret_core::{Color, Corners, Px};
use fret_ui::Theme;
use fret_ui::element::{ShadowLayerStyle, ShadowStyle};

use crate::foundation::token_resolver::MaterialTokenResolver;

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
