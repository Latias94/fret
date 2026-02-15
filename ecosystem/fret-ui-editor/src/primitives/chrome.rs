//! Shared chrome resolution helpers for editor controls.
//!
//! v1 goal: keep "frame" defaults consistent across controls (inputs, triggers, scrub surfaces)
//! without hard-binding `fret-ui-editor` to a specific design system crate.

use fret_core::{Color, Edges, Px};
use fret_ui::Theme;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, Size};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedEditorFrameChrome {
    pub(crate) padding: Edges,
    pub(crate) radius: Px,
    pub(crate) border_width: Px,
    pub(crate) bg: Color,
    pub(crate) border: Color,
    pub(crate) border_focus: Color,
    pub(crate) fg: Color,
    pub(crate) text_px: Px,
}

pub(crate) fn resolve_editor_frame_chrome(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
    keys: InputTokenKeys,
) -> ResolvedEditorFrameChrome {
    let resolved = resolve_input_chrome(theme, size, refinement, keys);
    ResolvedEditorFrameChrome {
        padding: resolved.padding,
        radius: resolved.radius,
        border_width: resolved.border_width,
        bg: resolved.background,
        border: resolved.border_color,
        border_focus: resolved.border_color_focused,
        fg: resolved.text_color,
        text_px: resolved.text_px,
    }
}
