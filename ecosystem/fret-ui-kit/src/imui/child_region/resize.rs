use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::imui::{ChildRegionResizeXOptions, ChildRegionResizeYOptions, ChildRegionResponse};

mod axis;
mod handle;

use axis::ChildRegionResizeAxis;
use handle::child_region_resize_handle;

pub(super) fn child_region_resize_x_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    options: ChildRegionResizeXOptions,
    test_id: Option<Arc<str>>,
    response: &mut ChildRegionResponse,
) -> AnyElement {
    let enabled = !super::super::imui_is_disabled(cx);
    let resize_response = response.resize_x_mut();
    resize_response.enabled = enabled;
    resize_response.min_width = options.min_width;
    resize_response.max_width = options.max_width;

    child_region_resize_handle(
        cx,
        id,
        ChildRegionResizeAxis::X,
        enabled,
        test_id,
        &mut resize_response.drag,
    )
}

pub(super) fn child_region_resize_y_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    options: ChildRegionResizeYOptions,
    test_id: Option<Arc<str>>,
    response: &mut ChildRegionResponse,
) -> AnyElement {
    let enabled = !super::super::imui_is_disabled(cx);
    let resize_response = response.resize_y_mut();
    resize_response.enabled = enabled;
    resize_response.min_height = options.min_height;
    resize_response.max_height = options.max_height;

    child_region_resize_handle(
        cx,
        id,
        ChildRegionResizeAxis::Y,
        enabled,
        test_id,
        &mut resize_response.drag,
    )
}
