use super::{resolve_slider_chrome, resolve_slider_paint};
use crate::primitives::EditorTokenKeys;
use fret_app::App;
use fret_core::Color;
use fret_ui::{Theme, ThemeConfig};

fn install_slider_colors(app: &mut App) {
    Theme::with_global_mut(app, |theme| {
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
}

fn assert_color_close(actual: Color, expected: Color) {
    const EPSILON: f32 = 0.000_001;
    assert!((actual.r - expected.r).abs() <= EPSILON);
    assert!((actual.g - expected.g).abs() <= EPSILON);
    assert!((actual.b - expected.b).abs() <= EPSILON);
    assert!((actual.a - expected.a).abs() <= EPSILON);
}

#[test]
fn slider_chrome_prefers_editor_owned_tokens_over_generic_palette() {
    let mut app = App::new();
    install_slider_colors(&mut app);

    let theme = Theme::global(&app);
    let chrome = resolve_slider_chrome(theme);

    assert_eq!(chrome.track_bg, Color::from_srgb_hex_rgb(0x17_1d_26));
    assert_eq!(chrome.fill_bg, Color::from_srgb_hex_rgb(0x35_5a_86));
    assert_eq!(chrome.thumb_bg, Color::from_srgb_hex_rgb(0x14_1b_24));
    assert_eq!(chrome.thumb_border, Color::from_srgb_hex_rgb(0x3b_47_58));
}

#[test]
fn slider_paint_applies_disabled_alpha_to_all_chrome_channels() {
    let mut app = App::new();
    install_slider_colors(&mut app);

    let theme = Theme::global(&app);
    let enabled = resolve_slider_paint(theme, true, true, false, false);
    let disabled = resolve_slider_paint(theme, false, false, false, false);

    assert_color_close(
        disabled.track_bg,
        Color {
            a: enabled.track_bg.a * 0.55,
            ..enabled.track_bg
        },
    );
    assert_color_close(
        disabled.fill_bg,
        Color {
            a: enabled.fill_bg.a * 0.55,
            ..enabled.fill_bg
        },
    );
    assert_color_close(
        disabled.thumb_bg,
        Color {
            a: enabled.thumb_bg.a * 0.55,
            ..enabled.thumb_bg
        },
    );
    assert_color_close(
        disabled.thumb_border,
        Color {
            a: enabled.thumb_border.a * 0.55,
            ..enabled.thumb_border
        },
    );
}

#[test]
fn slider_paint_mixes_hover_and_pressed_track_fill_when_enabled() {
    let mut app = App::new();
    install_slider_colors(&mut app);

    let theme = Theme::global(&app);
    let base = resolve_slider_paint(theme, true, true, false, false);
    let hovered = resolve_slider_paint(theme, true, true, true, false);
    let pressed = resolve_slider_paint(theme, true, true, true, true);
    let disabled_hovered = resolve_slider_paint(theme, false, false, true, true);

    assert_ne!(hovered.track_bg, base.track_bg);
    assert_ne!(hovered.fill_bg, base.fill_bg);
    assert_ne!(pressed.track_bg, hovered.track_bg);
    assert_ne!(pressed.fill_bg, hovered.fill_bg);

    assert_color_close(
        disabled_hovered.track_bg,
        Color {
            a: base.track_bg.a * 0.55,
            ..base.track_bg
        },
    );
    assert_color_close(
        disabled_hovered.fill_bg,
        Color {
            a: base.fill_bg.a * 0.55,
            ..base.fill_bg
        },
    );
}
