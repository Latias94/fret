use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{ChildRegionOptions, ChildRegionResponse, ImUiFacade};

pub(in crate::imui::child_region) fn child_region_keyed_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ChildRegionOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> (AnyElement, ChildRegionResponse) {
    let chrome = options.chrome;
    let layout = options.layout.clone();
    let resize_x = options.resize_x.clone();
    let resize_y = options.resize_y.clone();
    let has_resize = resize_x.is_some() || resize_y.is_some();
    let scroll_layout = if has_resize {
        crate::LayoutRefinement::default().size_full()
    } else {
        layout.clone()
    };
    let scroll_options = options.scroll.clone();
    let test_id = options.test_id.clone();
    let root_test_id = test_id.clone();
    let scroll_root_test_id = if has_resize { None } else { test_id };
    let scroll = super::scroll::child_region_scroll_element(
        cx,
        super::scroll::ChildRegionScrollInput {
            build_focus,
            build: f,
            chrome,
            scroll_layout,
            scroll_options,
            root_test_id: scroll_root_test_id,
            content_test_id: options.content_test_id.clone(),
        },
    );
    let mut response = ChildRegionResponse::empty();

    let element = if has_resize {
        super::resize_stack::child_region_resize_stack_element(
            cx,
            super::resize_stack::ChildRegionResizeStackInput {
                id,
                scroll,
                layout,
                root_test_id,
                resize_x,
                resize_y,
                response: &mut response,
            },
        )
    } else {
        scroll
    };

    (element, response)
}
