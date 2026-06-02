use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, GlobalElementId};

use super::super::super::{
    FloatWindowResizeHandle, float_layer_bring_to_front_if_activated,
    float_window_resize_kind_for_element,
};
use super::cursor::resize_handle_cursor;
use super::layout::resize_handle_layout;
use events::{ResizeHandlePointerInput, install_resize_handle_pointer_events};

mod events;

pub(super) fn resize_handle_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    handle: FloatWindowResizeHandle,
    test_id: Arc<str>,
    enable_activation: bool,
) -> AnyElement {
    let cursor = resize_handle_cursor(handle);
    let layout = resize_handle_layout(handle);

    let kind = float_window_resize_kind_for_element(window_id, handle);
    cx.pointer_region(
        PointerRegionProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let _region_id = cx.root_id();
            float_layer_bring_to_front_if_activated(cx, window_id);

            install_resize_handle_pointer_events(
                cx,
                ResizeHandlePointerInput {
                    window_id,
                    kind,
                    cursor,
                    enable_activation,
                },
            );

            Vec::new()
        },
    )
    .test_id(test_id)
}
