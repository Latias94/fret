use std::sync::Arc;

use fret_core::Px;

use super::identity::inferred_column_id;
use super::{TableColumn, TableColumnPin, TableColumnWidth};

impl TableColumn {
    pub fn px(header: impl Into<Arc<str>>, width: Px) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Px(width),
            visible: true,
            sortable: false,
            sort_direction: None,
            resize: None,
            pin: TableColumnPin::None,
        }
    }

    pub fn fill(header: impl Into<Arc<str>>) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Fill(1.0),
            visible: true,
            sortable: false,
            sort_direction: None,
            resize: None,
            pin: TableColumnPin::None,
        }
    }

    pub fn weighted(header: impl Into<Arc<str>>, weight: f32) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Fill(weight),
            visible: true,
            sortable: false,
            sort_direction: None,
            resize: None,
            pin: TableColumnPin::None,
        }
    }

    pub fn unlabeled(width: TableColumnWidth) -> Self {
        Self {
            header: None,
            id: None,
            width,
            visible: true,
            sortable: false,
            sort_direction: None,
            resize: None,
            pin: TableColumnPin::None,
        }
    }
}
