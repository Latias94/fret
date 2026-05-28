use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::ImUiTableColumnVisibilityState;
use crate::imui::{MenuItemOptions, ResponseExt, TableColumn, UiWriterImUiFacadeExt};

pub(in crate::imui::table_column_visibility) fn table_column_visibility_menu_item<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
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
