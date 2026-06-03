use fret_core::Color;
use fret_ui::Theme;

use crate::primitives::{chrome::ResolvedEditorFrameChrome, colors::editor_accent};

use super::{
    EditorFrameState, EditorFrameVisuals,
    color_math::{alpha_mul, mix},
};

pub(super) fn selection_frame_visuals(
    theme: &Theme,
    chrome: ResolvedEditorFrameChrome,
    state: EditorFrameState,
    base_bg: Color,
    selected_bg: Color,
    selected_fg: Color,
    selected: bool,
) -> EditorFrameVisuals {
    let disabled_alpha = if state.enabled { 1.0 } else { 0.55 };

    let accent = editor_accent(theme);
    let mut bg = alpha_mul(if selected { selected_bg } else { base_bg }, disabled_alpha);
    let mut border = alpha_mul(
        if selected {
            mix(chrome.border, selected_bg, 0.35)
        } else {
            chrome.border
        },
        disabled_alpha,
    );
    let fg = alpha_mul(
        if selected { selected_fg } else { chrome.fg },
        disabled_alpha,
    );

    if state.hovered && state.enabled {
        bg = mix(bg, accent, if selected { 0.05 } else { 0.08 });
        border = mix(border, accent, if selected { 0.08 } else { 0.10 });
    }
    if state.pressed && state.enabled {
        bg = mix(bg, accent, if selected { 0.10 } else { 0.14 });
        border = mix(border, accent, if selected { 0.12 } else { 0.16 });
    }
    if (state.focused || state.open) && state.enabled {
        bg = mix(bg, accent, if selected { 0.04 } else { 0.08 });
        border = chrome.border_focus;
    }

    EditorFrameVisuals {
        bg,
        border,
        fg,
        icon: fg,
    }
}
