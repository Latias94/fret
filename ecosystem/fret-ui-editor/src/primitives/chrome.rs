//! Shared chrome resolution helpers for editor controls.
//!
//! v1 goal: keep "frame" defaults consistent across controls (inputs, triggers, scrub surfaces)
//! without hard-binding `fret-ui-editor` to a specific design system crate.

use fret_core::{Color, Corners, Edges, Px, TextStyle};
use fret_ui::element::{RingPlacement, RingStyle};
use fret_ui::{TextAreaStyle, TextInputStyle, Theme};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, Size};

use super::colors::{editor_focus_ring, editor_muted_foreground};

mod input;
mod surface;

use input::resolve_editor_text_field_input_chrome;
pub(crate) use surface::sanitize_editor_surface_bg;

#[cfg(test)]
mod tests;

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

impl ResolvedEditorFrameChrome {
    pub(crate) fn control_outer_height(self, line_height: Px) -> Px {
        Px(line_height.0 + self.padding.top.0 + self.padding.bottom.0 + self.border_width.0 * 2.0)
    }
}

pub(crate) fn resolve_editor_text_field_frame_chrome(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> ResolvedEditorFrameChrome {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);
    ResolvedEditorFrameChrome {
        padding: resolved.padding,
        radius: resolved.radius,
        border_width: resolved.border_width,
        bg: sanitize_editor_surface_bg(theme, resolved.background),
        border: resolved.border_color,
        border_focus: resolved.border_color_focused,
        fg: resolved.text_color,
        text_px: resolved.text_px,
    }
}

pub(crate) fn joined_text_input_style(mut chrome: TextInputStyle) -> TextInputStyle {
    chrome.padding = Edges::all(Px(0.0));
    chrome.border = Edges::all(Px(0.0));
    chrome.corner_radii = Corners::all(Px(0.0));
    chrome.background = Color {
        a: 0.0,
        ..chrome.background
    };
    chrome.border_color = Color {
        a: 0.0,
        ..chrome.border_color
    };
    chrome.border_color_focused = chrome.border_color;
    chrome.focus_ring = None;
    chrome
}

pub(crate) fn joined_text_area_style(mut chrome: TextAreaStyle) -> TextAreaStyle {
    chrome.padding_x = Px(0.0);
    chrome.padding_y = Px(0.0);
    chrome.border = Edges::all(Px(0.0));
    chrome.corner_radii = Corners::all(Px(0.0));
    chrome.background = Color {
        a: 0.0,
        ..chrome.background
    };
    chrome.border_color = Color {
        a: 0.0,
        ..chrome.border_color
    };
    chrome.focus_ring = None;
    chrome
}

pub(crate) fn resolve_editor_text_field_style(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> (TextInputStyle, TextStyle) {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);

    let mut chrome = TextInputStyle::from_theme(theme.snapshot());
    chrome.padding = resolved.padding;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.border = Edges::all(resolved.border_width);
    chrome.background = sanitize_editor_surface_bg(theme, resolved.background);
    chrome.border_color = resolved.border_color;
    chrome.border_color_focused = resolved.border_color_focused;
    chrome.text_color = resolved.text_color;
    chrome.caret_color = resolved.text_color;
    chrome.selection_color = resolved.selection_color;

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let text_style = typography::as_control_text(TextStyle {
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    });

    (chrome, text_style)
}

pub(crate) fn resolve_editor_text_area_field_style(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> (TextAreaStyle, TextStyle) {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);
    let ring_color = editor_focus_ring(theme);

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let text_style = typography::as_content_text(TextStyle {
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    });

    let chrome = TextAreaStyle {
        padding_x: resolved.padding.left,
        padding_y: resolved.padding.top,
        background: sanitize_editor_surface_bg(theme, resolved.background),
        border: Edges::all(resolved.border_width),
        border_color: resolved.border_color,
        border_color_focused: resolved.border_color_focused,
        focus_ring: Some(RingStyle {
            placement: RingPlacement::Outset,
            width: Px(2.0),
            offset: Px(2.0),
            color: ring_color,
            offset_color: None,
            corner_radii: Corners::all(resolved.radius),
        }),
        corner_radii: Corners::all(resolved.radius),
        text_color: resolved.text_color,
        placeholder_color: editor_muted_foreground(theme),
        selection_color: resolved.selection_color,
        caret_color: resolved.text_color,
        preedit_bg_color: Color {
            a: 0.22,
            ..resolved.selection_color
        },
        preedit_underline_color: editor_focus_ring(theme),
    };

    (chrome, text_style)
}
