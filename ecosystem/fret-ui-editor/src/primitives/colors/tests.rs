use fret_app::App;
use fret_core::Color;
use fret_ui::{Theme, ThemeConfig};

use super::{
    editor_accent, editor_border, editor_focus_ring, editor_foreground, editor_invalid_border,
    editor_invalid_foreground, editor_muted_foreground, editor_panel_background,
    editor_panel_border, editor_panel_header_background, editor_panel_header_border,
    editor_popup_background, editor_popup_border, editor_property_group_border,
    editor_property_header_background, editor_property_header_border,
    editor_property_header_foreground, editor_subtle_bg,
};
use crate::primitives::EditorTokenKeys;

#[test]
fn editor_semantic_color_helpers_prefer_editor_owned_keys() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert(
            EditorTokenKeys::TEXT_FIELD_BG.to_string(),
            "#141b24".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::TEXT_FIELD_FG.to_string(),
            "#f0f4f8".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::CHROME_MUTED_FG.to_string(),
            "#8aa1b7".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::CHROME_ACCENT.to_string(),
            "#355a86".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::CHROME_RING.to_string(),
            "#7faee8".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::TEXT_FIELD_BORDER.to_string(),
            "#3b4758".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_PANEL_BG.to_string(),
            "#0f151d".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_PANEL_BORDER.to_string(),
            "#3d4d5f".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_PANEL_HEADER_BG.to_string(),
            "#243445".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_PANEL_HEADER_BORDER.to_string(),
            "#5a7087".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_GROUP_BORDER.to_string(),
            "#33414f".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_HEADER_BG.to_string(),
            "#19232e".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_HEADER_BORDER.to_string(),
            "#384857".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_HEADER_FG.to_string(),
            "#edf3fa".to_string(),
        );
        cfg.colors
            .insert(EditorTokenKeys::POPUP_BG.to_string(), "#16212d".to_string());
        cfg.colors.insert(
            EditorTokenKeys::POPUP_BORDER.to_string(),
            "#4f6478".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::CONTROL_INVALID_FG.to_string(),
            "#ffd3d6".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::CONTROL_INVALID_BORDER.to_string(),
            "#c76f77".to_string(),
        );
        cfg.colors
            .insert("foreground".to_string(), "#ffffff".to_string());
        cfg.colors
            .insert("border".to_string(), "#ff8800".to_string());
        cfg.colors
            .insert("muted-foreground".to_string(), "#00ff00".to_string());
        cfg.colors
            .insert("accent".to_string(), "#ff00ff".to_string());
        cfg.colors.insert("ring".to_string(), "#ff8800".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    assert_eq!(
        editor_foreground(theme),
        Color::from_srgb_hex_rgb(0xf0_f4_f8)
    );
    assert_eq!(
        editor_muted_foreground(theme),
        Color::from_srgb_hex_rgb(0x8a_a1_b7)
    );
    assert_eq!(editor_accent(theme), Color::from_srgb_hex_rgb(0x35_5a_86));
    assert_eq!(
        editor_focus_ring(theme),
        Color::from_srgb_hex_rgb(0x7f_ae_e8)
    );
    assert_eq!(
        editor_invalid_foreground(theme),
        Color::from_srgb_hex_rgb(0xff_d3_d6)
    );
    assert_eq!(
        editor_invalid_border(theme),
        Color::from_srgb_hex_rgb(0xc7_6f_77)
    );
    assert_eq!(editor_border(theme), Color::from_srgb_hex_rgb(0x3b_47_58));
    assert_eq!(
        editor_panel_background(theme),
        Color::from_srgb_hex_rgb(0x0f_15_1d)
    );
    assert_eq!(
        editor_panel_border(theme),
        Color::from_srgb_hex_rgb(0x3d_4d_5f)
    );
    assert_eq!(
        editor_panel_header_background(theme),
        Color::from_srgb_hex_rgb(0x24_34_45)
    );
    assert_eq!(
        editor_panel_header_border(theme),
        Color::from_srgb_hex_rgb(0x5a_70_87)
    );
    assert_eq!(
        editor_property_group_border(theme),
        Color::from_srgb_hex_rgb(0x33_41_4f)
    );
    assert_eq!(
        editor_property_header_background(theme),
        Color::from_srgb_hex_rgb(0x19_23_2e)
    );
    assert_eq!(
        editor_property_header_border(theme),
        Color::from_srgb_hex_rgb(0x38_48_57)
    );
    assert_eq!(
        editor_property_header_foreground(theme),
        Color::from_srgb_hex_rgb(0xed_f3_fa)
    );
    assert_eq!(
        editor_popup_background(theme),
        Color::from_srgb_hex_rgb(0x16_21_2d)
    );
    assert_eq!(
        editor_popup_border(theme),
        Color::from_srgb_hex_rgb(0x4f_64_78)
    );
    assert_eq!(
        editor_subtle_bg(theme),
        Color::from_srgb_hex_rgb(0x14_1b_24)
    );
}

#[test]
fn editor_focus_ring_color_falls_back_to_text_field_focus_border() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert(
            EditorTokenKeys::TEXT_FIELD_BORDER_FOCUS.to_string(),
            "#6ea8e0".to_string(),
        );
        cfg.colors.insert("ring".to_string(), "#ff8800".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    assert_eq!(
        editor_focus_ring(theme),
        Color::from_srgb_hex_rgb(0x6e_a8_e0)
    );
}

#[test]
fn editor_muted_foreground_keeps_shared_palette_fallback() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("muted-foreground".to_string(), "#9eabbc".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    assert_eq!(
        editor_muted_foreground(theme),
        Color::from_srgb_hex_rgb(0x9e_ab_bc)
    );
}

#[test]
fn editor_invalid_border_falls_back_to_numeric_error_lane() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert(
            EditorTokenKeys::NUMERIC_ERROR_BORDER.to_string(),
            "#d06a6a".to_string(),
        );
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    assert_eq!(
        editor_invalid_border(theme),
        Color::from_srgb_hex_rgb(0xd0_6a_6a)
    );
}

#[test]
fn editor_panel_helpers_keep_shared_surface_fallbacks() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert("card".to_string(), "#202328".to_string());
        cfg.colors
            .insert("border".to_string(), "#454d59".to_string());
        cfg.colors
            .insert("muted".to_string(), "#2a2d33".to_string());
        cfg.colors
            .insert("foreground".to_string(), "#e6e8eb".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    assert_eq!(
        editor_panel_background(theme),
        Color::from_srgb_hex_rgb(0x20_23_28)
    );
    assert_eq!(
        editor_panel_border(theme),
        Color::from_srgb_hex_rgb(0x45_4d_59)
    );
    assert_eq!(
        editor_panel_header_background(theme),
        Color::from_srgb_hex_rgb(0x2a_2d_33)
    );
    assert_eq!(
        editor_property_header_foreground(theme),
        Color::from_srgb_hex_rgb(0xe6_e8_eb)
    );
}

#[test]
fn editor_popup_helpers_fall_back_to_legacy_component_then_popover_and_panel_tokens() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("component.text_field.bg".to_string(), "#1c2530".to_string());
        cfg.colors.insert(
            "component.text_field.border".to_string(),
            "#4b6074".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_PANEL_BG.to_string(),
            "#0f151d".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_PANEL_BORDER.to_string(),
            "#3d4d5f".to_string(),
        );
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    assert_eq!(
        editor_popup_background(theme),
        Color::from_srgb_hex_rgb(0x1c_25_30)
    );
    assert_eq!(
        editor_popup_border(theme),
        Color::from_srgb_hex_rgb(0x4b_60_74)
    );

    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("popover".to_string(), "#101820".to_string());
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_PANEL_BG.to_string(),
            "#0f151d".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::PROPERTY_PANEL_BORDER.to_string(),
            "#3d4d5f".to_string(),
        );
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    assert_eq!(
        editor_popup_background(theme),
        Color::from_srgb_hex_rgb(0x10_18_20)
    );
    assert_eq!(
        editor_popup_border(theme),
        Color::from_srgb_hex_rgb(0x3d_4d_5f)
    );
}
