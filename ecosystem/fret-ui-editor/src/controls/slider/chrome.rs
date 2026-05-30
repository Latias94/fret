use crate::primitives::EditorTokenKeys;
use crate::primitives::colors::{editor_accent, editor_border, editor_subtle_bg};
use fret_core::Color;
use fret_ui::Theme;

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

#[cfg(test)]
mod tests {
    use super::resolve_slider_chrome;
    use crate::primitives::EditorTokenKeys;
    use fret_app::App;
    use fret_core::Color;
    use fret_ui::{Theme, ThemeConfig};

    #[test]
    fn slider_chrome_prefers_editor_owned_tokens_over_generic_palette() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors.insert(
                EditorTokenKeys::SLIDER_TRACK_BG.to_string(),
                "#171d26".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::SLIDER_FILL_BG.to_string(),
                "#355a86".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::SLIDER_THUMB_BG.to_string(),
                "#141b24".to_string(),
            );
            cfg.colors.insert(
                EditorTokenKeys::SLIDER_THUMB_BORDER.to_string(),
                "#3b4758".to_string(),
            );
            cfg.colors
                .insert("muted".to_string(), "#ff0000".to_string());
            cfg.colors
                .insert("primary".to_string(), "#00ff00".to_string());
            cfg.colors
                .insert("background".to_string(), "#0000ff".to_string());
            cfg.colors
                .insert("border".to_string(), "#ffffff".to_string());
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        let chrome = resolve_slider_chrome(theme);

        assert_eq!(chrome.track_bg, Color::from_srgb_hex_rgb(0x17_1d_26));
        assert_eq!(chrome.fill_bg, Color::from_srgb_hex_rgb(0x35_5a_86));
        assert_eq!(chrome.thumb_bg, Color::from_srgb_hex_rgb(0x14_1b_24));
        assert_eq!(chrome.thumb_border, Color::from_srgb_hex_rgb(0x3b_47_58));
    }
}
