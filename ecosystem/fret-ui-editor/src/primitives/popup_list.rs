use fret_core::{Color, Px};
use fret_ui::Theme;

use super::colors::{editor_accent, editor_foreground, editor_muted_foreground};

#[cfg(test)]
mod tests;

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
