use std::sync::Arc;

use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, UiHost};

use crate::imui::TableColumn;
use crate::imui::label_identity::parse_label_identity;

use super::super::cell::table_cell_padding;

mod sort;

pub(super) use sort::sortable_header_a11y_label;
pub(in crate::imui::table_controls) use sort::{column_is_sortable, table_sort_indicator_text};

pub(in crate::imui::table_controls) fn visible_header_label(
    column: &TableColumn,
) -> Option<Arc<str>> {
    column.header().map(|label| {
        let parts = parse_label_identity(label);
        Arc::<str>::from(parts.visible)
    })
}

pub(super) fn table_header_content_box<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    content: AnyElement,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    props.layout.flex.grow = 1.0;
    props.layout.flex.shrink = 1.0;
    props.padding = table_cell_padding().into();
    cx.container(props, move |_cx| vec![content])
}

pub(in crate::imui::table_controls) fn table_header_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
) -> AnyElement {
    crate::declarative::text::text_table_header_label(cx, label)
}
