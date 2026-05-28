//! Immediate child-region helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_ui::element::{AnyElement, StackProps};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};

use super::{ChildRegionOptions, ChildRegionResponse, ImUiFacade};

mod resize;
mod scroll;

use resize::{child_region_resize_x_handle, child_region_resize_y_handle};

pub(super) fn child_region_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ChildRegionOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> (AnyElement, ChildRegionResponse) {
    cx.keyed(id, |cx| {
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
        let scroll = scroll::child_region_scroll_element(
            cx,
            scroll::ChildRegionScrollInput {
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
            let resize_x_handle = resize_x.map(|resize_options| {
                let handle_test_id = resize_options.handle_test_id.clone().or_else(|| {
                    root_test_id
                        .as_ref()
                        .map(|base| Arc::from(format!("{base}.resize-x")))
                });
                child_region_resize_x_handle(cx, id, resize_options, handle_test_id, &mut response)
            });
            let resize_y_handle = resize_y.map(|resize_options| {
                let handle_test_id = resize_options.handle_test_id.clone().or_else(|| {
                    root_test_id
                        .as_ref()
                        .map(|base| Arc::from(format!("{base}.resize-y")))
                });
                child_region_resize_y_handle(cx, id, resize_options, handle_test_id, &mut response)
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
        } else {
            scroll
        };

        (element, response)
    })
}
