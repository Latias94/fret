use std::any::Any;

use fret_ui::UiHost;

use crate::imui::{ResponseExt, UiWriterImUiFacadeExt};

use super::{SortableRowOptions, SortableRowResponse, vertical_insertion_side};

/// Attach sortable drag/drop behavior to a single immediate row.
///
/// The row still owns rendering and identity. This helper only packages:
/// - publishing the row payload,
/// - resolving compatible target preview/delivery,
/// - and deriving `Before` / `After` from the row rect midpoint.
pub fn sortable_row<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, T: Any>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
) -> SortableRowResponse<T> {
    sortable_row_with_options(ui, trigger, payload, SortableRowOptions::default())
}

pub fn sortable_row_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, T: Any>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
    options: SortableRowOptions,
) -> SortableRowResponse<T> {
    let source = ui.drag_source_with_options(trigger, payload, options.drag_source);
    let target = ui.drop_target_with_options::<T>(trigger, options.drop_target);
    let side = vertical_insertion_side(trigger, &target);

    SortableRowResponse::new(source, target, side)
}
