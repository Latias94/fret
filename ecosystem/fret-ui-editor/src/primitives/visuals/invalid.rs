use fret_core::Color;
use fret_ui::Theme;

use crate::primitives::{
    EditorTokenKeys,
    colors::{editor_invalid_border, editor_invalid_foreground},
};

use super::color_math::mix;

pub(super) fn control_invalid_fg(theme: &Theme) -> Color {
    editor_invalid_foreground(theme)
}

pub(super) fn control_invalid_border(theme: &Theme) -> Color {
    editor_invalid_border(theme)
}

pub(super) fn control_invalid_bg(theme: &Theme, base: Color, border: Color) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CONTROL_INVALID_BG)
        .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_BG))
        .unwrap_or_else(|| {
            let mut out = mix(base, Color { a: 1.0, ..border }, 0.10);
            out.a = 1.0;
            out
        })
}
