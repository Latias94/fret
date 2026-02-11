use std::sync::Arc;

use fret_authoring::UiWriter as _;
use fret_core::{Corners, CursorIcon, Edges, KeyCode, MouseButton, Point, Px, SemanticsRole, Size};
use fret_interaction::runtime_drag::{DragMoveOutcome, update_immediate_move};
use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow,
    PointerRegionProps, PositionStyle, PressableA11y, PressableProps, RowProps, ScrollAxis,
    ScrollProps,
};

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
        ) = cx.with_state_for(
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
            cx.with_state_for(
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
                theme.color_required("popover"),
                theme.color_required("border"),
                theme.color_required("muted"),
            )
        };

        let mut window_props = ContainerProps::default();
        // Clip window contents to the window bounds (ImGui-style): items should not paint outside
        // the window chrome even when they don't wrap.
        window_props.layout.overflow = Overflow::Clip;
        if resizable_layout {
            window_props.layout.size.width = Length::Px(size.width);
            if !collapsed {
                window_props.layout.size.height = Length::Px(size.height);
            }
        }
        window_props.background = Some(popover);
        window_props.border = Edges::all(Px(1.0));
        window_props.border_color = Some(border);
        window_props.corner_radii = Corners::all(Px(8.0));

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
                        left: Px(8.0),
                        right: Px(6.0),
                        top: Px(4.0),
                        bottom: Px(4.0),
                    };
                    props.background = Some(muted);
                    props.border = Edges {
                        left: Px(0.0),
                        right: Px(0.0),
                        top: Px(0.0),
                        bottom: Px(1.0),
                    };
                    props.border_color = Some(border);
                    props.corner_radii = Corners {
                        top_left: Px(8.0),
                        top_right: Px(8.0),
                        bottom_left: Px(0.0),
                        bottom_right: Px(0.0),
                    };
                    props
                },
                move |cx| {
                    let mut row = RowProps::default();
                    row.layout.size.width = if resizable_layout {
                        Length::Fill
                    } else {
                        Length::Auto
                    };
                    row.layout.size.height = Length::Fill;
                    row.gap = Px(6.0);
                    row.align = fret_ui::element::CrossAlign::Center;

                    let title = title_for_window.clone();
                    let title_bar_test_id = title_bar_test_id.clone();
                    let open_for_key = open_for_window.clone();
                    let can_interact = options.inputs_enabled;
                    let can_close = can_interact && options.closable && open_for_key.is_some();
                    let can_collapse = can_interact && options.collapsible;
                    let can_move = can_interact && options.movable;
                    let on_left_double_click: Option<super::OnFloatingAreaLeftDoubleClick> =
                        if can_collapse {
                            Some(Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      acx: fret_ui::action::ActionCx| {
                                    host.record_transient_event(
                                        acx,
                                        super::KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED,
                                    );
                                    host.notify(acx);
                                },
                            ))
                        } else {
                            None
                        };

                    let drag_surface = super::floating_area_drag_surface_element(
                        cx,
                        area,
                        PointerRegionProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                // Ensure the drag surface claims remaining row space (and can
                                // shrink) instead of being measured in min-content mode (which
                                // can force wrapped titles like "Window" + "A").
                                layout.flex.grow = 1.0;
                                layout.flex.shrink = 1.0;
                                layout.flex.basis = Length::Px(Px(0.0));
                                layout.size.min_width = Some(Px(0.0));
                                layout
                            },
                            enabled: can_interact,
                            ..Default::default()
                        },
                        on_left_double_click,
                        can_move,
                        options.activate_on_click,
                        move |cx, region_id| {
                            cx.key_clear_on_key_down_for(region_id);
                            if can_close && let Some(open) = open_for_key {
                                cx.key_on_key_down_for(
                                    region_id,
                                    Arc::new(move |host, acx, down| {
                                        if down.key != KeyCode::Escape || down.repeat {
                                            return false;
                                        }
                                        let _ = host.update_model(&open, |v: &mut bool| {
                                            *v = false;
                                        });
                                        host.notify(acx);
                                        true
                                    }),
                                );
                            }
                        },
                        move |ui| {
                            let element = ui.with_cx_mut(|cx| {
                                let mut props = fret_ui::element::TextProps::new(title.clone());
                                props.layout.size.width = Length::Fill;
                                props.layout.size.min_width = Some(Px(0.0));
                                props.layout.flex.grow = 1.0;
                                props.layout.flex.shrink = 1.0;
                                props.layout.flex.basis = Length::Px(Px(0.0));
                                props.wrap = fret_core::TextWrap::None;
                                props.overflow = fret_core::TextOverflow::Ellipsis;
                                cx.text_props(props).attach_semantics(
                                    fret_ui::element::SemanticsDecoration::default()
                                        .test_id(title_bar_test_id.clone()),
                                )
                            });
                            ui.add(element);
                        },
                    );

                    let close = (options.inputs_enabled && options.closable)
                        .then(|| open_for_window.clone())
                        .flatten()
                        .map(|open| {
                            let mut props = PressableProps::default();
                            props.a11y = PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(Arc::from("Close")),
                                test_id: Some(close_button_test_id.clone()),
                                ..Default::default()
                            };
                            props.layout.size.width = Length::Px(Px(20.0));
                            props.layout.size.height = Length::Px(Px(20.0));
                            props.layout.flex.shrink = 0.0;
                            cx.pressable(props, move |cx, _state| {
                                cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                                    let _ = host.update_model(&open, |v: &mut bool| {
                                        *v = false;
                                    });
                                    host.notify(acx);
                                }));
                                vec![cx.text("\u{00D7}")]
                            })
                        });

                    vec![cx.row(row, move |_cx| {
                        let mut out = vec![drag_surface];
                        if let Some(close) = close {
                            out.push(close);
                        }
                        out
                    })]
                },
            );

            let content = {
                let content_container = |cx: &mut ElementContext<'_, H>| {
                    let handle =
                        cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
                    let mut scroll_layout = LayoutStyle::default();
                    if resizable_layout {
                        scroll_layout.size.width = Length::Fill;
                        scroll_layout.size.height = Length::Fill;
                    } else {
                        scroll_layout.size.width = Length::Auto;
                        scroll_layout.size.height = Length::Auto;
                    }
                    scroll_layout.overflow = Overflow::Clip;

                    cx.scroll(
                        ScrollProps {
                            layout: scroll_layout,
                            axis: ScrollAxis::Y,
                            scroll_handle: Some(handle),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = if resizable_layout {
                                        Length::Fill
                                    } else {
                                        Length::Auto
                                    };
                                    props.padding = Edges::all(Px(8.0));
                                    props
                                },
                                move |cx| {
                                    let mut out = Vec::new();
                                    let mut ui = super::ImUiFacade {
                                        cx,
                                        out: &mut out,
                                        build_focus: None,
                                    };
                                    build(&mut ui);
                                    out
                                },
                            )]
                        },
                    )
                };

                if options.inputs_enabled && (options.activate_on_click || options.focus_on_click) {
                    let layout = {
                        let mut layout = LayoutStyle::default();
                        if resizable_layout {
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                        } else {
                            layout.size.width = Length::Auto;
                            layout.size.height = Length::Auto;
                        }
                        layout
                    };
                    let focus_on_click = options.focus_on_click;
                    let activate_on_click = options.activate_on_click;
                    cx.pointer_region(
                        PointerRegionProps {
                            layout,
                            enabled: true,
                        },
                        move |cx| {
                            let region_id = cx.root_id();
                            super::float_layer_bring_to_front_if_activated(
                                cx, region_id, window_id,
                            );
                            // Make the surface focusable so `request_focus(...)` is effective even
                            // when the click lands on a non-focusable background area.
                            cx.key_on_key_down_for(region_id, Arc::new(|_host, _acx, _down| false));

                            cx.pointer_region_clear_on_pointer_down();
                            cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, _down| {
                                if focus_on_click {
                                    host.request_focus(acx.target);
                                }
                                if activate_on_click {
                                    host.record_transient_event(
                                        acx,
                                        super::KEY_FLOAT_WINDOW_ACTIVATE,
                                    );
                                }
                                host.notify(acx);
                                false
                            }));

                            vec![content_container(cx)]
                        },
                    )
                } else {
                    content_container(cx)
                }
            };

            let body = if collapsed {
                title_bar
            } else {
                cx.column(col, move |_cx| vec![title_bar, content])
            };

            let blocker = (!options.inputs_enabled).then(|| {
                let mut layout = LayoutStyle::default();
                layout.position = PositionStyle::Absolute;
                layout.inset = InsetStyle {
                    left: Some(Px(0.0)),
                    right: Some(Px(0.0)),
                    top: Some(Px(0.0)),
                    bottom: Some(Px(0.0)),
                };
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;

                cx.pointer_region(
                    PointerRegionProps {
                        layout,
                        enabled: true,
                    },
                    move |cx| {
                        cx.pointer_region_clear_on_pointer_down();
                        cx.pointer_region_clear_on_pointer_move();
                        cx.pointer_region_clear_on_pointer_up();

                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _acx, _down| true));
                        cx.pointer_region_on_pointer_move(Arc::new(|_host, _acx, _mv| true));
                        cx.pointer_region_on_pointer_up(Arc::new(|_host, _acx, _up| true));
                        Vec::new()
                    },
                )
            });

            if !resizable_layout || collapsed || !resize_enabled {
                if let Some(blocker) = blocker {
                    return vec![cx.stack(move |_cx| vec![body, blocker])];
                }
                return vec![body];
            }

            let enable_activation = options.activate_on_click;
            let mut resize_handle = |handle: super::FloatWindowResizeHandle, test_id: Arc<str>| {
                let (cursor, layout) = match handle {
                    super::FloatWindowResizeHandle::Left => {
                        let mut layout = LayoutStyle::default();
                        layout.position = PositionStyle::Absolute;
                        layout.inset = InsetStyle {
                            left: Some(Px(0.0)),
                            top: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
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
                            right: Some(Px(0.0)),
                            top: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
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
                            left: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            top: Some(Px(0.0)),
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
                            left: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
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
                            left: Some(Px(0.0)),
                            top: Some(Px(0.0)),
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
                            right: Some(Px(0.0)),
                            top: Some(Px(0.0)),
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
                            left: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
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
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
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
                        let region_id = cx.root_id();
                        super::float_layer_bring_to_front_if_activated(cx, region_id, window_id);

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
                                host.begin_drag_with_kind(
                                    down.pointer_id,
                                    kind,
                                    acx.window,
                                    down.position,
                                );
                            }
                            if enable_activation {
                                host.record_transient_event(acx, super::KEY_FLOAT_WINDOW_ACTIVATE);
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

                            let outcome = update_immediate_move(
                                drag,
                                acx.window,
                                mv.position,
                                mv.buttons.left,
                            );
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
                .test_id(test_id.clone())
            };

            let mut stacked: Vec<AnyElement> = Vec::new();
            stacked.push(body);

            stacked.push(resize_handle(
                super::FloatWindowResizeHandle::Left,
                resize_left_test_id.clone(),
            ));
            stacked.push(resize_handle(
                super::FloatWindowResizeHandle::Right,
                resize_right_test_id.clone(),
            ));
            stacked.push(resize_handle(
                super::FloatWindowResizeHandle::Top,
                resize_top_test_id.clone(),
            ));
            stacked.push(resize_handle(
                super::FloatWindowResizeHandle::Bottom,
                resize_bottom_test_id.clone(),
            ));
            stacked.push(resize_handle(
                super::FloatWindowResizeHandle::TopLeft,
                resize_top_left_test_id.clone(),
            ));
            stacked.push(resize_handle(
                super::FloatWindowResizeHandle::TopRight,
                resize_top_right_test_id.clone(),
            ));
            stacked.push(resize_handle(
                super::FloatWindowResizeHandle::BottomLeft,
                resize_bottom_left_test_id.clone(),
            ));
            stacked.push(resize_handle(
                super::FloatWindowResizeHandle::BottomRight,
                resize_corner_test_id.clone(),
            ));

            if let Some(blocker) = blocker {
                stacked.push(blocker);
            }

            vec![cx.stack(move |_cx| stacked)]
        });
        (window, chrome)
    });

    ui.add(window);
    chrome
}
