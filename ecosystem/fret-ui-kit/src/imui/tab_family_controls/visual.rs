use std::sync::Arc;

use fret_core::{Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, Theme, UiHost};

pub(super) fn tab_trigger_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    selected: bool,
    enabled: bool,
    state: fret_ui::element::PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let foreground = if !enabled {
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"))
    } else if selected {
        theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"))
    } else {
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"))
    };
    let accent = theme
        .color_by_key("accent")
        .unwrap_or_else(|| theme.color_token("accent"));
    let hover_bg = if enabled && (state.hovered || state.focused || state.pressed) {
        Some(
            theme
                .color_by_key("muted")
                .unwrap_or_else(|| theme.color_token("muted")),
        )
    } else {
        None
    };

    let mut panel = ContainerProps::default();
    panel.layout.size.width = Length::Auto;
    panel.layout.size.height = Length::Auto;
    panel.padding = Edges {
        left: Px(10.0),
        right: Px(10.0),
        top: Px(6.0),
        bottom: Px(6.0),
    }
    .into();
    panel.background = hover_bg;
    panel.border = Edges {
        left: Px(0.0),
        right: Px(0.0),
        top: Px(0.0),
        bottom: Px(2.0),
    };
    panel.border_color = Some(if selected {
        accent
    } else {
        fret_core::Color::TRANSPARENT
    });

    cx.container(panel, move |cx| {
        vec![
            crate::declarative::text::text_button_label(cx, label.clone())
                .inherit_foreground(foreground),
        ]
    })
}
