use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, Theme, UiHost};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SelectablePalette {
    pub(super) bg: Option<Color>,
    pub(super) fg: Color,
}

pub(super) fn selectable_row_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    enabled: bool,
    selected: bool,
    highlighted: bool,
    state: fret_ui::element::PressableState,
) -> AnyElement {
    let palette = resolve_selectable_palette(
        Theme::global(&*cx.app),
        enabled,
        selected,
        highlighted || state.hovered || state.focused,
        state.pressed,
    );

    let mut row = ContainerProps::default();
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Auto;
    row.padding = Edges {
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();
    row.background = palette.bg;
    row.corner_radii = Corners::all(super::super::control_chrome::CONTROL_RADIUS);

    cx.container(row, move |cx| {
        vec![
            crate::declarative::text::text_list_row_label(cx, label.clone())
                .inherit_foreground(palette.fg),
        ]
    })
}

pub(super) fn resolve_selectable_palette(
    theme: &Theme,
    enabled: bool,
    selected: bool,
    hovered: bool,
    pressed: bool,
) -> SelectablePalette {
    let hovered_or_pressed = enabled && (hovered || pressed);
    let selected_bg = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("list.row.selected"))
        .or_else(|| theme.color_by_key("selection.background"))
        .unwrap_or_else(|| theme.color_token("selection.background"));
    let hover_bg = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let fg = if !enabled {
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"))
    } else if !selected && hovered_or_pressed {
        theme
            .color_by_key("accent-foreground")
            .unwrap_or_else(|| theme.color_token("accent-foreground"))
    } else {
        theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"))
    };

    let bg = if selected {
        Some(selected_bg)
    } else if hovered_or_pressed {
        Some(hover_bg)
    } else {
        None
    };

    SelectablePalette { bg, fg }
}
