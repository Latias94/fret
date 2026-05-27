//! Immediate child-region helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_ui::element::{AnyElement, StackProps};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};

use super::{
    ChildRegionChrome, ChildRegionOptions, ChildRegionResponse, ImUiFacade,
    containers::build_imui_children_with_focus,
};

mod resize;

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
        let scroll_layout = if resize_x.is_some() || resize_y.is_some() {
            crate::LayoutRefinement::default().size_full()
        } else {
            layout.clone()
        };
        let scroll_options = options.scroll.clone();
        let test_id = options.test_id.clone();
        let root_test_id = test_id.clone();
        let content_test_id = options.content_test_id.clone();
        let viewport_test_id = scroll_options.viewport_test_id.clone();

        let mut builder = crate::ui::scroll_area_build(move |cx, out| {
            let mut content = crate::ui::v_flex_build(move |cx, out| {
                build_imui_children_with_focus(cx, out, build_focus, f);
            })
            .no_wrap();

            if let Some(test_id) = content_test_id.clone() {
                content = content.test_id(test_id);
            }

            out.push(content.into_element(cx));
        })
        .axis(scroll_options.axis)
        .show_scrollbars(
            scroll_options.show_scrollbar_x,
            scroll_options.show_scrollbar_y,
        )
        .layout(scroll_layout);

        if chrome == ChildRegionChrome::Framed {
            builder = builder
                .p_2()
                .rounded_md()
                .border_1()
                .bg(crate::ColorRef::Token {
                    key: "card",
                    fallback: crate::ColorFallback::ThemePanelBackground,
                })
                .border_color(crate::ColorRef::Token {
                    key: "border",
                    fallback: crate::ColorFallback::ThemePanelBorder,
                });
        }

        if let Some(handle) = scroll_options.handle {
            builder = builder.handle(handle);
        }

        if let Some(test_id) = viewport_test_id {
            builder = builder.viewport_test_id(test_id);
        }

        if resize_x.is_none()
            && resize_y.is_none()
            && let Some(test_id) = test_id
        {
            builder = builder.test_id(test_id);
        }

        let scroll = builder.into_element(cx);
        let mut response = ChildRegionResponse::empty();

        let element = if resize_x.is_some() || resize_y.is_some() {
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
