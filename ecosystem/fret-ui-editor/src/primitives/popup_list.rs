use fret_core::{Color, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography;

use super::colors::{editor_accent, editor_foreground, editor_muted_foreground};

const DEFAULT_EDITOR_POPUP_LIST_ROW_GAP: Px = Px(2.0);
const DEFAULT_EDITOR_POPUP_LIST_SURFACE_PADDING: Px = Px(4.0);
const DEFAULT_EDITOR_POPUP_LIST_ROW_RADIUS: Px = Px(6.0);
const DEFAULT_EDITOR_POPUP_SIDE_OFFSET: Px = Px(4.0);
const DEFAULT_EDITOR_POPUP_WINDOW_MARGIN: Px = Px(8.0);
const DEFAULT_EDITOR_POPUP_LIST_MAX_VISIBLE_ROWS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct EditorPopupListRowState {
    pub(crate) active: bool,
    pub(crate) disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EditorPopupListRowPalette {
    pub(crate) bg: Option<Color>,
    pub(crate) fg: Color,
}

pub(crate) fn editor_popup_list_row_gap() -> Px {
    DEFAULT_EDITOR_POPUP_LIST_ROW_GAP
}

pub(crate) fn editor_popup_list_surface_padding() -> Px {
    DEFAULT_EDITOR_POPUP_LIST_SURFACE_PADDING
}

pub(crate) fn editor_popup_list_row_radius() -> Px {
    DEFAULT_EDITOR_POPUP_LIST_ROW_RADIUS
}

pub(crate) fn editor_popup_side_offset() -> Px {
    DEFAULT_EDITOR_POPUP_SIDE_OFFSET
}

pub(crate) fn editor_popup_window_margin() -> Px {
    DEFAULT_EDITOR_POPUP_WINDOW_MARGIN
}

pub(crate) fn editor_popup_list_content_height(row_height: Px, visible_count: usize) -> Px {
    let row_count = visible_count as f32;
    let gaps = visible_count.saturating_sub(1) as f32;
    Px(row_count * row_height.0 + gaps * editor_popup_list_row_gap().0)
}

pub(crate) fn editor_popup_list_default_max_content_height(row_height: Px) -> Px {
    let rows = DEFAULT_EDITOR_POPUP_LIST_MAX_VISIBLE_ROWS as f32;
    let gaps = DEFAULT_EDITOR_POPUP_LIST_MAX_VISIBLE_ROWS.saturating_sub(1) as f32;
    Px(rows * row_height.0 + gaps * editor_popup_list_row_gap().0)
}

pub(crate) fn editor_popup_list_row_text_style(row_height: Px) -> TextStyle {
    typography::as_control_text(TextStyle {
        size: Px(12.0),
        line_height: Some(row_height),
        ..Default::default()
    })
}

pub(crate) fn editor_popup_list_row_palette(
    theme: &Theme,
    hovered: bool,
    state: EditorPopupListRowState,
) -> EditorPopupListRowPalette {
    let highlighted = state.active || hovered;
    let fg = if state.disabled {
        editor_muted_foreground(theme)
    } else if highlighted {
        theme.color_token("accent-foreground")
    } else {
        editor_foreground(theme)
    };

    EditorPopupListRowPalette {
        bg: highlighted.then(|| editor_accent(theme)),
        fg,
    }
}

#[cfg(test)]
mod tests {
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
}
