use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::AnyElement;
use fret_ui::element::VirtualListMeasureMode;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::ImUiFacade;
use super::super::super::containers::build_imui_children_with_focus;
use super::super::range::VirtualListRenderedRangeTracker;
use super::super::row::{pack_row_children, row_height_for_index, row_test_id, wrap_row};

pub(super) struct VirtualListRowItemInput<'a> {
    pub(super) index: usize,
    pub(super) build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    pub(super) rendered_range: &'a VirtualListRenderedRangeTracker,
    pub(super) root_test_id: Option<&'a Arc<str>>,
    pub(super) measure_mode: VirtualListMeasureMode,
    pub(super) estimate_row_height: Px,
    pub(super) row_height_fn: Option<&'a Arc<dyn Fn(usize) -> Px + Send + Sync>>,
}

pub(super) fn build_virtual_list_row_item<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    input: VirtualListRowItemInput<'_>,
    row: &mut R,
) -> AnyElement
where
    R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
{
    input.rendered_range.record(input.index);

    let mut out = Vec::new();
    build_imui_children_with_focus(cx, &mut out, input.build_focus, |ui| {
        row(ui, input.index);
    });
    let content = pack_row_children(cx, out);
    wrap_row(
        cx,
        input.index,
        content,
        row_test_id(input.root_test_id, input.index),
        row_height_for_index(
            input.index,
            input.measure_mode,
            input.estimate_row_height,
            input.row_height_fn,
        ),
    )
}
