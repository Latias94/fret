use super::{
    EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals, editor_icon_button_bg,
    hover_overlay_bg, mix,
};
use crate::primitives::EditorTokenKeys;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::colors::editor_subtle_bg;
use fret_app::App;
use fret_core::{Color, Edges, Px};
use fret_ui::{Theme, ThemeConfig};

fn test_chrome() -> ResolvedEditorFrameChrome {
    ResolvedEditorFrameChrome {
        padding: Edges::all(Px(0.0)),
        radius: Px(4.0),
        border_width: Px(1.0),
        bg: Color::from_srgb_hex_rgb(0x18_18_18),
        border: Color::from_srgb_hex_rgb(0x44_44_44),
        border_focus: Color::from_srgb_hex_rgb(0x33_99_ff),
        fg: Color::from_srgb_hex_rgb(0xee_ee_ee),
        text_px: Px(12.0),
    }
}

#[test]
fn selection_frame_visuals_use_selected_fill_and_foreground() {
    let app = App::new();
    let theme = Theme::global(&app);
    let visuals = EditorWidgetVisuals::new(theme).selection_frame_visuals(
        test_chrome(),
        EditorFrameState {
            enabled: true,
            ..Default::default()
        },
        Color::from_srgb_hex_rgb(0x20_20_20),
        Color::from_srgb_hex_rgb(0x55_88_cc),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        true,
    );

    assert_eq!(visuals.bg, Color::from_srgb_hex_rgb(0x55_88_cc));
    assert_eq!(visuals.fg, Color::from_srgb_hex_rgb(0xff_ff_ff));
    assert_eq!(visuals.icon, Color::from_srgb_hex_rgb(0xff_ff_ff));
}

#[test]
fn selection_frame_visuals_use_focus_border_when_focused() {
    let app = App::new();
    let theme = Theme::global(&app);
    let chrome = test_chrome();
    let visuals = EditorWidgetVisuals::new(theme).selection_frame_visuals(
        chrome,
        EditorFrameState {
            enabled: true,
            focused: true,
            ..Default::default()
        },
        Color::from_srgb_hex_rgb(0x20_20_20),
        Color::from_srgb_hex_rgb(0x55_88_cc),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        false,
    );

    assert_eq!(visuals.border, chrome.border_focus);
}

#[test]
fn selection_frame_visuals_reduce_alpha_when_disabled() {
    let app = App::new();
    let theme = Theme::global(&app);
    let selected_bg = Color::from_srgb_hex_rgb(0x55_88_cc);
    let selected_fg = Color::from_srgb_hex_rgb(0xff_ff_ff);
    let visuals = EditorWidgetVisuals::new(theme).selection_frame_visuals(
        test_chrome(),
        EditorFrameState {
            enabled: false,
            ..Default::default()
        },
        Color::from_srgb_hex_rgb(0x20_20_20),
        selected_bg,
        selected_fg,
        true,
    );

    assert!(visuals.bg.a < selected_bg.a);
    assert!(visuals.fg.a < selected_fg.a);
}

#[test]
fn frame_visuals_tint_typing_state_more_than_focus_only() {
    let app = App::new();
    let theme = Theme::global(&app);
    let chrome = test_chrome();
    let visuals_focus = EditorWidgetVisuals::new(theme).frame_visuals(
        chrome,
        EditorFrameState {
            enabled: true,
            focused: true,
            ..Default::default()
        },
    );
    let visuals_typing = EditorWidgetVisuals::new(theme).frame_visuals(
        chrome,
        EditorFrameState {
            enabled: true,
            focused: true,
            semantic: EditorFrameSemanticState {
                typing: true,
                invalid: false,
            },
            ..Default::default()
        },
    );

    assert_ne!(visuals_focus.bg, visuals_typing.bg);
    assert_eq!(visuals_focus.border, chrome.border_focus);
    assert_eq!(visuals_typing.border, chrome.border_focus);
}

#[test]
fn frame_visuals_use_shared_invalid_chrome() {
    let app = App::new();
    let theme = Theme::global(&app);
    let widget_visuals = EditorWidgetVisuals::new(theme);
    let invalid_border = widget_visuals.control_invalid_border();
    let invalid_bg = widget_visuals.control_invalid_bg(test_chrome().bg, invalid_border);
    let visuals = EditorWidgetVisuals::new(theme).frame_visuals(
        test_chrome(),
        EditorFrameState {
            enabled: true,
            semantic: EditorFrameSemanticState {
                typing: false,
                invalid: true,
            },
            ..Default::default()
        },
    );

    assert_eq!(visuals.border, invalid_border);
    assert_eq!(visuals.bg, mix(test_chrome().bg, invalid_bg, 0.96));
    assert_eq!(
        widget_visuals.control_invalid_fg(),
        theme.color_token("destructive")
    );
}

#[test]
fn icon_button_bg_prefers_editor_subtle_bg_over_host_background() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("background".to_string(), "#ffffff".to_string());
        cfg.colors.insert(
            EditorTokenKeys::TEXT_FIELD_BG.to_string(),
            "#141b24".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::CHROME_ACCENT.to_string(),
            "#355a86".to_string(),
        );
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let hovered = editor_icon_button_bg(theme, true, true, false)
        .expect("hovered icon button should render a background");

    assert_eq!(
        hovered,
        hover_overlay_bg(theme, editor_subtle_bg(theme), true, false)
    );
    assert_ne!(
        hovered,
        hover_overlay_bg(theme, theme.color_token("background"), true, false)
    );
}
