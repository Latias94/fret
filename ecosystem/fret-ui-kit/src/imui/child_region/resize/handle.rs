use std::sync::Arc;

use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};

use crate::imui::DragResponse;

use super::axis::ChildRegionResizeAxis;
use drag_state::populate_child_region_resize_drag_response;
use events::install_child_region_resize_handle_pointer_events;

mod drag_state;
mod events;

pub(super) fn child_region_resize_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    axis: ChildRegionResizeAxis,
    enabled: bool,
    test_id: Option<Arc<str>>,
    drag: &mut DragResponse,
) -> AnyElement {
    let handle = cx.keyed((axis.key(), id), |cx| {
        let props = PointerRegionProps {
            enabled,
            layout: axis.layout(),
            ..Default::default()
        };

        cx.pointer_region(props, move |cx| {
            let region_id = cx.root_id();
            install_child_region_resize_handle_pointer_events(
                cx,
                region_id,
                enabled,
                axis.cursor(),
            );
            populate_child_region_resize_drag_response(cx, region_id, drag);

            Vec::new()
        })
    });

    if let Some(test_id) = test_id {
        handle.test_id(test_id)
    } else {
        handle
    }
}
