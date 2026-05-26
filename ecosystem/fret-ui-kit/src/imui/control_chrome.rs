//! Shared visual chrome helpers for immediate controls.

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{ContainerProps, Length, PressableState};
use fret_ui::{ElementContext, Theme, UiHost};

// Dear ImGui's default style is compact and mostly square.
// Keep Fret IMUI slightly roomier than upstream to preserve a usable hit target, but use the
// same overall density direction instead of the old shadcn-form defaults.
pub(super) const CONTROL_RADIUS: Px = Px(2.0);
pub(super) const PANEL_RADIUS: Px = Px(4.0);
pub(super) const BUTTON_MIN_HEIGHT: Px = Px(24.0);
pub(super) const SMALL_BUTTON_MIN_HEIGHT: Px = Px(20.0);
pub(super) const FIELD_MIN_HEIGHT: Px = Px(24.0);
pub(super) const ARROW_BUTTON_SIZE: Px = Px(20.0);
pub(super) const RADIO_INDICATOR_SIZE: Px = Px(14.0);
pub(super) const RADIO_DOT_SIZE: Px = Px(6.0);
pub(super) const STACK_GAP: Px = Px(4.0);
pub(super) const ROW_GAP: Px = Px(8.0);
pub(super) const SLIDER_TRACK_HEIGHT: Px = Px(4.0);

mod layout;
mod text;

pub(super) use layout::{centered_row_props, fill_row_props, fill_stack_props};
pub(super) use text::{caption_text, control_text, fill_text, pill};

#[derive(Debug, Clone, Copy)]
pub(super) struct ImUiControlPalette {
    pub background: Color,
    pub border: Color,
    pub foreground: Color,
    pub muted_foreground: Color,
    pub accent_background: Color,
    pub accent_foreground: Color,
    pub subtle_background: Color,
}

pub(super) fn button_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    state: PressableState,
) -> (ImUiControlPalette, ContainerProps) {
    let theme = Theme::global(&*cx.app);
    let muted = theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_token("muted"));
    let muted_fg = theme
        .color_by_key("muted-foreground")
        .unwrap_or_else(|| theme.color_token("muted-foreground"));
    let secondary = theme
        .color_by_key("secondary")
        .unwrap_or_else(|| theme.color_token("secondary"));
    let secondary_fg = theme
        .color_by_key("secondary-foreground")
        .or_else(|| theme.color_by_key("foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"));
    let accent = theme
        .color_by_key("accent")
        .unwrap_or_else(|| theme.color_token("accent"));
    let accent_fg = theme
        .color_by_key("accent-foreground")
        .or_else(|| theme.color_by_key("foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"));
    let ring = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("ring"));
    let border_idle = theme
        .color_by_key("border")
        .unwrap_or_else(|| theme.color_token("border"));

    let palette = ImUiControlPalette {
        background: if !enabled {
            muted
        } else if state.pressed {
            accent
        } else if state.hovered || state.focused {
            muted
        } else {
            secondary
        },
        border: if state.hovered || state.focused {
            ring
        } else {
            border_idle
        },
        foreground: if !enabled {
            muted_fg
        } else if state.pressed || state.hovered || state.focused {
            accent_fg
        } else {
            secondary_fg
        },
        muted_foreground: muted_fg,
        accent_background: accent,
        accent_foreground: accent_fg,
        subtle_background: secondary,
    };

    let mut chrome = ContainerProps::default();
    chrome.padding = Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
    .into();
    chrome.background = Some(palette.background);
    chrome.border = Edges::all(Px(1.0));
    chrome.border_color = Some(palette.border);
    chrome.corner_radii = Corners::all(CONTROL_RADIUS);

    (palette, chrome)
}

pub(super) fn field_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    state: PressableState,
) -> (ImUiControlPalette, ContainerProps) {
    let theme = Theme::global(&*cx.app);
    let background = theme
        .color_by_key("card")
        .or_else(|| theme.color_by_key("muted"))
        .or_else(|| theme.color_by_key("background"))
        .unwrap_or_else(|| theme.color_token("background"));
    let muted = theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_token("muted"));
    let muted_fg = theme
        .color_by_key("muted-foreground")
        .unwrap_or_else(|| theme.color_token("muted-foreground"));
    let foreground = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));
    let border_idle = theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("input"));
    let ring = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("ring"));
    let primary = theme
        .color_by_key("primary")
        .unwrap_or_else(|| theme.color_token("primary"));
    let primary_fg = theme
        .color_by_key("primary-foreground")
        .unwrap_or_else(|| theme.color_token("primary-foreground"));

    let palette = ImUiControlPalette {
        background: if !enabled || state.pressed || state.hovered {
            muted
        } else {
            background
        },
        border: if state.hovered || state.focused {
            ring
        } else {
            border_idle
        },
        foreground: if enabled { foreground } else { muted_fg },
        muted_foreground: muted_fg,
        accent_background: primary,
        accent_foreground: primary_fg,
        subtle_background: muted,
    };

    let mut chrome = ContainerProps::default();
    chrome.layout.size.width = Length::Fill;
    chrome.padding = Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(3.0),
        bottom: Px(3.0),
    }
    .into();
    chrome.background = Some(palette.background);
    chrome.border = Edges::all(Px(1.0));
    chrome.border_color = Some(palette.border);
    chrome.corner_radii = Corners::all(CONTROL_RADIUS);

    (palette, chrome)
}

#[cfg(test)]
mod tests;
