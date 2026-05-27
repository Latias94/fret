use fret_core::{Corners, Edges, Px};
use fret_ui::element::{ContainerProps, PressableState};
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::CONTROL_RADIUS;
use super::palette::ImUiControlPalette;

pub(in crate::imui) fn button_chrome<H: UiHost>(
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
