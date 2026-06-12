use fret_runtime::Model;
use fret_ui::UiHost;

use super::{
    ImUiTableColumnVisibilityState, TableColumnVisibilityHeaderContextMenuOptions,
    TableColumnVisibilityHeaderContextMenuResponse, TableColumnVisibilityMenuResponse,
};
use crate::imui::{TableColumn, TableResponse, UiWriterImUiFacadeExt};

mod identity;
mod item;
mod items;

pub(super) use identity::{menu_column_id, menu_test_id_suffix, visible_menu_label};
pub use item::table_column_visibility_menu_item;
pub use items::table_column_visibility_menu_items;

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
    let mut trigger = None;
    let mut fallback_trigger = None;
    for header in table.headers() {
        let response = header.response();
        if fallback_trigger.is_none() && response.id().is_some() {
            fallback_trigger = Some(response);
        }
        if response.context_menu_requested() {
            trigger = Some(response);
            break;
        }
    }
    let trigger = trigger.or(fallback_trigger).unwrap_or_default();

    let mut items = TableColumnVisibilityMenuResponse::default();
    let open = ui.begin_popup_context_menu_with_options(id, trigger, options.popup, |ui| {
        items = table_column_visibility_menu_items(ui, columns, model, options.menu);
    });

    TableColumnVisibilityHeaderContextMenuResponse { open, items }
}
