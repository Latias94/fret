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

        let resize_state = super::floating_window_resize::prepare_resize_state(
            cx,
            window_id,
            id,
            area.position,
            initial_size,
            resize,
            resize_snapshot,
            collapsed,
            scale_factor,
        );

        if resize_state.position_after_resize != area.position {
            cx.state_for(
                window_id,
                || super::FloatingAreaState {
                    position: initial_position,
                    last_drag_position: None,
                    test_id: Arc::from(format!("imui.float_window.window:{id}")),
                },
                |st| {
                    st.position = resize_state.position_after_resize;
                },
            );
        }

        let chrome = super::FloatingWindowChromeResponse {
            size: resizable_layout.then_some(resize_state.size),
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
            window_props.layout.size.width = Length::Px(resize_state.size.width);
            if !collapsed {
                window_props.layout.size.height = Length::Px(resize_state.size.height);
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
                            resize_state.title_bar_test_id.clone(),
                            resize_state.close_button_test_id.clone(),
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
                resize_state.handle_test_ids,
            );

            vec![stacked_body]
        });
        (window, chrome)
    });

    ui.add(window);
    chrome
}
