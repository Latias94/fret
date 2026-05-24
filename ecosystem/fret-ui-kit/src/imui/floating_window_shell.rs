use fret_core::{Color, Corners, Edges, Px, Size};
use fret_ui::element::{AnyElement, ColumnProps, ContainerProps, Length, Overflow};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

pub(super) fn floating_window_shell_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    title_bar_row: AnyElement,
    content: AnyElement,
    window_size: Size,
    resizable_layout: bool,
    collapsed: bool,
    resize_enabled: bool,
    options: super::FloatingWindowOptions,
    handle_test_ids: super::floating_window_resize::FloatingWindowResizeHandleTestIds,
) -> AnyElement {
    let (popover, border, muted) = {
        let theme = fret_ui::Theme::global(&*cx.app);
        (
            theme.color_token("popover"),
            theme.color_token("border"),
            theme.color_token("muted"),
        )
    };

    let window_props =
        window_frame_props(window_size, resizable_layout, collapsed, popover, border);

    cx.container(window_props, move |cx| {
        let col = shell_column_props(resizable_layout, collapsed);

        let title_bar = cx.container(
            title_bar_container_props(resizable_layout, muted, border),
            move |_cx| vec![title_bar_row],
        );

        let body = if collapsed {
            title_bar
        } else {
            cx.column(col, move |_cx| vec![title_bar, content])
        };

        let clipped_body = cx.container(
            clipped_body_props(resizable_layout, collapsed),
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
            handle_test_ids,
        );

        vec![stacked_body]
    })
}

fn window_frame_props(
    window_size: Size,
    resizable_layout: bool,
    collapsed: bool,
    popover: Color,
    border: Color,
) -> ContainerProps {
    let mut props = ContainerProps::default();
    if resizable_layout {
        props.layout.size.width = Length::Px(window_size.width);
        if !collapsed {
            props.layout.size.height = Length::Px(window_size.height);
        }
    }
    props.background = Some(popover);
    props.border = Edges::all(Px(1.0));
    props.border_color = Some(border);
    props.corner_radii = Corners::all(super::control_chrome::PANEL_RADIUS);
    props
}

fn shell_column_props(resizable_layout: bool, collapsed: bool) -> ColumnProps {
    let mut props = ColumnProps::default();
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
    props
}

fn title_bar_container_props(
    resizable_layout: bool,
    muted: Color,
    border: Color,
) -> ContainerProps {
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
}

fn clipped_body_props(resizable_layout: bool, collapsed: bool) -> ContainerProps {
    let mut props = ContainerProps::default();
    // Clip window contents to the window bounds (ImGui-style): items should not paint outside the
    // window chrome even when they don't wrap. Keep this as an inner clip container so resize
    // handles can still receive hits near rounded corners.
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
}
