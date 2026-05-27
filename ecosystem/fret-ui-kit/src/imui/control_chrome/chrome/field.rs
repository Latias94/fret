use fret_core::{Corners, Edges, Px};
use fret_ui::element::{ContainerProps, Length, PressableState};
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::CONTROL_RADIUS;
use super::palette::ImUiControlPalette;

pub(in crate::imui) fn field_chrome<H: UiHost>(
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
