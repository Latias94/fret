//! Runtime table-column visibility helpers for IMUI table authoring.

use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::{MenuItemOptions, ResponseExt, TableColumn, TableResponse, UiWriterImUiFacadeExt};

mod menu;
mod model;
mod options;
mod response;
mod state;

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
    model::table_column_visibility_use_model(cx, controlled, default_value)
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
