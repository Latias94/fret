//! Immediate child-region helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{CursorIcon, Px};
use fret_ui::element::{
    AnyElement, InsetStyle, LayoutStyle, Length, PointerRegionProps, PositionStyle, StackProps,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};

use super::{
    ChildRegionChrome, ChildRegionOptions, ChildRegionResizeYOptions, ChildRegionResponse,
    ImUiFacade, ResponseExt, containers::build_imui_children_with_focus,
};

const CHILD_REGION_RESIZE_Y_HANDLE_HEIGHT: Px = Px(6.0);

#[derive(Default)]
struct ChildRegionResizeYDragState {
    was_dragging: bool,
}

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
        let resize_y = options.resize_y.clone();
        let scroll_layout = if resize_y.is_some() {
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

        if resize_y.is_none()
            && let Some(test_id) = test_id
        {
            builder = builder.test_id(test_id);
        }

        let scroll = builder.into_element(cx);
        let mut response = ChildRegionResponse::empty();

        let element = if let Some(resize_options) = resize_y {
            let handle_test_id = resize_options.handle_test_id.clone().or_else(|| {
                root_test_id
                    .as_ref()
                    .map(|base| Arc::from(format!("{base}.resize-y")))
            });
            let handle =
                child_region_resize_y_handle(cx, id, resize_options, handle_test_id, &mut response);

            let mut stack = StackProps::default();
            stack.layout = crate::declarative::style::layout_style(Theme::global(&*cx.app), layout);

            let stack = cx.stack_props(stack, move |_cx| vec![scroll, handle]);
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

fn child_region_resize_y_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    options: ChildRegionResizeYOptions,
    test_id: Option<Arc<str>>,
    response: &mut ChildRegionResponse,
) -> AnyElement {
    let enabled = !super::imui_is_disabled(cx);
    let resize_response = response.resize_y_mut();
    resize_response.enabled = enabled;
    resize_response.min_height = options.min_height;
    resize_response.max_height = options.max_height;

    let handle = cx.keyed(("child-region-resize-y", id), |cx| {
        let mut layout = LayoutStyle::default();
        layout.position = PositionStyle::Absolute;
        layout.inset = InsetStyle {
            left: Some(Px(0.0)).into(),
            right: Some(Px(0.0)).into(),
            bottom: Some(Px(0.0)).into(),
            ..Default::default()
        };
        layout.size.width = Length::Fill;
        layout.size.height = Length::Px(CHILD_REGION_RESIZE_Y_HANDLE_HEIGHT);

        let mut props = PointerRegionProps::default();
        props.enabled = enabled;
        props.layout = layout;

        cx.pointer_region(props, move |cx| {
            let region_id = cx.root_id();
            let drag_kind = super::drag_kind_for_element(region_id);
            let drag_threshold = super::drag_threshold_for(cx);

            cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
                super::prepare_pointer_region_drag_on_left_down(
                    host,
                    acx,
                    down,
                    enabled.then_some(drag_kind),
                    Some(CursorIcon::RowResize),
                )
            }));
            cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                if !enabled {
                    return false;
                }
                host.set_cursor_icon(CursorIcon::RowResize);
                super::handle_pointer_region_drag_move_with_threshold(
                    host,
                    acx,
                    mv,
                    drag_kind,
                    drag_threshold,
                )
            }));
            cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                if !enabled {
                    return false;
                }
                super::finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
            }));

            let mut drag_response = ResponseExt::default();
            super::populate_pressable_drag_response(cx, region_id, &mut drag_response);
            resize_response.drag = drag_response.drag();

            let dragging = resize_response.drag.dragging();
            let (started, stopped) =
                cx.state_for(region_id, ChildRegionResizeYDragState::default, |state| {
                    let started = dragging && !state.was_dragging;
                    let stopped = !dragging && state.was_dragging;
                    state.was_dragging = dragging;
                    (started, stopped)
                });
            resize_response.drag.merge_edges({
                let mut edges = super::DragResponse::default();
                edges.set_started(started);
                edges.set_stopped(stopped);
                edges
            });

            Vec::new()
        })
    });

    if let Some(test_id) = test_id {
        handle.test_id(test_id)
    } else {
        handle
    }
}
