use fret_ui::Theme;

use crate::primitives::{
    chrome::ResolvedEditorFrameChrome,
    colors::{editor_accent, editor_muted_foreground},
};

use super::{
    EditorFrameState, EditorFrameVisuals,
    color_math::{alpha_mul, mix},
    invalid,
};

pub(super) fn frame_visuals(
    theme: &Theme,
    chrome: ResolvedEditorFrameChrome,
    state: EditorFrameState,
) -> EditorFrameVisuals {
    // Keep disabled visuals conservative: only scale alpha and avoid color shifts that can reduce
    // contrast too much on dark themes.
    let disabled_alpha = if state.enabled { 1.0 } else { 0.55 };

    let accent = editor_accent(theme);
    let mut bg = alpha_mul(chrome.bg, disabled_alpha);
    let mut border = alpha_mul(chrome.border, disabled_alpha);
    let fg = alpha_mul(chrome.fg, disabled_alpha);
    let mut icon = alpha_mul(editor_muted_foreground(theme), disabled_alpha);

    if state.hovered && state.enabled {
        bg = mix(bg, accent, 0.08);
        border = mix(border, accent, 0.10);
    }
    if state.pressed && state.enabled {
        bg = mix(bg, accent, 0.14);
        border = mix(border, accent, 0.16);
    }
    if (state.focused || state.open) && state.enabled {
        bg = mix(bg, accent, 0.08);
        border = chrome.border_focus;
    }
    if state.semantic.typing && state.enabled {
        bg = mix(
            bg,
            accent,
            if state.focused || state.open {
                0.14
            } else {
                0.11
            },
        );
        border = mix(border, chrome.border_focus, 0.72);
        icon = mix(icon, chrome.border_focus, 0.24);
    }
    if state.semantic.invalid && state.enabled {
        let invalid_fg = invalid::control_invalid_fg(theme);
        let invalid_border = invalid::control_invalid_border(theme);
        let invalid_bg = invalid::control_invalid_bg(theme, chrome.bg, invalid_border);

        bg = mix(
            bg,
            invalid_bg,
            if state.semantic.typing { 0.90 } else { 0.96 },
        );
        border = if state.focused || state.open {
            mix(invalid_border, chrome.border_focus, 0.12)
        } else {
            invalid_border
        };
        icon = mix(icon, invalid_fg, 0.36);
    }

    EditorFrameVisuals {
        bg,
        border,
        fg,
        icon,
    }
}
