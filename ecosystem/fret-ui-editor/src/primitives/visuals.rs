//! Editor-grade widget visuals policy.
//!
//! This module intentionally lives in `fret-ui-editor` (ecosystem/policy layer). It provides a
//! small, reusable mapping from theme tokens + widget interaction state into consistent "chrome"
//! colors so controls don't drift.

use fret_core::Color;
use fret_ui::Theme;

use super::chrome::ResolvedEditorFrameChrome;

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

    pub(crate) fn muted_foreground(&self) -> Color {
        self.theme
            .color_by_key("muted-foreground")
            .or_else(|| self.theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| self.theme.color_token("foreground"))
    }

    pub(crate) fn hover_overlay_bg_custom(
        &self,
        base: Color,
        hovered: bool,
        pressed: bool,
        hover_mix: f32,
        press_mix: f32,
    ) -> Color {
        let accent = self.theme.color_token("accent");
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
        let accent = self.theme.color_token("accent");
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

        Some(self.hover_overlay_bg(self.theme.color_token("background"), hovered, pressed))
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

        let base = self
            .theme
            .color_by_key("border")
            .or_else(|| self.theme.color_by_key("component.input.border"))
            .unwrap_or_else(|| self.theme.color_token("foreground"));

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
        // Keep disabled visuals conservative: we only scale alpha and avoid color shifts that can
        // reduce contrast too much on dark themes.
        let disabled_alpha = if state.enabled { 1.0 } else { 0.55 };

        let icon = alpha_mul(self.muted_foreground(), disabled_alpha);

        let accent = self.theme.color_token("accent");
        let mut bg = alpha_mul(chrome.bg, disabled_alpha);
        let mut border = alpha_mul(chrome.border, disabled_alpha);
        let fg = alpha_mul(chrome.fg, disabled_alpha);

        if state.hovered && state.enabled {
            bg = mix(bg, accent, 0.08);
            border = mix(border, accent, 0.10);
        }
        if state.pressed && state.enabled {
            bg = mix(bg, accent, 0.14);
            border = mix(border, accent, 0.16);
        }
        if (state.focused || state.open) && state.enabled {
            border = chrome.border_focus;
        }

        EditorFrameVisuals {
            bg,
            border,
            fg,
            icon,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EditorFrameState {
    pub(crate) enabled: bool,
    pub(crate) hovered: bool,
    pub(crate) pressed: bool,
    pub(crate) focused: bool,
    pub(crate) open: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EditorFrameVisuals {
    pub(crate) bg: Color,
    pub(crate) border: Color,
    pub(crate) fg: Color,
    pub(crate) icon: Color,
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

pub(crate) fn hover_overlay_bg(theme: &Theme, base: Color, hovered: bool, pressed: bool) -> Color {
    EditorWidgetVisuals::new(theme).hover_overlay_bg(base, hovered, pressed)
}

pub(crate) fn hover_overlay_border(
    theme: &Theme,
    base: Color,
    hovered: bool,
    pressed: bool,
) -> Color {
    EditorWidgetVisuals::new(theme).hover_overlay_border(base, hovered, pressed)
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
