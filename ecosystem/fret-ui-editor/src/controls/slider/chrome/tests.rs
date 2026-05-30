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
