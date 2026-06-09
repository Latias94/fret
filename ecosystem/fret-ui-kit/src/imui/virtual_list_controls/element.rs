use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{ImUiFacade, VirtualListOptions, VirtualListResponse};
use super::range::VirtualListRenderedRangeTracker;
use super::runtime::{list_layout, resolved_measure_mode, runtime_options};
use row_item::{VirtualListRowItemInput, build_virtual_list_row_item};

mod output;
mod row_item;

pub(in crate::imui) fn virtual_list_element<H: UiHost, K, R>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    len: usize,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: VirtualListOptions,
    mut key_at: K,
    mut row: R,
) -> (AnyElement, VirtualListResponse)
where
    K: FnMut(usize) -> fret_ui::ItemKey,
    R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
{
    cx.keyed(("fret-ui-kit.imui.virtual_list", id), |cx| {
        let handle = options
            .handle
            .clone()
            .unwrap_or_else(|| cx.slot_state(VirtualListScrollHandle::new, |h| h.clone()));
        let root_test_id = options.test_id.clone();
        let row_height_fn = options.known_row_height_at.clone();
        let resolved_measure_mode = resolved_measure_mode(&options);
        let rendered_range = VirtualListRenderedRangeTracker::new();
        let rendered_range_out = rendered_range.clone();

        let list = cx.virtual_list_keyed_with_layout(
            list_layout(&options),
            len,
            runtime_options(&options, resolved_measure_mode),
            &handle,
            &mut key_at,
            move |cx, index| {
                build_virtual_list_row_item(
                    cx,
                    VirtualListRowItemInput {
                        index,
                        build_focus: build_focus.clone(),
                        rendered_range: &rendered_range,
                        root_test_id: root_test_id.as_ref(),
                        measure_mode: resolved_measure_mode,
                        estimate_row_height: options.estimate_row_height,
                        row_height_fn: row_height_fn.as_ref(),
                    },
                    &mut row,
                )
            },
        );

        let list = output::decorate_list_semantics(cx, list, options.test_id.clone());

        (
            list,
            output::virtual_list_response(handle, rendered_range_out.range()),
        )
    })
}
