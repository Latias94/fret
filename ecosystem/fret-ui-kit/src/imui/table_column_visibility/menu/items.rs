use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::{
    ImUiTableColumnVisibilityState, TableColumnVisibilityMenuItemResponse,
    TableColumnVisibilityMenuOptions, TableColumnVisibilityMenuResponse,
};
use super::{
    menu_column_id, menu_test_id_suffix, table_column_visibility_menu_item, visible_menu_label,
};
use crate::imui::{TableColumn, UiWriterImUiFacadeExt};

pub fn table_column_visibility_menu_items<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    columns: &[TableColumn],
    model: &Model<ImUiTableColumnVisibilityState>,
    options: TableColumnVisibilityMenuOptions,
) -> TableColumnVisibilityMenuResponse {
    let mut items = Vec::new();

    for (index, column) in columns.iter().enumerate() {
        let Some(column_id) = menu_column_id(column) else {
            continue;
        };
        if visible_menu_label(column).is_none() {
            continue;
        }

        let mut item_options = options.item_options.clone();
        if let Some(prefix) = options.test_id_prefix.as_ref() {
            item_options.test_id = Some(Arc::from(format!(
                "{}{}",
                prefix,
                menu_test_id_suffix(column_id.as_ref(), index)
            )));
        }

        let Some(response) = table_column_visibility_menu_item(ui, column, model, item_options)
        else {
            continue;
        };
        let visible = ui.with_cx_mut(|cx| {
            cx.read_model(model, fret_ui::Invalidation::Paint, |_app, state| {
                state.is_visible(column_id.as_ref(), column.visible())
            })
            .unwrap_or(column.visible())
        });
        items.push(TableColumnVisibilityMenuItemResponse::new(
            column_id, visible, response,
        ));
    }

    TableColumnVisibilityMenuResponse { items }
}
