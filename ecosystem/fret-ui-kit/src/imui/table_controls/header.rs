use fret_ui::element::AnyElement;

use crate::imui::ResponseExt;

mod cell;
mod labels;
mod plain;
mod resize;
mod sortable;
mod trigger;

pub(super) use labels::{
    column_is_sortable, table_header_label_text, table_sort_indicator_text, visible_header_label,
};
pub(super) use plain::wrap_plain_header_cell;
pub(super) use sortable::wrap_sortable_header_cell;

pub(super) struct BuiltHeaderCell {
    pub(super) element: AnyElement,
    pub(super) trigger: ResponseExt,
}
