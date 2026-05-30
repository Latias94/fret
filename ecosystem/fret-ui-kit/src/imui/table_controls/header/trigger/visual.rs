use std::sync::Arc;

use fret_ui::element::{AnyElement, ContainerProps, Length, PressableState};
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::{table_header_label_text, table_sort_indicator_text};
use crate::imui::TableSortDirection;
use crate::imui::table_controls::cell::table_cell_padding;

pub(in crate::imui::table_controls::header) fn sortable_header_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    visible_label: Option<Arc<str>>,
    sort_direction: Option<TableSortDirection>,
    enabled: bool,
    state: PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let hover_bg = if enabled && (state.hovered || state.focused || state.pressed) {
        Some(
            theme
                .color_by_key("muted")
                .unwrap_or_else(|| theme.color_token("muted")),
        )
    } else {
        None
    };
    let mut cell = ContainerProps::default();
    cell.layout.size.width = Length::Fill;
    cell.layout.size.height = Length::Auto;
    cell.padding = table_cell_padding().into();
    cell.background = hover_bg;

    cx.container(cell, move |cx| {
        let mut children = Vec::new();
        if let Some(label) = visible_label.clone() {
            children.push(table_header_label_text(cx, label));
        }
        if let Some(direction) = sort_direction {
            children.push(table_sort_indicator_text(cx, direction));
        }
        if children.is_empty() {
            Vec::new()
        } else if children.len() == 1 {
            children
        } else {
            vec![
                crate::ui::h_flex(move |_cx| children)
                    .gap_metric(crate::MetricRef::space(crate::Space::N1))
                    .justify(crate::Justify::Start)
                    .items(crate::Items::Center)
                    .no_wrap()
                    .into_element(cx),
            ]
        }
    })
}
