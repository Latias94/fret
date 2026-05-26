use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, Length, PressableState};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::imui::control_chrome;

pub(super) fn menu_trigger_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    open: bool,
    enabled: bool,
    state: PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let active = open || state.hovered || state.focused || state.pressed;
    let background = if active {
        Some(
            theme
                .color_by_key("accent")
                .unwrap_or_else(|| theme.color_token("accent")),
        )
    } else {
        None
    };
    let foreground = if !enabled {
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"))
    } else if active {
        theme
            .color_by_key("accent-foreground")
            .unwrap_or_else(|| theme.color_token("accent-foreground"))
    } else {
        theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"))
    };

    let mut chrome = ContainerProps::default();
    chrome.layout.size.width = Length::Auto;
    chrome.layout.size.height = Length::Auto;
    chrome.padding = Edges {
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();
    chrome.background = background;
    chrome.corner_radii = Corners::all(control_chrome::CONTROL_RADIUS);

    cx.container(chrome, move |cx| {
        vec![
            crate::declarative::text::text_button_label(cx, label.clone())
                .inherit_foreground(foreground),
        ]
    })
}
