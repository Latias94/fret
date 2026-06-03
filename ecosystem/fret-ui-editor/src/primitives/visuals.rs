//! Editor-grade widget visuals policy.
//!
//! This module intentionally lives in `fret-ui-editor` (ecosystem/policy layer). It provides a
//! small, reusable mapping from theme tokens + widget interaction state into consistent "chrome"
//! colors so controls don't drift.

use fret_core::Color;
use fret_ui::Theme;

use super::{
    chrome::ResolvedEditorFrameChrome,
    colors::{editor_accent, editor_border, editor_subtle_bg},
};

mod color_math;
mod frame;
mod invalid;
mod selection;

use color_math::mix;

#[cfg(test)]
mod tests;

/// Shared editor-grade widget visuals policy.
///
/// This mirrors the intent of egui's `Visuals::widgets`: provide a single place to resolve
/// interaction-state-dependent chrome so controls don't drift.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EditorWidgetVisuals<'a> {
    theme: &'a Theme,
}

impl<'a> EditorWidgetVisuals<'a> {
    pub(crate) fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    pub(crate) fn hover_overlay_bg_custom(
        &self,
        base: Color,
        hovered: bool,
        pressed: bool,
        hover_mix: f32,
        press_mix: f32,
    ) -> Color {
        let accent = editor_accent(self.theme);
        let mut out = base;
        if hovered {
            out = mix(out, accent, hover_mix);
        }
        if pressed {
            out = mix(out, accent, press_mix);
        }
        out
    }

    pub(crate) fn hover_overlay_border_custom(
        &self,
        base: Color,
        hovered: bool,
        pressed: bool,
        hover_mix: f32,
        press_mix: f32,
    ) -> Color {
        let accent = editor_accent(self.theme);
        let mut out = base;
        if hovered {
            out = mix(out, accent, hover_mix);
        }
        if pressed {
            out = mix(out, accent, press_mix);
        }
        out
    }

    pub(crate) fn hover_overlay_bg(&self, base: Color, hovered: bool, pressed: bool) -> Color {
        self.hover_overlay_bg_custom(base, hovered, pressed, 0.06, 0.10)
    }

    pub(crate) fn hover_overlay_border(&self, base: Color, hovered: bool, pressed: bool) -> Color {
        self.hover_overlay_border_custom(base, hovered, pressed, 0.10, 0.14)
    }

    pub(crate) fn icon_button_bg(
        &self,
        enabled: bool,
        hovered: bool,
        pressed: bool,
    ) -> Option<Color> {
        if !enabled {
            return None;
        }
        if !hovered && !pressed {
            return None;
        }

        Some(self.hover_overlay_bg(editor_subtle_bg(self.theme), hovered, pressed))
    }

    pub(crate) fn icon_button_border(
        &self,
        enabled: bool,
        hovered: bool,
        pressed: bool,
    ) -> Option<Color> {
        if !enabled {
            return None;
        }
        if !hovered && !pressed {
            return None;
        }

        let base = editor_border(self.theme);

        Some(self.hover_overlay_border(base, hovered, pressed))
    }

    /// Compute input-like frame visuals for the given interaction state.
    ///
    /// This is a small helper intended for editor controls built from `Container` + `Pressable`.
    pub(crate) fn frame_visuals(
        &self,
        chrome: ResolvedEditorFrameChrome,
        state: EditorFrameState,
    ) -> EditorFrameVisuals {
        frame::frame_visuals(self.theme, chrome, state)
    }

    /// Compute selection/toggle-like frame visuals (checkboxes, segmented toggles, etc.).
    ///
    /// This keeps "selected vs unselected" chrome on the same interaction-state policy as the
    /// rest of the editor control set while still allowing selected surfaces to use a stronger
    /// fill/foreground pair than plain text inputs.
    pub(crate) fn selection_frame_visuals(
        &self,
        chrome: ResolvedEditorFrameChrome,
        state: EditorFrameState,
        base_bg: Color,
        selected_bg: Color,
        selected_fg: Color,
        selected: bool,
    ) -> EditorFrameVisuals {
        selection::selection_frame_visuals(
            self.theme,
            chrome,
            state,
            base_bg,
            selected_bg,
            selected_fg,
            selected,
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EditorFrameSemanticState {
    pub(crate) typing: bool,
    pub(crate) invalid: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EditorFrameState {
    pub(crate) enabled: bool,
    pub(crate) hovered: bool,
    pub(crate) pressed: bool,
    pub(crate) focused: bool,
    pub(crate) open: bool,
    pub(crate) semantic: EditorFrameSemanticState,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EditorFrameVisuals {
    pub(crate) bg: Color,
    pub(crate) border: Color,
    pub(crate) fg: Color,
    pub(crate) icon: Color,
}

#[cfg(test)]
impl<'a> EditorWidgetVisuals<'a> {
    fn control_invalid_fg(&self) -> Color {
        invalid::control_invalid_fg(self.theme)
    }

    fn control_invalid_border(&self) -> Color {
        invalid::control_invalid_border(self.theme)
    }

    fn control_invalid_bg(&self, base: Color, border: Color) -> Color {
        invalid::control_invalid_bg(self.theme, base, border)
    }
}

pub(crate) fn hover_overlay_bg(theme: &Theme, base: Color, hovered: bool, pressed: bool) -> Color {
    EditorWidgetVisuals::new(theme).hover_overlay_bg(base, hovered, pressed)
}

pub(crate) fn editor_icon_button_bg(
    theme: &Theme,
    enabled: bool,
    hovered: bool,
    pressed: bool,
) -> Option<Color> {
    EditorWidgetVisuals::new(theme).icon_button_bg(enabled, hovered, pressed)
}

pub(crate) fn editor_icon_button_border(
    theme: &Theme,
    enabled: bool,
    hovered: bool,
    pressed: bool,
) -> Option<Color> {
    EditorWidgetVisuals::new(theme).icon_button_border(enabled, hovered, pressed)
}
