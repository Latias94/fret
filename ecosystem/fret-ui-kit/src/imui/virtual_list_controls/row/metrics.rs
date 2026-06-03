use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::VirtualListMeasureMode;

pub(in crate::imui::virtual_list_controls) fn row_height_for_index(
    index: usize,
    measure_mode: VirtualListMeasureMode,
    estimate_row_height: Px,
    known_row_height_at: Option<&Arc<dyn Fn(usize) -> Px + Send + Sync>>,
) -> Option<Px> {
    match measure_mode {
        VirtualListMeasureMode::Measured => None,
        VirtualListMeasureMode::Fixed => Some(estimate_row_height),
        VirtualListMeasureMode::Known => known_row_height_at
            .map(|f| f(index))
            .or(Some(estimate_row_height)),
    }
}

pub(in crate::imui::virtual_list_controls) fn row_test_id(
    base: Option<&Arc<str>>,
    index: usize,
) -> Option<Arc<str>> {
    base.map(|base| Arc::from(format!("{base}.row.{index}")))
}
