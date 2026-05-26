use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId};

mod layout;
mod pointer;

use super::super::FloatWindowResizeHandle;
use super::FloatingWindowResizeHandleTestIds;
use pointer::resize_handle_element;

pub(in crate::imui) fn resize_stack_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    body: AnyElement,
    blocker: Option<AnyElement>,
    resizable_layout: bool,
    collapsed: bool,
    resize_enabled: bool,
    enable_activation: bool,
    test_ids: FloatingWindowResizeHandleTestIds,
) -> AnyElement {
    if !resizable_layout || collapsed || !resize_enabled {
        if let Some(blocker) = blocker {
            return cx.stack(move |_cx| vec![body, blocker]);
        }
        return body;
    }

    let mut resize_handle = |handle: FloatWindowResizeHandle, test_id: Arc<str>| {
        resize_handle_element(cx, window_id, handle, test_id, enable_activation)
    };
    let mut stacked: Vec<AnyElement> = vec![
        body,
        resize_handle(FloatWindowResizeHandle::Left, test_ids.left),
        resize_handle(FloatWindowResizeHandle::Right, test_ids.right),
        resize_handle(FloatWindowResizeHandle::Top, test_ids.top),
        resize_handle(FloatWindowResizeHandle::Bottom, test_ids.bottom),
        resize_handle(FloatWindowResizeHandle::TopLeft, test_ids.top_left),
        resize_handle(FloatWindowResizeHandle::TopRight, test_ids.top_right),
        resize_handle(FloatWindowResizeHandle::BottomLeft, test_ids.bottom_left),
        resize_handle(FloatWindowResizeHandle::BottomRight, test_ids.bottom_right),
    ];

    if let Some(blocker) = blocker {
        stacked.push(blocker);
    }

    cx.stack(move |_cx| stacked)
}
