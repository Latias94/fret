use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::{
    ImUiTableColumnVisibilityState, TableColumnVisibilityHeaderContextMenuOptions,
    TableColumnVisibilityHeaderContextMenuResponse, TableColumnVisibilityMenuItemResponse,
    TableColumnVisibilityMenuOptions, TableColumnVisibilityMenuResponse,
};
use crate::imui::label_identity::parse_label_identity;
use crate::imui::{
    MenuItemOptions, ResponseExt, TableColumn, TableResponse, UiWriterImUiFacadeExt,
};

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

pub(super) fn table_column_visibility_menu_item<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    column: &TableColumn,
    model: &Model<ImUiTableColumnVisibilityState>,
    options: MenuItemOptions,
) -> Option<ResponseExt> {
    let id = column.id_arc()?;
    if id.is_empty() {
        return None;
    }

    let label = column
        .header_arc()
        .unwrap_or_else(|| Arc::from(id.as_ref()));
    let visible = ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, state| {
            state.is_visible(id.as_ref(), column.visible())
        })
        .unwrap_or(column.visible())
    });

    let mut response = ui.menu_item_checkbox_with_options(label, visible, options);
    if response.clicked() {
        let changed_to = !visible;
        let mut changed = false;
        let _ = ui.with_cx_mut(|cx| {
            cx.app.models_mut().update(model, |state| {
                if state.is_visible(id.as_ref(), column.visible()) != changed_to {
                    state.set_visible(id.clone(), changed_to);
                    changed = true;
                }
            })
        });
        response.set_core_changed(changed);
        response.merge_edited(changed);
    }

    Some(response)
}

pub(super) fn menu_column_id(column: &TableColumn) -> Option<Arc<str>> {
    let id = column.id_arc()?;
    (!id.is_empty()).then_some(id)
}

pub(super) fn visible_menu_label(column: &TableColumn) -> Option<&str> {
    let header = column.header()?;
    let parts = parse_label_identity(header);
    (!parts.visible.is_empty()).then_some(parts.visible)
}

pub(super) fn menu_test_id_suffix(id: &str, index: usize) -> String {
    let mut out = String::new();
    let mut last_was_separator = false;

    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !out.is_empty() && !last_was_separator {
            out.push('-');
            last_was_separator = true;
        }
    }

    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        index.to_string()
    } else {
        out
    }
}
