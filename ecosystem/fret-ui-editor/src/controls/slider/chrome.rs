use crate::primitives::EditorTokenKeys;
use crate::primitives::colors::{editor_accent, editor_border, editor_subtle_bg};
use fret_core::Color;
use fret_ui::Theme;

#[cfg(test)]
mod tests;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub(super) fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

pub(super) fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ResolvedSliderChrome {
    pub(super) track_bg: Color,
    pub(super) fill_bg: Color,
    pub(super) thumb_bg: Color,
    pub(super) thumb_border: Color,
}

pub(super) fn resolve_slider_chrome(theme: &Theme) -> ResolvedSliderChrome {
    let track_bg = theme
        .color_by_key(EditorTokenKeys::SLIDER_TRACK_BG)
        .or_else(|| theme.color_by_key("component.slider.track_bg"))
        .unwrap_or_else(|| editor_subtle_bg(theme));
    let fill_bg = theme
        .color_by_key(EditorTokenKeys::SLIDER_FILL_BG)
        .or_else(|| theme.color_by_key("component.slider.fill_bg"))
        .unwrap_or_else(|| editor_accent(theme));
    let thumb_bg = theme
        .color_by_key(EditorTokenKeys::SLIDER_THUMB_BG)
        .or_else(|| theme.color_by_key("component.slider.thumb_bg"))
        .unwrap_or_else(|| editor_subtle_bg(theme));
    let thumb_border = theme
        .color_by_key(EditorTokenKeys::SLIDER_THUMB_BORDER)
        .or_else(|| theme.color_by_key("component.slider.thumb_border"))
        .unwrap_or_else(|| editor_border(theme));

    ResolvedSliderChrome {
        track_bg,
        fill_bg,
        thumb_bg,
        thumb_border,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ResolvedSliderPaint {
    pub(super) track_bg: Color,
    pub(super) fill_bg: Color,
    pub(super) thumb_bg: Color,
    pub(super) thumb_border: Color,
}

pub(super) fn resolve_slider_paint(
    theme: &Theme,
    interactive_enabled: bool,
    enabled: bool,
    hovered: bool,
    pressed: bool,
) -> ResolvedSliderPaint {
    let accent = editor_accent(theme);
    let disabled_alpha = if interactive_enabled { 1.0 } else { 0.55 };
    let chrome = resolve_slider_chrome(theme);

    let mut track_bg = chrome.track_bg;
    let mut fill_bg = chrome.fill_bg;
    let thumb_bg = chrome.thumb_bg;
    let thumb_border = chrome.thumb_border;

    if hovered && enabled {
        track_bg = mix(track_bg, accent, 0.06);
        fill_bg = mix(fill_bg, accent, 0.04);
    }
    if pressed && enabled {
        track_bg = mix(track_bg, accent, 0.10);
        fill_bg = mix(fill_bg, accent, 0.08);
    }

    ResolvedSliderPaint {
        track_bg: alpha_mul(track_bg, disabled_alpha),
        fill_bg: alpha_mul(fill_bg, disabled_alpha),
        thumb_bg: alpha_mul(thumb_bg, disabled_alpha),
        thumb_border: alpha_mul(thumb_border, disabled_alpha),
    }
}
