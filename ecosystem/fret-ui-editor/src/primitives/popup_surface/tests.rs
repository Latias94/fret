use fret_app::App;
use fret_core::{Color, Px};
use fret_ui::{Theme, ThemeConfig};

use super::resolve_editor_popup_surface_chrome;
use crate::primitives::EditorTokenKeys;
use crate::theme::{EditorThemePreset, apply_editor_theme_preset};

#[test]
fn overlay_popup_surface_adds_shadow() {
    let app = App::new();
    let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), true);
    assert!(chrome.shadow.is_some());
}

#[test]
fn inline_popup_surface_skips_shadow() {
    let app = App::new();
    let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), false);
    assert!(chrome.shadow.is_none());
}

#[test]
fn editor_popup_surface_prefers_editor_owned_popup_tokens() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("popover".to_string(), "#ffffff".to_string());
        theme.apply_config_patch(&cfg);
    });
    apply_editor_theme_preset(&mut app, EditorThemePreset::Default);

    let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), true);
    assert_eq!(
        chrome.bg,
        Color::from_srgb_hex_rgb(0x13_1b_25),
        "editor popup background should not fall back to host popover"
    );
    assert_eq!(
        chrome.border,
        Theme::global(&app)
            .color_by_key(EditorTokenKeys::POPUP_BORDER)
            .unwrap()
    );
}

#[test]
fn popup_surface_respects_editor_popup_radius_and_shadow_metrics() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.metrics
            .insert(EditorTokenKeys::POPUP_RADIUS.to_string(), 5.0);
        cfg.metrics
            .insert(EditorTokenKeys::POPUP_SHADOW_OFFSET_Y.to_string(), 3.0);
        cfg.metrics
            .insert(EditorTokenKeys::POPUP_SHADOW_BLUR.to_string(), 9.0);
        cfg.metrics
            .insert(EditorTokenKeys::POPUP_SHADOW_SPREAD.to_string(), -2.0);
        theme.apply_config_patch(&cfg);
    });

    let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), true);
    assert_eq!(chrome.radius, Px(5.0));
    let shadow = chrome.shadow.expect("overlay popup should keep shadow");
    assert_eq!(shadow.primary.offset_y, Px(3.0));
    assert_eq!(shadow.primary.blur, Px(9.0));
    assert_eq!(shadow.primary.spread, Px(-2.0));
}

#[test]
fn popup_surface_respects_editor_shadow_color_token() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert(
            EditorTokenKeys::POPUP_SHADOW_COLOR.to_string(),
            "#010203".to_string(),
        );
        cfg.colors
            .insert("muted".to_string(), "#ff0000".to_string());
        theme.apply_config_patch(&cfg);
    });

    let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), true);
    let shadow = chrome.shadow.expect("overlay popup should keep shadow");
    assert_eq!(shadow.primary.color, Color::from_srgb_hex_rgb(0x01_02_03));
}

#[test]
fn dense_preset_uses_tighter_popup_radius_than_default() {
    let mut default_app = App::new();
    apply_editor_theme_preset(&mut default_app, EditorThemePreset::Default);
    let default_chrome = resolve_editor_popup_surface_chrome(Theme::global(&default_app), true);

    let mut dense_app = App::new();
    apply_editor_theme_preset(&mut dense_app, EditorThemePreset::ImguiLikeDense);
    let dense_chrome = resolve_editor_popup_surface_chrome(Theme::global(&dense_app), true);

    assert!(dense_chrome.radius.0 < default_chrome.radius.0);
    let default_shadow = default_chrome.shadow.expect("default overlay shadow");
    let dense_shadow = dense_chrome.shadow.expect("dense overlay shadow");
    assert!(dense_shadow.primary.blur.0 < default_shadow.primary.blur.0);
}
