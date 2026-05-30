use std::sync::Arc;

use fret_core::{Px, Size};

use super::super::{MenuItemOptions, PopupMenuOptions};

/// Options for composing a group of table-column visibility menu items.
#[derive(Debug, Clone, Default)]
pub struct TableColumnVisibilityMenuOptions {
    /// Base options cloned into every generated checkbox menu item.
    pub item_options: MenuItemOptions,
    /// Optional test-id prefix. When set, item test ids are `{prefix}{stable_column_id_slug}`.
    pub test_id_prefix: Option<Arc<str>>,
}

/// Options for wiring a table header context menu to table-column visibility items.
#[derive(Debug, Clone)]
pub struct TableColumnVisibilityHeaderContextMenuOptions {
    pub popup: PopupMenuOptions,
    pub menu: TableColumnVisibilityMenuOptions,
}

impl Default for TableColumnVisibilityHeaderContextMenuOptions {
    fn default() -> Self {
        Self {
            popup: PopupMenuOptions {
                estimated_size: Size::new(Px(180.0), Px(160.0)),
                ..Default::default()
            },
            menu: TableColumnVisibilityMenuOptions::default(),
        }
    }
}
