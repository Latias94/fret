use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::{
    ImUiTableColumnVisibilityState, TableColumnVisibilityHeaderContextMenuOptions,
    TableColumnVisibilityHeaderContextMenuResponse, TableColumnVisibilityMenuItemResponse,
    TableColumnVisibilityMenuOptions, TableColumnVisibilityMenuResponse,
};
use crate::imui::{TableColumn, TableResponse, UiWriterImUiFacadeExt};

mod identity;
mod item;

pub(super) use identity::{menu_column_id, menu_test_id_suffix, visible_menu_label};
pub(super) use item::table_column_visibility_menu_item;

pub(super) fn table_column_visibility_header_context_menu<
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

pub(super) fn table_column_visibility_menu_items<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
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
        items.push(TableColumnVisibilityMenuItemResponse {
            column_id,
            visible,
            response,
        });
    }

    TableColumnVisibilityMenuResponse { items }
}
