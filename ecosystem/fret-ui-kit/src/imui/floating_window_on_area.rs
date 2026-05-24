use std::sync::Arc;

use fret_authoring::UiWriter as _;
use fret_core::{Corners, Edges, Point, Px, Size};
use fret_ui::UiHost;
use fret_ui::element::{ColumnProps, ContainerProps, Length, Overflow};

pub(super) fn render_floating_window_in_area<H: UiHost, Build>(
    ui: &mut super::ImUiFacade<'_, '_, H>,
    area: super::FloatingAreaContext,
    id: &str,
    title: Arc<str>,
    open_model: Option<fret_runtime::Model<bool>>,
    initial_position: Point,
    initial_size: Option<Size>,
    resize: Option<super::FloatingWindowResizeOptions>,
    options: super::FloatingWindowOptions,
    build: Build,
) -> super::FloatingWindowChromeResponse
where
    Build: for<'cx2, 'a2> FnOnce(&mut super::ImUiFacade<'cx2, 'a2, H>),
{
    let (window, chrome) = ui.with_cx_mut(|cx| {
        let window_id = area.id;
        let resizable_layout = initial_size.is_some();
        let resize_enabled = options.inputs_enabled && options.resizable && resizable_layout;

        let resize_snapshot = if resize_enabled {
            [
                super::FloatWindowResizeHandle::Left,
                super::FloatWindowResizeHandle::Right,
                super::FloatWindowResizeHandle::Top,
                super::FloatWindowResizeHandle::Bottom,
                super::FloatWindowResizeHandle::TopLeft,
                super::FloatWindowResizeHandle::TopRight,
                super::FloatWindowResizeHandle::BottomLeft,
                super::FloatWindowResizeHandle::BottomRight,
            ]
            .into_iter()
            .find_map(|handle| {
                let kind = super::float_window_resize_kind_for_element(window_id, handle);
                cx.app
                    .find_drag_pointer_id(|d| {
                        d.kind == kind
                            && d.source_window == cx.window
                            && d.current_window == cx.window
                    })
                    .and_then(|pointer_id| cx.app.drag(pointer_id))
                    .filter(|drag| drag.kind == kind)
                    .map(|drag| (handle, drag.dragging, drag.position, drag.start_position))
            })
        } else {
            None
        };
        let resizing = resize_snapshot
            .map(|(_, dragging, _, _)| dragging)
            .unwrap_or(false);
        let collapsed_model = super::float_window_collapsed_model_for(cx, window_id);
        if options.inputs_enabled
            && options.collapsible
            && cx.take_transient_for(window_id, super::KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED)
        {
            let _ = cx.app.models_mut().update(&collapsed_model, |v| {
                *v = !*v;
            });
        }
        let collapsed = cx
            .read_model(&collapsed_model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);

        let scale_factor = cx
            .app
            .global::<fret_core::window::WindowMetricsService>()
            .and_then(|svc| svc.scale_factor(cx.window))
            .unwrap_or(1.0);

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
                resize_top_left_test_id: Arc::from(format!(
                    "imui.float_window.resize.top_left:{id}"
                )),
                resize_top_right_test_id: Arc::from(format!(
                    "imui.float_window.resize.top_right:{id}"
                )),
                resize_bottom_left_test_id: Arc::from(format!(
                    "imui.float_window.resize.bottom_left:{id}"
                )),
                resize_corner_test_id: Arc::from(format!("imui.float_window.resize.corner:{id}")),
            },
            |st| {
                let mut position = area.position;

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

        if position_after_resize != area.position {
            cx.state_for(
                window_id,
                || super::FloatingAreaState {
                    position: initial_position,
                    last_drag_position: None,
                    test_id: Arc::from(format!("imui.float_window.window:{id}")),
                },
                |st| {
                    st.position = position_after_resize;
                },
            );
        }

        let chrome = super::FloatingWindowChromeResponse {
            size: resizable_layout.then_some(size),
            resizing: resizing && !collapsed,
            collapsed,
        };

        let (popover, border, muted) = {
            let theme = fret_ui::Theme::global(&*cx.app);
            (
                theme.color_token("popover"),
                theme.color_token("border"),
                theme.color_token("muted"),
            )
        };

        let mut window_props = ContainerProps::default();
        if resizable_layout {
            window_props.layout.size.width = Length::Px(size.width);
            if !collapsed {
                window_props.layout.size.height = Length::Px(size.height);
            }
        }
        window_props.background = Some(popover);
        window_props.border = Edges::all(Px(1.0));
        window_props.border_color = Some(border);
        window_props.corner_radii = Corners::all(super::control_chrome::PANEL_RADIUS);

        let title_for_window = title.clone();
        let open_for_window = open_model.clone();

        let window = cx.container(window_props, move |cx| {
            let mut col = ColumnProps::default();
            col.layout.size.width = if resizable_layout {
                Length::Fill
            } else {
                Length::Auto
            };
            col.layout.size.height = if resizable_layout && !collapsed {
                Length::Fill
            } else {
                Length::Auto
            };

            let title_bar = cx.container(
                {
                    let mut props = ContainerProps::default();
                    props.layout.size.width = if resizable_layout {
                        Length::Fill
                    } else {
                        Length::Auto
                    };
                    props.layout.size.height = Length::Px(Px(24.0));
                    // Prevent multi-line title text from painting into the content area at
                    // non-1.0 DPI when the layout engine probes min-content widths.
                    props.layout.overflow = Overflow::Clip;
                    props.padding = Edges {
                        left: Px(6.0),
                        right: Px(4.0),
                        top: Px(4.0),
                        bottom: Px(4.0),
                    }
                    .into();
                    props.background = Some(muted);
                    props.border = Edges {
                        left: Px(0.0),
                        right: Px(0.0),
                        top: Px(0.0),
                        bottom: Px(1.0),
                    };
                    props.border_color = Some(border);
                    props.corner_radii = Corners {
                        top_left: super::control_chrome::PANEL_RADIUS,
                        top_right: super::control_chrome::PANEL_RADIUS,
                        bottom_left: Px(0.0),
                        bottom_right: Px(0.0),
                    };
                    props
                },
                move |cx| {
                    vec![
                        super::floating_window_title_bar::floating_window_title_bar_row(
                            cx,
                            area,
                            title_for_window.clone(),
                            open_for_window.clone(),
                            title_bar_test_id.clone(),
                            close_button_test_id.clone(),
                            resizable_layout,
                            options,
                        ),
                    ]
                },
            );

            let content = super::floating_window_content::floating_window_content_element(
                cx,
                window_id,
                resizable_layout,
                options,
                build,
            );

            let body = if collapsed {
                title_bar
            } else {
                cx.column(col, move |_cx| vec![title_bar, content])
            };

            let clipped_body = cx.container(
                {
                    let mut props = ContainerProps::default();
                    // Clip window contents to the window bounds (ImGui-style): items should not paint outside
                    // the window chrome even when they don't wrap. Keep this as an inner clip container so
                    // resize handles can still receive hits near rounded corners.
                    props.layout.overflow = Overflow::Clip;
                    props.layout.size.width = if resizable_layout {
                        Length::Fill
                    } else {
                        Length::Auto
                    };
                    props.layout.size.height = if resizable_layout && !collapsed {
                        Length::Fill
                    } else {
                        Length::Auto
                    };
                    props.corner_radii = Corners::all(super::control_chrome::PANEL_RADIUS);
                    props
                },
                move |_cx| vec![body],
            );

            let blocker = super::floating_window_blocker::floating_window_blocker_element(
                cx,
                options.inputs_enabled,
            );

            let stacked_body = super::floating_window_resize::resize_stack_element(
                cx,
                window_id,
                clipped_body,
                blocker,
                resizable_layout,
                collapsed,
                resize_enabled,
                options.activate_on_click,
                super::floating_window_resize::FloatingWindowResizeHandleTestIds {
                    left: resize_left_test_id.clone(),
                    right: resize_right_test_id.clone(),
                    top: resize_top_test_id.clone(),
                    bottom: resize_bottom_test_id.clone(),
                    top_left: resize_top_left_test_id.clone(),
                    top_right: resize_top_right_test_id.clone(),
                    bottom_left: resize_bottom_left_test_id.clone(),
                    bottom_right: resize_corner_test_id.clone(),
                },
            );

            vec![stacked_body]
        });
        (window, chrome)
    });

    ui.add(window);
    chrome
}
