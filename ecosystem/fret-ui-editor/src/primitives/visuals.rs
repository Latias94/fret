//! Editor-grade widget visuals policy.
//!
//! This module intentionally lives in `fret-ui-editor` (ecosystem/policy layer). It provides a
//! small, reusable mapping from theme tokens + widget interaction state into consistent "chrome"
//! colors so controls don't drift.

use fret_core::Color;
use fret_ui::Theme;

use super::{
    EditorTokenKeys,
    chrome::ResolvedEditorFrameChrome,
    colors::{
        editor_accent, editor_border, editor_invalid_border, editor_invalid_foreground,
        editor_muted_foreground,
    },
};

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
        editor_muted_foreground(self.theme)
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
        // Keep disabled visuals conservative: we only scale alpha and avoid color shifts that can
        // reduce contrast too much on dark themes.
        let disabled_alpha = if state.enabled { 1.0 } else { 0.55 };

        let accent = editor_accent(self.theme);
        let mut bg = alpha_mul(chrome.bg, disabled_alpha);
        let mut border = alpha_mul(chrome.border, disabled_alpha);
        let fg = alpha_mul(chrome.fg, disabled_alpha);
        let mut icon = alpha_mul(self.muted_foreground(), disabled_alpha);

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
            let invalid_fg = self.control_invalid_fg();
            let invalid_border = self.control_invalid_border();
            let invalid_bg = self.control_invalid_bg(chrome.bg, invalid_border);

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
        let disabled_alpha = if state.enabled { 1.0 } else { 0.55 };

        let accent = editor_accent(self.theme);
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

impl<'a> EditorWidgetVisuals<'a> {
    fn control_invalid_fg(&self) -> Color {
        editor_invalid_foreground(self.theme)
    }

    fn control_invalid_border(&self) -> Color {
        editor_invalid_border(self.theme)
    }

    fn control_invalid_bg(&self, base: Color, border: Color) -> Color {
        self.theme
            .color_by_key(EditorTokenKeys::CONTROL_INVALID_BG)
            .or_else(|| self.theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_BG))
            .unwrap_or_else(|| {
                let mut out = mix(base, Color { a: 1.0, ..border }, 0.10);
                out.a = 1.0;
                out
            })
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

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{Color, Edges, Px};
    use fret_ui::Theme;

    use super::*;

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
}
