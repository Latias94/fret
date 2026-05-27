use std::sync::Arc;

mod construction;
mod identity;
mod pinning;
mod primitives;
mod resize;
mod sorting;
mod visibility;

pub use primitives::{
    TableColumnPin, TableColumnResizeOptions, TableColumnWidth, TableSortDirection,
};

#[derive(Debug, Clone)]
pub struct TableColumn {
    header: Option<Arc<str>>,
    id: Option<Arc<str>>,
    width: TableColumnWidth,
    visible: bool,
    sortable: bool,
    sort_direction: Option<TableSortDirection>,
    resize: Option<TableColumnResizeOptions>,
    pin: TableColumnPin,
}
