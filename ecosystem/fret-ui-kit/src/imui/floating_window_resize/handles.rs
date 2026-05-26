use std::sync::Arc;

use fret_core::{CursorIcon, MouseButton, Px};
use fret_interaction::runtime_drag::{DragMoveOutcome, update_immediate_move};
use fret_ui::UiHost;
use fret_ui::element::{
    AnyElement, InsetStyle, LayoutStyle, Length, PointerRegionProps, PositionStyle,
};
use fret_ui::{ElementContext, GlobalElementId};

use super::super::{
    FloatWindowResizeHandle, KEY_FLOAT_WINDOW_ACTIVATE, float_layer_bring_to_front_if_activated,
    float_window_resize_kind_for_element,
};
use super::FloatingWindowResizeHandleTestIds;

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

fn resize_handle_layout(handle: FloatWindowResizeHandle) -> (CursorIcon, LayoutStyle) {
    match handle {
        FloatWindowResizeHandle::Left => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(6.0));
            layout.size.height = Length::Fill;
            (CursorIcon::ColResize, layout)
        }
        FloatWindowResizeHandle::Right => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                right: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(6.0));
            layout.size.height = Length::Fill;
            (CursorIcon::ColResize, layout)
        }
        FloatWindowResizeHandle::Top => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                right: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(6.0));
            (CursorIcon::RowResize, layout)
        }
        FloatWindowResizeHandle::Bottom => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                right: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(6.0));
            (CursorIcon::RowResize, layout)
        }
        FloatWindowResizeHandle::TopLeft => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(10.0));
            layout.size.height = Length::Px(Px(10.0));
            (CursorIcon::NwseResize, layout)
        }
        FloatWindowResizeHandle::TopRight => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                right: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(10.0));
            layout.size.height = Length::Px(Px(10.0));
            (CursorIcon::NeswResize, layout)
        }
        FloatWindowResizeHandle::BottomLeft => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(10.0));
            layout.size.height = Length::Px(Px(10.0));
            (CursorIcon::NeswResize, layout)
        }
        FloatWindowResizeHandle::BottomRight => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                right: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(10.0));
            layout.size.height = Length::Px(Px(10.0));
            (CursorIcon::NwseResize, layout)
        }
    }
}

fn resize_handle_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    handle: FloatWindowResizeHandle,
    test_id: Arc<str>,
    enable_activation: bool,
) -> AnyElement {
    let (cursor, layout) = resize_handle_layout(handle);

    let kind = float_window_resize_kind_for_element(window_id, handle);
    cx.pointer_region(
        PointerRegionProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let _region_id = cx.root_id();
            float_layer_bring_to_front_if_activated(cx, window_id);

            cx.pointer_region_clear_on_pointer_down();
            cx.pointer_region_clear_on_pointer_move();
            cx.pointer_region_clear_on_pointer_up();

            cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
                if down.button != MouseButton::Left {
                    return false;
                }

                host.request_focus(acx.target);
                host.capture_pointer();
                host.set_cursor_icon(cursor);
                if host.drag(down.pointer_id).is_none() {
                    host.begin_drag_with_kind(down.pointer_id, kind, acx.window, down.position);
                }
                if enable_activation {
                    host.record_transient_event(
                        fret_ui::action::ActionCx {
                            window: acx.window,
                            target: window_id,
                        },
                        KEY_FLOAT_WINDOW_ACTIVATE,
                    );
                }
                host.notify(acx);
                false
            }));

            cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                host.set_cursor_icon(cursor);

                let Some(drag) = host.drag_mut(mv.pointer_id) else {
                    return false;
                };
                if drag.kind != kind || drag.source_window != acx.window {
                    return false;
                }

                let outcome = update_immediate_move(drag, acx.window, mv.position, mv.buttons.left);
                if outcome == DragMoveOutcome::Canceled {
                    host.cancel_drag(mv.pointer_id);
                    host.release_pointer_capture();
                    host.notify(acx);
                    return false;
                }

                host.notify(acx);
                false
            }));

            cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                if let Some(drag) = host.drag(up.pointer_id)
                    && drag.kind == kind
                    && drag.source_window == acx.window
                {
                    host.cancel_drag(up.pointer_id);
                }
                host.release_pointer_capture();
                host.notify(acx);
                false
            }));

            Vec::new()
        },
    )
    .test_id(test_id)
}
