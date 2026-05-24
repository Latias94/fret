use std::sync::Arc;

use fret_core::{CursorIcon, MouseButton, Point, Px, Size};
use fret_interaction::runtime_drag::{DragMoveOutcome, update_immediate_move};
use fret_ui::UiHost;
use fret_ui::element::{
    AnyElement, InsetStyle, LayoutStyle, Length, PointerRegionProps, PositionStyle,
};
use fret_ui::{ElementContext, GlobalElementId};

pub(super) struct FloatingWindowResizeHandleTestIds {
    pub(super) left: Arc<str>,
    pub(super) right: Arc<str>,
    pub(super) top: Arc<str>,
    pub(super) bottom: Arc<str>,
    pub(super) top_left: Arc<str>,
    pub(super) top_right: Arc<str>,
    pub(super) bottom_left: Arc<str>,
    pub(super) bottom_right: Arc<str>,
}

pub(super) struct FloatingWindowResizeStateOutput {
    pub(super) position_after_resize: Point,
    pub(super) size: Size,
    pub(super) title_bar_test_id: Arc<str>,
    pub(super) close_button_test_id: Arc<str>,
    pub(super) handle_test_ids: FloatingWindowResizeHandleTestIds,
}

pub(super) fn prepare_resize_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    id: &str,
    area_position: Point,
    initial_size: Option<Size>,
    resize: Option<super::FloatingWindowResizeOptions>,
    resize_snapshot: Option<(super::FloatWindowResizeHandle, bool, Point, Point)>,
    collapsed: bool,
    scale_factor: f32,
) -> FloatingWindowResizeStateOutput {
    let (
        position_after_resize,
        size,
        title_bar_test_id,
        close_button_test_id,
        resize_left_test_id,
        resize_right_test_id,
        resize_top_test_id,
        resize_bottom_test_id,
        resize_top_left_test_id,
        resize_top_right_test_id,
        resize_bottom_left_test_id,
        resize_corner_test_id,
    ) = cx.state_for(
        window_id,
        || super::FloatWindowState {
            size: initial_size.unwrap_or_else(|| Size::new(Px(0.0), Px(0.0))),
            last_resize_position: None,
            title_bar_test_id: Arc::from(format!("imui.float_window.title_bar:{id}")),
            close_button_test_id: Arc::from(format!("imui.float_window.close:{id}")),
            resize_left_test_id: Arc::from(format!("imui.float_window.resize.left:{id}")),
            resize_right_test_id: Arc::from(format!("imui.float_window.resize.right:{id}")),
            resize_top_test_id: Arc::from(format!("imui.float_window.resize.top:{id}")),
            resize_bottom_test_id: Arc::from(format!("imui.float_window.resize.bottom:{id}")),
            resize_top_left_test_id: Arc::from(format!("imui.float_window.resize.top_left:{id}")),
            resize_top_right_test_id: Arc::from(format!("imui.float_window.resize.top_right:{id}")),
            resize_bottom_left_test_id: Arc::from(format!(
                "imui.float_window.resize.bottom_left:{id}"
            )),
            resize_corner_test_id: Arc::from(format!("imui.float_window.resize.corner:{id}")),
        },
        |st| {
            let mut position = area_position;

            let resize_cfg = resize.unwrap_or_default();
            let min = resize_cfg.min_size;
            let max = resize_cfg.max_size;
            let clamp_width = |value: f32| -> Px {
                let mut out = value.max(min.width.0);
                if let Some(max) = max {
                    out = out.min(max.width.0);
                }
                Px(out)
            };
            let clamp_height = |value: f32| -> Px {
                let mut out = value.max(min.height.0);
                if let Some(max) = max {
                    out = out.min(max.height.0);
                }
                Px(out)
            };

            if collapsed {
                st.last_resize_position = None;
            } else if let Some((handle, dragging, current, start)) = resize_snapshot {
                if dragging {
                    let prev = st.last_resize_position.unwrap_or(start);
                    let delta = super::point_sub(current, prev);

                    match handle {
                        super::FloatWindowResizeHandle::Left => {
                            let right = Px(position.x.0 + st.size.width.0);
                            let width = clamp_width(st.size.width.0 - delta.x.0);
                            st.size.width = width;
                            position.x = Px(right.0 - width.0);
                        }
                        super::FloatWindowResizeHandle::Right => {
                            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
                        }
                        super::FloatWindowResizeHandle::Top => {
                            let bottom = Px(position.y.0 + st.size.height.0);
                            let height = clamp_height(st.size.height.0 - delta.y.0);
                            st.size.height = height;
                            position.y = Px(bottom.0 - height.0);
                        }
                        super::FloatWindowResizeHandle::Bottom => {
                            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
                        }
                        super::FloatWindowResizeHandle::TopLeft => {
                            let right = Px(position.x.0 + st.size.width.0);
                            let bottom = Px(position.y.0 + st.size.height.0);

                            let width = clamp_width(st.size.width.0 - delta.x.0);
                            let height = clamp_height(st.size.height.0 - delta.y.0);
                            st.size.width = width;
                            st.size.height = height;
                            position.x = Px(right.0 - width.0);
                            position.y = Px(bottom.0 - height.0);
                        }
                        super::FloatWindowResizeHandle::TopRight => {
                            let bottom = Px(position.y.0 + st.size.height.0);
                            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
                            let height = clamp_height(st.size.height.0 - delta.y.0);
                            st.size.height = height;
                            position.y = Px(bottom.0 - height.0);
                        }
                        super::FloatWindowResizeHandle::BottomLeft => {
                            let right = Px(position.x.0 + st.size.width.0);
                            let width = clamp_width(st.size.width.0 - delta.x.0);
                            st.size.width = width;
                            position.x = Px(right.0 - width.0);
                            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
                        }
                        super::FloatWindowResizeHandle::BottomRight => {
                            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
                            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
                        }
                    }

                    st.last_resize_position = Some(current);
                } else {
                    st.last_resize_position = None;
                }
            } else {
                st.last_resize_position = None;
            }

            st.size = super::snap_size_to_device_pixels(scale_factor, st.size);
            position = super::snap_point_to_device_pixels(scale_factor, position);

            (
                position,
                st.size,
                st.title_bar_test_id.clone(),
                st.close_button_test_id.clone(),
                st.resize_left_test_id.clone(),
                st.resize_right_test_id.clone(),
                st.resize_top_test_id.clone(),
                st.resize_bottom_test_id.clone(),
                st.resize_top_left_test_id.clone(),
                st.resize_top_right_test_id.clone(),
                st.resize_bottom_left_test_id.clone(),
                st.resize_corner_test_id.clone(),
            )
        },
    );

    FloatingWindowResizeStateOutput {
        position_after_resize,
        size,
        title_bar_test_id,
        close_button_test_id,
        handle_test_ids: FloatingWindowResizeHandleTestIds {
            left: resize_left_test_id,
            right: resize_right_test_id,
            top: resize_top_test_id,
            bottom: resize_bottom_test_id,
            top_left: resize_top_left_test_id,
            top_right: resize_top_right_test_id,
            bottom_left: resize_bottom_left_test_id,
            bottom_right: resize_corner_test_id,
        },
    }
}

pub(super) fn resize_stack_element<H: UiHost>(
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

    let mut resize_handle = |handle: super::FloatWindowResizeHandle, test_id: Arc<str>| {
        resize_handle_element(cx, window_id, handle, test_id, enable_activation)
    };
    let mut stacked: Vec<AnyElement> = vec![
        body,
        resize_handle(super::FloatWindowResizeHandle::Left, test_ids.left),
        resize_handle(super::FloatWindowResizeHandle::Right, test_ids.right),
        resize_handle(super::FloatWindowResizeHandle::Top, test_ids.top),
        resize_handle(super::FloatWindowResizeHandle::Bottom, test_ids.bottom),
        resize_handle(super::FloatWindowResizeHandle::TopLeft, test_ids.top_left),
        resize_handle(super::FloatWindowResizeHandle::TopRight, test_ids.top_right),
        resize_handle(
            super::FloatWindowResizeHandle::BottomLeft,
            test_ids.bottom_left,
        ),
        resize_handle(
            super::FloatWindowResizeHandle::BottomRight,
            test_ids.bottom_right,
        ),
    ];

    if let Some(blocker) = blocker {
        stacked.push(blocker);
    }

    cx.stack(move |_cx| stacked)
}

pub(super) fn resize_handle_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    handle: super::FloatWindowResizeHandle,
    test_id: Arc<str>,
    enable_activation: bool,
) -> AnyElement {
    let (cursor, layout) = match handle {
        super::FloatWindowResizeHandle::Left => {
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
        super::FloatWindowResizeHandle::Right => {
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
        super::FloatWindowResizeHandle::Top => {
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
        super::FloatWindowResizeHandle::Bottom => {
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
        super::FloatWindowResizeHandle::TopLeft => {
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
        super::FloatWindowResizeHandle::TopRight => {
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
        super::FloatWindowResizeHandle::BottomLeft => {
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
        super::FloatWindowResizeHandle::BottomRight => {
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
    };

    let kind = super::float_window_resize_kind_for_element(window_id, handle);
    cx.pointer_region(
        PointerRegionProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let _region_id = cx.root_id();
            super::float_layer_bring_to_front_if_activated(cx, window_id);

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
                        super::KEY_FLOAT_WINDOW_ACTIVATE,
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
