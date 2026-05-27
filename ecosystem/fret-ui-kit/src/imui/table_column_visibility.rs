//! Runtime table-column visibility helpers for IMUI table authoring.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::{
    MenuItemOptions, PopupMenuOptions, ResponseExt, TableColumn, TableResponse,
    UiWriterImUiFacadeExt,
};

mod menu;
mod response;
mod state;

pub use response::{
    TableColumnVisibilityHeaderContextMenuResponse, TableColumnVisibilityMenuItemResponse,
    TableColumnVisibilityMenuResponse,
};
pub use state::{
    ImUiTableColumnVisibilityState, TableColumnVisibilityEntry, TableColumnVisibilitySnapshot,
};

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
                estimated_size: fret_core::Size::new(fret_core::Px(180.0), fret_core::Px(160.0)),
                ..Default::default()
            },
            menu: TableColumnVisibilityMenuOptions::default(),
        }
    }
}

/// Opens and renders a table-column visibility context menu from table header context requests.
///
/// This is a kit-layer composition policy: callers still own the visibility model and decide when
/// to apply it to their column list, while the helper wires table header context-menu requests to
/// the existing popup/menu-item policy.
pub fn table_column_visibility_header_context_menu<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    table: &TableResponse,
    columns: &[TableColumn],
    model: &Model<ImUiTableColumnVisibilityState>,
    options: TableColumnVisibilityHeaderContextMenuOptions,
) -> TableColumnVisibilityHeaderContextMenuResponse {
    menu::table_column_visibility_header_context_menu(ui, id, table, columns, model, options)
}

/// Returns a controllable visibility model for an immediate table column set.
pub fn table_column_visibility_use_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<ImUiTableColumnVisibilityState>>,
    default_value: impl FnOnce() -> ImUiTableColumnVisibilityState,
) -> crate::primitives::controllable_state::ControllableModel<ImUiTableColumnVisibilityState> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

pub fn table_column_visibility_menu_items<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    columns: &[TableColumn],
    model: &Model<ImUiTableColumnVisibilityState>,
    options: TableColumnVisibilityMenuOptions,
) -> TableColumnVisibilityMenuResponse {
    menu::table_column_visibility_menu_items(ui, columns, model, options)
}

pub fn table_column_visibility_menu_item<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    column: &TableColumn,
    model: &Model<ImUiTableColumnVisibilityState>,
    options: MenuItemOptions,
) -> Option<ResponseExt> {
    menu::table_column_visibility_menu_item(ui, column, model, options)
}

#[cfg(test)]
mod tests;
