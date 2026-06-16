use super::{
    ResolvedEditorFrameChrome, resolve_editor_text_area_field_style,
    resolve_editor_text_field_style,
};
use crate::primitives::EditorTokenKeys;
use fret_app::App;
use fret_core::{Color, Edges, Px, TextLineHeightPolicy, TextVerticalPlacement};
use fret_ui::{Theme, ThemeConfig};
use fret_ui_kit::{ChromeRefinement, Size};

#[test]
fn editor_text_field_style_uses_control_intent_defaults() {
    let app = App::new();
    let theme = Theme::global(&app);
    let (_chrome, style) =
        resolve_editor_text_field_style(theme, Size::Small, &ChromeRefinement::default());

    assert!(style.line_height.is_some());
    assert_eq!(
        style.line_height_policy,
        TextLineHeightPolicy::FixedFromStyle
    );
    assert_eq!(
        style.vertical_placement,
        TextVerticalPlacement::BoundsAsLineBox
    );
}

#[test]
fn editor_text_area_style_uses_content_intent_defaults() {
    let app = App::new();
    let theme = Theme::global(&app);
    let (_chrome, style) =
        resolve_editor_text_area_field_style(theme, Size::Small, &ChromeRefinement::default());

    assert!(style.line_height.is_some());
    assert_eq!(style.line_height_policy, TextLineHeightPolicy::ExpandToFit);
    assert_eq!(
        style.vertical_placement,
        TextVerticalPlacement::CenterMetricsBox
    );
}

#[test]
fn editor_text_area_style_uses_editor_focus_ring_token() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert(
            EditorTokenKeys::CHROME_RING.to_string(),
            "#7faee8".to_string(),
        );
        cfg.colors.insert("ring".to_string(), "#ff0000".to_string());
        cfg.colors
            .insert("primary".to_string(), "#00ff00".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let (chrome, _style) =
        resolve_editor_text_area_field_style(theme, Size::Small, &ChromeRefinement::default());

    let ring = chrome.focus_ring.expect("text area should keep focus ring");
    assert_eq!(ring.color, Color::from_srgb_hex_rgb(0x7f_ae_e8));
    assert_eq!(
        chrome.preedit_underline_color,
        Color::from_srgb_hex_rgb(0x7f_ae_e8)
    );
}

#[test]
fn editor_text_field_style_prefers_editor_tokens_over_legacy_component_tokens() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert(
            EditorTokenKeys::TEXT_FIELD_BG.to_string(),
            "#141b24".to_string(),
        );
        cfg.colors
            .insert("component.text_field.bg".to_string(), "#ffffff".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let (chrome, _style) =
        resolve_editor_text_field_style(theme, Size::Small, &ChromeRefinement::default());

    assert_eq!(chrome.background, Color::from_srgb_hex_rgb(0x14_1b_24));
}

#[test]
fn editor_text_field_style_keeps_legacy_component_text_field_fallback() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("component.text_field.bg".to_string(), "#141b24".to_string());
        cfg.metrics
            .insert("component.text_field.min_height".to_string(), 29.0);
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let (chrome, _style) =
        resolve_editor_text_field_style(theme, Size::Small, &ChromeRefinement::default());

    assert_eq!(chrome.background, Color::from_srgb_hex_rgb(0x14_1b_24));
    assert_eq!(chrome.padding.top, Size::Small.input_py(theme));
    assert_eq!(chrome.border, Edges::all(Px(1.0)));
}

#[test]
fn resolved_editor_frame_chrome_reports_outer_control_height() {
    let chrome = ResolvedEditorFrameChrome {
        padding: Edges {
            top: Px(3.0),
            right: Px(5.0),
            bottom: Px(4.0),
            left: Px(5.0),
        },
        radius: Px(2.0),
        border_width: Px(1.0),
        bg: Color::from_srgb_hex_rgb(0x11_11_11),
        border: Color::from_srgb_hex_rgb(0x22_22_22),
        border_focus: Color::from_srgb_hex_rgb(0x33_33_33),
        fg: Color::from_srgb_hex_rgb(0xee_ee_ee),
        text_px: Px(12.0),
    };

    assert_eq!(chrome.control_outer_height(Px(22.0)), Px(31.0));
}
