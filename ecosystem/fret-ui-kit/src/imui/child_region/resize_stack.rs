use std::sync::Arc;

use fret_ui::element::{AnyElement, StackProps};
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::{ChildRegionResizeXOptions, ChildRegionResizeYOptions, ChildRegionResponse};
use super::resize::{child_region_resize_x_handle, child_region_resize_y_handle};

pub(super) struct ChildRegionResizeStackInput<'a> {
    pub(super) id: &'a str,
    pub(super) scroll: AnyElement,
    pub(super) layout: crate::LayoutRefinement,
    pub(super) root_test_id: Option<Arc<str>>,
    pub(super) resize_x: Option<ChildRegionResizeXOptions>,
    pub(super) resize_y: Option<ChildRegionResizeYOptions>,
    pub(super) response: &'a mut ChildRegionResponse,
}

pub(super) fn child_region_resize_stack_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: ChildRegionResizeStackInput<'_>,
) -> AnyElement {
    let ChildRegionResizeStackInput {
        id,
        scroll,
        layout,
        root_test_id,
        resize_x,
        resize_y,
        response,
    } = input;

    let resize_x_handle = resize_x.map(|resize_options| {
        let handle_test_id = resize_options.handle_test_id.clone().or_else(|| {
            root_test_id
                .as_ref()
                .map(|base| Arc::from(format!("{base}.resize-x")))
        });
        child_region_resize_x_handle(cx, id, resize_options, handle_test_id, response)
    });
    let resize_y_handle = resize_y.map(|resize_options| {
        let handle_test_id = resize_options.handle_test_id.clone().or_else(|| {
            root_test_id
                .as_ref()
                .map(|base| Arc::from(format!("{base}.resize-y")))
        });
        child_region_resize_y_handle(cx, id, resize_options, handle_test_id, response)
    });

    let mut stack = StackProps::default();
    stack.layout = crate::declarative::style::layout_style(Theme::global(&*cx.app), layout);

    let stack = cx.stack_props(stack, move |_cx| {
        let mut children = vec![scroll];
        if let Some(handle) = resize_x_handle {
            children.push(handle);
        }
        if let Some(handle) = resize_y_handle {
            children.push(handle);
        }
        children
    });
    if let Some(test_id) = root_test_id {
        stack.test_id(test_id)
    } else {
        stack
    }
}
