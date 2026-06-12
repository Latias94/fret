//! Runtime table-column visibility helpers for IMUI table authoring.

mod menu;
mod model;
mod options;
mod response;
mod state;

pub use menu::{
    table_column_visibility_header_context_menu, table_column_visibility_menu_item,
    table_column_visibility_menu_items,
};
pub use model::table_column_visibility_use_model;
pub use options::{
    TableColumnVisibilityHeaderContextMenuOptions, TableColumnVisibilityMenuOptions,
};
pub use response::{
    TableColumnVisibilityHeaderContextMenuResponse, TableColumnVisibilityMenuItemResponse,
    TableColumnVisibilityMenuResponse,
};
pub use state::{
    ImUiTableColumnVisibilityState, TableColumnVisibilityEntry, TableColumnVisibilitySnapshot,
};

#[cfg(test)]
mod tests;
