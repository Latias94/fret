use fret_app::App;
use fret_core::{Color, Px};
use fret_ui::{Theme, ThemeConfig};

use super::{
    EditorPopupListRowState, editor_popup_list_content_height,
    editor_popup_list_default_max_content_height, editor_popup_list_row_gap,
    editor_popup_list_row_palette,
};
use crate::primitives::EditorTokenKeys;

#[test]
fn popup_list_row_palette_uses_editor_highlight_and_muted_disabled_foreground() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert(
            EditorTokenKeys::CHROME_ACCENT.to_string(),
            "#355a86".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::CHROME_MUTED_FG.to_string(),
            "#8aa1b7".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::TEXT_FIELD_FG.to_string(),
            "#f0f4f8".to_string(),
        );
        cfg.colors
            .insert("accent-foreground".to_string(), "#fcfdff".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let active = editor_popup_list_row_palette(
        theme,
        false,
        EditorPopupListRowState {
            active: true,
            disabled: false,
        },
    );
    assert_eq!(active.bg, Some(Color::from_srgb_hex_rgb(0x35_5a_86)));
    assert_eq!(active.fg, Color::from_srgb_hex_rgb(0xfc_fd_ff));

    let disabled = editor_popup_list_row_palette(
        theme,
        true,
        EditorPopupListRowState {
            active: false,
            disabled: true,
        },
    );
    assert_eq!(disabled.bg, Some(Color::from_srgb_hex_rgb(0x35_5a_86)));
    assert_eq!(disabled.fg, Color::from_srgb_hex_rgb(0x8a_a1_b7));
}

#[test]
fn popup_list_height_helpers_share_the_same_row_gap_budget() {
    assert_eq!(editor_popup_list_row_gap(), Px(2.0));
    assert_eq!(editor_popup_list_content_height(Px(28.0), 3), Px(88.0));
    assert_eq!(
        editor_popup_list_default_max_content_height(Px(28.0)),
        Px(178.0)
    );
}
